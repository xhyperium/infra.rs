//! OKX `VenueAdapter` scaffold（可选注入 `transportx::HttpDriver`）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderRef, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use contracts::{
    AccountSource, ExecutionVenue, InstrumentCatalog, MarketDataSource, VenueAdapter,
    VenueTimeSource,
};
use decimalx::{Currency, Decimal, Qty};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};
use transportx::{HttpDriver, HttpRequest, HttpResponse, TransportError};

/// 解析 OKX `/api/v5/public/time` JSON：`data[0].ts` 毫秒字符串。
///
/// 离线可调用（bench / 单元测试）；不发起网络。
pub fn parse_okx_server_time(body: &[u8]) -> XResult<i64> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("server_time json: {e}")))?;
    let ts = v
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .and_then(|o| o.get("ts"))
        .and_then(|x| x.as_str())
        .ok_or_else(|| XError::invalid("server_time missing data[0].ts"))?;
    ts.parse::<i64>().map_err(|e| XError::invalid(format!("server_time ts parse: {e}")))
}

fn map_transport_error(err: TransportError) -> XError {
    match err {
        TransportError::RateLimited { .. } => XError::transient("transport rate limited"),
        TransportError::ConnectTimeout => XError::transient("transport connect timeout"),
        TransportError::ReadTimeout => XError::transient("transport read timeout"),
        TransportError::ConnectionClosed { clean } => {
            XError::unavailable(format!("transport connection closed (clean={clean})"))
        }
        TransportError::ProtocolViolation(msg) => XError::invalid(format!("transport: {msg}")),
        TransportError::Io(source) => XError::unavailable(format!("transport io: {source}")),
        TransportError::PayloadTooLarge { kind, limit, got } => {
            XError::invalid(format!("transport payload too large: {kind} limit={limit} got={got}"))
        }
    }
}

/// 适配器连接状态（观测用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Disconnected,
    Connected,
}

/// OKX adapter scaffold。
///
/// 默认无 HTTP 驱动。通过 [`Self::with_http`] 注入 transportx 后走 HTTP 边界。
pub struct OkxAdapter {
    name: String,
    base_url: String,
    connected: AtomicBool,
    http: Option<Arc<dyn HttpDriver>>,
}

impl OkxAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            connected: AtomicBool::new(false),
            http: None,
        }
    }

    pub fn demo() -> Self {
        Self::new("okx-demo", "https://www.okx.com")
    }

    pub fn mainnet() -> Self {
        Self::new("okx-mainnet", "https://www.okx.com")
    }

    /// 注入 HTTP 驱动。
    #[must_use]
    pub fn with_http(mut self, http: Arc<dyn HttpDriver>) -> Self {
        self.http = Some(http);
        self
    }

    /// 是否已配置 HTTP 驱动。
    #[must_use]
    pub fn has_http(&self) -> bool {
        self.http.is_some()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn state(&self) -> AdapterState {
        if self.connected.load(Ordering::SeqCst) {
            AdapterState::Connected
        } else {
            AdapterState::Disconnected
        }
    }

    fn require_connected(&self) -> XResult<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(XError::unavailable("not connected"));
        }
        Ok(())
    }

    fn join_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let base = self.base_url.trim_end_matches('/');
        if path.starts_with('/') { format!("{base}{path}") } else { format!("{base}/{path}") }
    }

    /// 经注入的 [`HttpDriver`] 发起 GET。
    pub async fn http_get(&self, path: &str) -> XResult<HttpResponse> {
        let http =
            self.http.as_ref().ok_or_else(|| XError::unavailable("http driver not configured"))?;
        let request = HttpRequest {
            method: "GET".into(),
            url: self.join_url(path),
            headers: vec![],
            body: None,
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    /// 经注入的 [`HttpDriver`] 发起 POST。
    pub async fn http_post(&self, path: &str, body: Bytes) -> XResult<HttpResponse> {
        let http =
            self.http.as_ref().ok_or_else(|| XError::unavailable("http driver not configured"))?;
        let request = HttpRequest {
            method: "POST".into(),
            url: self.join_url(path),
            headers: vec![],
            body: Some(body),
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    fn zero_qty() -> XResult<Qty> {
        Ok(Qty::new(
            Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero qty: {e}")))?,
        ))
    }
}

#[async_trait]
impl VenueAdapter for OkxAdapter {
    async fn connect(&self) -> XResult<()> {
        if self.connected.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err()
        {
            return Err(XError::conflict("already connected"));
        }
        Ok(())
    }

    async fn disconnect(&self) -> XResult<()> {
        if self.connected.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_err()
        {
            return Err(XError::unavailable("not connected"));
        }
        Ok(())
    }

    async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
        self.require_connected()?;
        Ok(OrderAck { id: order.id.clone(), status: OrderStatus::Open, ts: 0 })
    }

    #[allow(deprecated)]
    async fn cancel_order(&self, _id: &str) -> XResult<()> {
        self.require_connected()?;
        Ok(())
    }

    #[allow(deprecated)]
    async fn query_order(&self, _id: &str) -> XResult<OrderStatus> {
        self.require_connected()?;
        Ok(OrderStatus::Open)
    }

    async fn cancel_order_request(&self, request: &CancelOrderRequest) -> XResult<()> {
        self.require_connected()?;
        if self.http.is_some() {
            // mock-first 结构化路径：经 HttpDriver POST 取消；非真实 OKX 协议
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let path =
                format!("/api/v5/trade/cancel-order?instId={}&ordId={}", request.instrument, id);
            let resp = self.http_post(&path, Bytes::new()).await?;
            if resp.status == 200 {
                return Ok(());
            }
            return Err(XError::unavailable(format!(
                "cancel_order_request http status {}",
                resp.status
            )));
        }
        Ok(())
    }

    async fn query_order_request(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        self.require_connected()?;
        if self.http.is_some() {
            // mock-first 结构化路径：经 HttpDriver GET 查询
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let path = format!("/api/v5/trade/order?instId={}&ordId={}", request.instrument, id);
            let resp = self.http_get(&path).await?;
            if resp.status != 200 {
                return Err(XError::unavailable(format!(
                    "query_order_request http status {}",
                    resp.status
                )));
            }
            let body = String::from_utf8_lossy(&resp.body);
            if body.contains("Cancelled")
                || body.contains("Canceled")
                || body.contains("canceled")
                || body.contains("CANCELED")
            {
                return Ok(OrderStatus::Cancelled);
            }
            if body.contains("Filled") || body.contains("filled") || body.contains("FILLED") {
                return Ok(OrderStatus::Filled);
            }
            return Ok(OrderStatus::Open);
        }
        Ok(OrderStatus::Open)
    }

    async fn query_position(&self) -> XResult<Vec<Position>> {
        self.require_connected()?;
        Ok(Vec::new())
    }

    async fn query_balance(&self) -> XResult<Vec<Money>> {
        self.require_connected()?;
        let ccy = Currency::try_new(*b"USD").map_err(|e| XError::internal(format!("ccy: {e}")))?;
        let amt = Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("amt: {e}")))?;
        Ok(vec![Money::try_new(amt, ccy).map_err(|e| XError::internal(format!("money: {e}")))?])
    }

    async fn subscribe_ticks(&self, _symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn subscribe_orderbook(
        &self,
        _symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        self.require_connected()?;
        Ok(Box::pin(stream::empty()))
    }

    async fn server_time(&self) -> XResult<i64> {
        self.require_connected()?;
        if self.http.is_some() {
            let resp = self.http_get("/api/v5/public/time").await?;
            if resp.status == 200 {
                if resp.body.is_empty() {
                    return Ok(0);
                }
                return parse_okx_server_time(&resp.body);
            }
            return Err(XError::unavailable(format!("server_time http status {}", resp.status)));
        }
        Ok(0)
    }

    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        self.require_connected()?;
        Ok(SymbolMeta {
            symbol: symbol.to_string(),
            base: "BASE".into(),
            quote: "QUOTE".into(),
            tick_size: Decimal::try_new(1, 2)
                .map_err(|e| XError::internal(format!("tick: {e}")))?,
            min_qty: Self::zero_qty()?,
        })
    }

    fn venue_id(&self) -> &'static str {
        "okx"
    }
}

#[async_trait]
impl MarketDataSource for OkxAdapter {
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        VenueAdapter::subscribe_ticks(self, symbol).await
    }
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        VenueAdapter::subscribe_orderbook(self, symbol).await
    }
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        VenueAdapter::subscribe_trades(self, symbol).await
    }
}

#[async_trait]
impl InstrumentCatalog for OkxAdapter {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        VenueAdapter::symbol_info(self, symbol).await
    }
}

#[async_trait]
impl ExecutionVenue for OkxAdapter {
    async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
        VenueAdapter::place_order(self, order).await
    }
    async fn cancel_order(&self, request: &CancelOrderRequest) -> XResult<()> {
        self.cancel_order_request(request).await
    }
    async fn query_order(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        self.query_order_request(request).await
    }
    fn venue_id(&self) -> VenueId {
        "okx".to_string()
    }
}

#[async_trait]
impl AccountSource for OkxAdapter {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        VenueAdapter::query_position(self).await
    }
    async fn query_balance(&self) -> XResult<Vec<Money>> {
        VenueAdapter::query_balance(self).await
    }
}

#[async_trait]
impl VenueTimeSource for OkxAdapter {
    async fn server_time(&self) -> XResult<i64> {
        VenueAdapter::server_time(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::{OrderRef, Side};
    use decimalx::{Price, Qty};

    fn sample_order() -> Order {
        Order {
            id: "o1".into(),
            symbol: "BTC-USDT".into(),
            side: Side::Buy,
            price: Price::new(Decimal::try_new(1, 0).unwrap()),
            qty: Qty::new(Decimal::try_new(1, 0).unwrap()),
            status: OrderStatus::Pending,
        }
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        VenueAdapter::disconnect(&a).await.unwrap();
    }

    #[tokio::test]
    async fn double_connect_fails() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert!(VenueAdapter::connect(&a).await.is_err());
    }

    #[tokio::test]
    async fn place_order_flow() {
        let a = OkxAdapter::demo();
        assert!(VenueAdapter::place_order(&a, &sample_order()).await.is_err());
        VenueAdapter::connect(&a).await.unwrap();
        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
    }

    #[tokio::test]
    async fn cancel_query_request() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Open);
    }

    #[tokio::test]
    async fn capability_traits() {
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert_eq!(VenueAdapter::venue_id(&a), "okx");
        assert_eq!(ExecutionVenue::venue_id(&a), "okx");
        let _ = VenueAdapter::server_time(&a).await.unwrap();
        assert!(VenueAdapter::query_position(&a).await.unwrap().is_empty());
    }

    #[test]
    fn parse_okx_server_time_ok() {
        let body = br#"{"code":"0","data":[{"ts":"1710000000456"}]}"#;
        assert_eq!(parse_okx_server_time(body).unwrap(), 1710000000456);
        assert!(parse_okx_server_time(br"{}").is_err());
    }

    #[tokio::test]
    async fn http_driver_get_and_server_time() {
        use bytes::Bytes;
        use transportx::MockHttpTransport;

        let mock = Arc::new(MockHttpTransport::new());
        let url = "https://www.okx.com/api/v5/public/time";
        mock.set_get(url, Bytes::from_static(br#"{"code":"0","data":[{"ts":"1710000000888"}]}"#));
        let a = OkxAdapter::mainnet().with_http(mock);
        assert!(a.has_http());
        VenueAdapter::connect(&a).await.unwrap();
        let resp = a.http_get("/api/v5/public/time").await.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(VenueAdapter::server_time(&a).await.unwrap(), 1710000000888);
    }

    #[tokio::test]
    async fn http_get_without_driver() {
        let a = OkxAdapter::demo();
        assert!(!a.has_http());
        assert_eq!(a.http_get("/x").await.expect_err("e").kind(), kernel::ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn http_cancel_and_query_order_request_paths() {
        use transportx::MockHttpTransport;

        let mock = Arc::new(MockHttpTransport::new());
        let cancel_url = "https://www.okx.com/api/v5/trade/cancel-order?instId=BTC-USDT&ordId=e1";
        let query_url = "https://www.okx.com/api/v5/trade/order?instId=BTC-USDT&ordId=e1";
        mock.set_post(cancel_url, Bytes::from_static(b"{\"code\":\"0\"}"));
        mock.set_get(query_url, Bytes::from_static(b"{\"state\":\"canceled\"}"));

        let a = OkxAdapter::mainnet().with_http(mock);
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn http_query_order_request_missing_mock_errors() {
        use transportx::MockHttpTransport;
        let mock = Arc::new(MockHttpTransport::new());
        let a = OkxAdapter::mainnet().with_http(mock);
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Exchange("missing".into()),
        };
        let e = a.query_order_request(&req).await.expect_err("missing");
        assert_eq!(e.kind(), kernel::ErrorKind::Invalid);
    }
}
