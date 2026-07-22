//! OKX `VenueAdapter` 生产默认路径（可选注入 `HttpDriver` / `OkxApiKey` / `WsConnector`）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderRef, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId, ns_from_unix_millis,
};
use contracts::{
    AccountSource, ExecutionVenue, InstrumentCatalog, MarketDataSource, VenueAdapter,
    VenueTimeSource,
};
use decimalx::{Currency, Decimal, Qty};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{ErrorKind, XError, XResult};
#[cfg(test)]
use transportx::WsConnection;
use transportx::{HttpDriver, HttpRequest, HttpResponse, TransportError, WsConnector};

use crate::auth::OkxApiKey;
use crate::market::{
    okx_public_ws_url, okx_subscribe_message, parse_okx_orderbook, parse_okx_ticker,
    parse_okx_trade,
};
use crate::response::{OkxError, OkxInstrumentData, OkxOrderData, OkxResponse};

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

fn check_okx_envelope(body: &[u8]) -> XResult<()> {
    // 优先识别错误信封
    if let Ok(err) = serde_json::from_slice::<OkxError>(body) {
        if err.code != "0" && !err.code.is_empty() {
            let kind = err.to_error_kind();
            let msg = format!("okx error {}: {}", err.code, err.msg);
            return Err(match kind {
                ErrorKind::Transient => XError::transient(msg),
                ErrorKind::Missing => XError::missing(msg),
                ErrorKind::Invalid => XError::invalid(msg),
                _ => XError::invalid(msg),
            });
        }
    }
    Ok(())
}

fn map_okx_state(state: &str) -> OrderStatus {
    match state {
        "live" => OrderStatus::Open,
        "partially_filled" => OrderStatus::PartiallyFilled,
        "filled" => OrderStatus::Filled,
        "canceled" | "cancelled" | "mmp_canceled" => OrderStatus::Cancelled,
        _ => OrderStatus::Open,
    }
}

/// 适配器连接状态（观测用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Disconnected,
    Connected,
}

/// OKX adapter 生产默认路径。
///
/// 默认无 HTTP / 凭证 / WS。注入后走真实 REST/WS 协议解析；
/// 未注入时明确内存占位 / 空流。
pub struct OkxAdapter {
    name: String,
    base_url: String,
    ws_base: String,
    connected: AtomicBool,
    http: Option<Arc<dyn HttpDriver>>,
    api_key: Option<OkxApiKey>,
    ws: Option<Arc<dyn WsConnector>>,
}

impl OkxAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            ws_base: "wss://ws.okx.com:8443/ws/v5/public".into(),
            connected: AtomicBool::new(false),
            http: None,
            api_key: None,
            ws: None,
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

    /// 注入 API 凭证，启用已认证端点。
    #[must_use]
    pub fn with_api_key(mut self, api_key: OkxApiKey) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 是否已配置 API 凭证。
    #[must_use]
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// 注入 WS 连接器，启用公共行情。
    #[must_use]
    pub fn with_ws(mut self, ws: Arc<dyn WsConnector>) -> Self {
        self.ws = Some(ws);
        self
    }

    /// 覆盖公共 WS 基址。
    #[must_use]
    pub fn with_ws_base(mut self, base: impl Into<String>) -> Self {
        self.ws_base = base.into();
        self
    }

    /// 是否已配置 WS。
    #[must_use]
    pub fn has_ws(&self) -> bool {
        self.ws.is_some()
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

    fn require_http(&self) -> XResult<&Arc<dyn HttpDriver>> {
        self.http.as_ref().ok_or_else(|| XError::unavailable("http driver not configured"))
    }

    fn require_api_key(&self) -> XResult<&OkxApiKey> {
        self.api_key.as_ref().ok_or_else(|| XError::unavailable("api key not configured"))
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
        let http = self.require_http()?;
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
        let http = self.require_http()?;
        let request = HttpRequest {
            method: "POST".into(),
            url: self.join_url(path),
            headers: vec![],
            body: Some(body),
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    /// 已签名请求：OKX 四头鉴权。`path` 含 query（如 `/api/v5/trade/order?instId=…`）。
    pub async fn http_signed(
        &self,
        method: &str,
        path: &str,
        body: Option<Bytes>,
    ) -> XResult<HttpResponse> {
        let http = self.require_http()?;
        let key = self.require_api_key()?;
        let body_str =
            body.as_ref().map(|b| String::from_utf8_lossy(b).into_owned()).unwrap_or_default();
        let mut headers = key.sign_headers(method, path, &body_str);
        if body.is_some() {
            headers.push(("Content-Type".into(), "application/json".into()));
        }
        let request =
            HttpRequest { method: method.into(), url: self.join_url(path), headers, body };
        http.execute(request).await.map_err(map_transport_error)
    }

    fn zero_qty() -> XResult<Qty> {
        Ok(Qty::new(
            Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero qty: {e}")))?,
        ))
    }

    fn parse_order_envelope(body: &[u8]) -> XResult<OkxOrderData> {
        check_okx_envelope(body)?;
        let env: OkxResponse<OkxOrderData> = serde_json::from_slice(body)
            .map_err(|e| XError::invalid(format!("okx order parse: {e}")))?;
        if env.code != "0" {
            let err = OkxError { code: env.code, msg: env.msg };
            let kind = err.to_error_kind();
            let msg = format!("okx error {}: {}", err.code, err.msg);
            return Err(match kind {
                ErrorKind::Transient => XError::transient(msg),
                ErrorKind::Missing => XError::missing(msg),
                _ => XError::invalid(msg),
            });
        }
        env.data.into_iter().next().ok_or_else(|| XError::invalid("okx order empty data"))
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

        if self.http.is_some() && self.api_key.is_some() {
            let side = match order.side {
                canonical::Side::Buy => "buy",
                canonical::Side::Sell => "sell",
            };
            let body = serde_json::json!({
                "instId": order.symbol,
                "tdMode": "cash",
                "side": side,
                "ordType": "limit",
                "sz": order.qty.as_decimal().to_string(),
                "px": order.price.as_decimal().to_string(),
                "clOrdId": order.id,
            })
            .to_string();
            let path = "/api/v5/trade/order";
            let resp = self.http_signed("POST", path, Some(Bytes::from(body))).await?;
            // place 返回 data[0] 常只有 ordId/clOrdId/sCode；状态取 live 或 sCode
            check_okx_envelope(&resp.body)?;
            let v: serde_json::Value = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("okx place parse: {e}")))?;
            let code = v.get("code").and_then(|c| c.as_str()).unwrap_or("1");
            if code != "0" {
                let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("place failed");
                return Err(XError::invalid(format!("okx error {code}: {msg}")));
            }
            let row = v
                .get("data")
                .and_then(|d| d.as_array())
                .and_then(|a| a.first())
                .ok_or_else(|| XError::invalid("okx place empty data"))?;
            // sCode 非 0 表示订单级失败
            if let Some(scode) = row.get("sCode").and_then(|x| x.as_str()) {
                if scode != "0" {
                    let smsg = row.get("sMsg").and_then(|x| x.as_str()).unwrap_or("order rejected");
                    return Err(XError::invalid(format!("okx sCode {scode}: {smsg}")));
                }
            }
            let id = row
                .get("clOrdId")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(order.id.as_str())
                .to_string();
            return Ok(OrderAck { id, status: OrderStatus::Open, ts: 0 });
        }

        // 无凭证：明确 mock，状态 Open（非 Filled）
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

        if self.http.is_some() && self.api_key.is_some() {
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let mut body = serde_json::Map::new();
            body.insert("instId".into(), serde_json::Value::String(request.instrument.clone()));
            match &request.id {
                OrderRef::Exchange(s) => {
                    body.insert("ordId".into(), serde_json::Value::String(s.clone()));
                }
                OrderRef::Client(s) => {
                    body.insert("clOrdId".into(), serde_json::Value::String(s.clone()));
                }
            }
            let _ = id;
            let body = serde_json::Value::Object(body).to_string();
            let path = "/api/v5/trade/cancel-order";
            let resp = self.http_signed("POST", path, Some(Bytes::from(body))).await?;
            check_okx_envelope(&resp.body)?;
            let v: serde_json::Value = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("okx cancel parse: {e}")))?;
            let code = v.get("code").and_then(|c| c.as_str()).unwrap_or("1");
            if code != "0" {
                let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("cancel failed");
                return Err(XError::invalid(format!("okx error {code}: {msg}")));
            }
            return Ok(());
        }

        // 无凭证但有 HTTP：兼容旧 mock 路径（非协议）
        if self.http.is_some() {
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

        if self.http.is_some() && self.api_key.is_some() {
            let path = match &request.id {
                OrderRef::Exchange(s) => {
                    format!("/api/v5/trade/order?instId={}&ordId={}", request.instrument, s)
                }
                OrderRef::Client(s) => {
                    format!("/api/v5/trade/order?instId={}&clOrdId={}", request.instrument, s)
                }
            };
            let resp = self.http_signed("GET", &path, None).await?;
            let order = Self::parse_order_envelope(&resp.body)?;
            return Ok(map_okx_state(&order.state));
        }

        // 无凭证 mock 路径
        if self.http.is_some() {
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
            // 尝试业务信封
            if let Ok(order) = Self::parse_order_envelope(&resp.body) {
                return Ok(map_okx_state(&order.state));
            }
            let body = String::from_utf8_lossy(&resp.body);
            if body.contains("canceled") || body.contains("Cancelled") || body.contains("CANCELED")
            {
                return Ok(OrderStatus::Cancelled);
            }
            if body.contains("filled") || body.contains("Filled") || body.contains("FILLED") {
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

        if self.http.is_some() && self.api_key.is_some() {
            let path = "/api/v5/account/balance";
            let resp = self.http_signed("GET", path, None).await?;
            check_okx_envelope(&resp.body)?;
            let v: serde_json::Value = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("okx balance parse: {e}")))?;
            if v.get("code").and_then(|c| c.as_str()) != Some("0") {
                return Err(XError::invalid("okx balance code != 0"));
            }
            let mut out = Vec::new();
            if let Some(details) = v
                .get("data")
                .and_then(|d| d.as_array())
                .and_then(|a| a.first())
                .and_then(|r| r.get("details"))
                .and_then(|d| d.as_array())
            {
                for d in details {
                    let ccy = d.get("ccy").and_then(|x| x.as_str()).unwrap_or("");
                    let eq = d.get("eq").and_then(|x| x.as_str()).unwrap_or("0");
                    if eq == "0" || eq == "0.0" || ccy.is_empty() {
                        continue;
                    }
                    let amt: Decimal = match eq.parse() {
                        Ok(a) => a,
                        Err(_) => continue,
                    };
                    let mut buf = [0u8; 3];
                    let bs = ccy.as_bytes();
                    let len = bs.len().min(3);
                    buf[..len].copy_from_slice(&bs[..len]);
                    let currency = match Currency::try_new(buf) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    if let Ok(m) = Money::try_new(amt, currency) {
                        out.push(m);
                    }
                }
            }
            return Ok(out);
        }

        let ccy = Currency::try_new(*b"USD").map_err(|e| XError::internal(format!("ccy: {e}")))?;
        let amt = Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("amt: {e}")))?;
        Ok(vec![Money::try_new(amt, ccy).map_err(|e| XError::internal(format!("money: {e}")))?])
    }

    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        self.require_connected()?;
        let Some(ws) = self.ws.clone() else {
            return Ok(Box::pin(stream::empty()));
        };
        let url = okx_public_ws_url(&self.ws_base);
        let mut conn = ws.connect(&url).await.map_err(map_transport_error)?;
        let sub = okx_subscribe_message("tickers", symbol);
        transportx::WsConnection::send_frame(conn.as_mut(), Bytes::from(sub))
            .await
            .map_err(map_transport_error)?;
        let s = stream::unfold(conn, |mut conn| async move {
            loop {
                match transportx::WsConnection::next_frame(conn.as_mut()).await {
                    Ok(Some(bytes)) => {
                        if let Ok(tick) = parse_okx_ticker(&bytes) {
                            return Some((tick, conn));
                        }
                    }
                    Ok(None) | Err(_) => return None,
                }
            }
        });
        Ok(Box::pin(s))
    }

    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        self.require_connected()?;
        let Some(ws) = self.ws.clone() else {
            return Ok(Box::pin(stream::empty()));
        };
        let url = okx_public_ws_url(&self.ws_base);
        let mut conn = ws.connect(&url).await.map_err(map_transport_error)?;
        let sub = okx_subscribe_message("books5", symbol);
        transportx::WsConnection::send_frame(conn.as_mut(), Bytes::from(sub))
            .await
            .map_err(map_transport_error)?;
        let sym = symbol.to_string();
        let s = stream::unfold((conn, sym), |(mut conn, sym)| async move {
            loop {
                match transportx::WsConnection::next_frame(conn.as_mut()).await {
                    Ok(Some(bytes)) => {
                        if let Ok(book) = parse_okx_orderbook(&bytes, &sym) {
                            return Some((book, (conn, sym)));
                        }
                    }
                    Ok(None) | Err(_) => return None,
                }
            }
        });
        Ok(Box::pin(s))
    }

    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        self.require_connected()?;
        let Some(ws) = self.ws.clone() else {
            return Ok(Box::pin(stream::empty()));
        };
        let url = okx_public_ws_url(&self.ws_base);
        let mut conn = ws.connect(&url).await.map_err(map_transport_error)?;
        let sub = okx_subscribe_message("trades", symbol);
        transportx::WsConnection::send_frame(conn.as_mut(), Bytes::from(sub))
            .await
            .map_err(map_transport_error)?;
        let s = stream::unfold(conn, |mut conn| async move {
            loop {
                match transportx::WsConnection::next_frame(conn.as_mut()).await {
                    Ok(Some(bytes)) => {
                        if let Ok(trade) = parse_okx_trade(&bytes) {
                            return Some((trade, conn));
                        }
                    }
                    Ok(None) | Err(_) => return None,
                }
            }
        });
        Ok(Box::pin(s))
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

        if self.http.is_some() {
            let path = format!("/api/v5/public/instruments?instType=SPOT&instId={symbol}");
            let resp = self.http_get(&path).await?;
            if resp.status == 200 {
                check_okx_envelope(&resp.body)?;
                let env: OkxResponse<OkxInstrumentData> = serde_json::from_slice(&resp.body)
                    .map_err(|e| XError::invalid(format!("instruments parse: {e}")))?;
                if env.code != "0" {
                    return Err(XError::invalid(format!("okx instruments code {}", env.code)));
                }
                if let Some(inst) = env.data.into_iter().next() {
                    let tick = inst
                        .tick_sz
                        .parse::<Decimal>()
                        .unwrap_or_else(|_| Decimal::try_new(1, 2).expect("tick"));
                    let min = inst
                        .min_sz
                        .parse::<Decimal>()
                        .map(Qty::new)
                        .unwrap_or_else(|_| Self::zero_qty().expect("qty"));
                    return Ok(SymbolMeta {
                        symbol: inst.inst_id,
                        base: inst.base_ccy,
                        quote: inst.quote_ccy,
                        tick_size: tick,
                        min_qty: min,
                    });
                }
                return Err(XError::missing(format!("symbol not found: {symbol}")));
            }
            return Err(XError::unavailable(format!("symbol_info http status {}", resp.status)));
        }

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

// silence unused import if ns helper reserved for future uTime mapping
#[allow(dead_code)]
fn _ns(ms: i64) -> Option<i64> {
    ns_from_unix_millis(ms)
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
    async fn place_order_flow_mock() {
        let a = OkxAdapter::demo();
        assert!(VenueAdapter::place_order(&a, &sample_order()).await.is_err());
        VenueAdapter::connect(&a).await.unwrap();
        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
        assert_eq!(ack.status, OrderStatus::Open);
        assert_ne!(ack.status, OrderStatus::Filled);
    }

    #[tokio::test]
    async fn cancel_query_request_mock() {
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
        mock.set_get(
            query_url,
            Bytes::from_static(
                br#"{"code":"0","msg":"","data":[{"ordId":"e1","clOrdId":"","instId":"BTC-USDT","side":"buy","ordType":"limit","px":"1","sz":"1","state":"canceled"}]}"#,
            ),
        );

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

    struct RecordingHttp {
        responses: std::sync::Mutex<Vec<(String, String, Bytes)>>,
        captured: std::sync::Mutex<Vec<HttpRequest>>,
    }

    impl RecordingHttp {
        fn new() -> Self {
            Self {
                responses: std::sync::Mutex::new(Vec::new()),
                captured: std::sync::Mutex::new(Vec::new()),
            }
        }
        fn push(&self, method: &str, path_prefix: &str, body: Bytes) {
            self.responses.lock().expect("lock").push((method.into(), path_prefix.into(), body));
        }
        fn last(&self) -> HttpRequest {
            self.captured.lock().expect("lock").last().cloned().expect("cap")
        }
    }

    #[async_trait]
    impl HttpDriver for RecordingHttp {
        async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
            self.captured.lock().expect("lock").push(request.clone());
            let method = request.method.to_ascii_uppercase();
            let path = request.url.split('?').next().unwrap_or(&request.url).to_string();
            for (m, prefix, body) in self.responses.lock().expect("lock").iter() {
                if m.eq_ignore_ascii_case(&method) && path.contains(prefix.as_str()) {
                    return Ok(HttpResponse { status: 200, body: body.clone() });
                }
            }
            Err(TransportError::ProtocolViolation(format!("miss {method} {}", request.url)))
        }
    }

    #[tokio::test]
    async fn signed_place_cancel_query_protocol() {
        let http = Arc::new(RecordingHttp::new());
        http.push(
            "POST",
            "/api/v5/trade/order",
            Bytes::from_static(
                br#"{"code":"0","msg":"","data":[{"clOrdId":"o1","ordId":"99","sCode":"0","sMsg":""}]}"#,
            ),
        );
        http.push(
            "POST",
            "/api/v5/trade/cancel-order",
            Bytes::from_static(
                br#"{"code":"0","msg":"","data":[{"clOrdId":"o1","ordId":"99","sCode":"0","sMsg":""}]}"#,
            ),
        );
        http.push(
            "GET",
            "/api/v5/trade/order",
            Bytes::from_static(
                br#"{"code":"0","msg":"","data":[{"ordId":"99","clOrdId":"o1","instId":"BTC-USDT","side":"buy","ordType":"limit","px":"1","sz":"1","state":"filled"}]}"#,
            ),
        );

        let a = OkxAdapter::mainnet().with_http(http.clone()).with_api_key(OkxApiKey::new(
            "k",
            "secret-key-value",
            "p",
        ));
        VenueAdapter::connect(&a).await.unwrap();

        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
        assert_eq!(ack.status, OrderStatus::Open);
        let place = http.last();
        assert_eq!(place.method.to_ascii_uppercase(), "POST");
        assert!(place.url.ends_with("/api/v5/trade/order"));
        let keys: Vec<_> = place.headers.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"OK-ACCESS-KEY"));
        assert!(keys.contains(&"OK-ACCESS-SIGN"));
        assert!(keys.contains(&"OK-ACCESS-TIMESTAMP"));
        assert!(keys.contains(&"OK-ACCESS-PASSPHRASE"));
        assert!(place.headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
        // secret 不得出现在任何 header 值中（passphrase 按协议会出现）
        for (k, v) in &place.headers {
            assert_ne!(v, "secret-key-value", "secret leaked in {k}");
            if k == "OK-ACCESS-SIGN" {
                assert_ne!(v, "secret-key-value");
                assert!(!v.is_empty());
            }
        }
        assert!(
            !format!("{:?}", OkxApiKey::new("k", "secret-key-value", "p"))
                .contains("secret-key-value")
        );

        let req = CancelOrderRequest {
            venue: "okx".into(),
            instrument: "BTC-USDT".into(),
            id: OrderRef::Client("o1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        let cancel = http.last();
        assert_eq!(cancel.method.to_ascii_uppercase(), "POST");
        assert!(cancel.url.contains("cancel-order"));

        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Filled);
        let query = http.last();
        assert_eq!(query.method.to_ascii_uppercase(), "GET");
        assert!(query.url.contains("clOrdId=o1"));
    }

    #[tokio::test]
    async fn signed_error_code_maps_invalid() {
        let http = Arc::new(RecordingHttp::new());
        http.push(
            "POST",
            "/api/v5/trade/order",
            Bytes::from_static(br#"{"code":"51000","msg":"Parameter error","data":[]}"#),
        );
        let a = OkxAdapter::mainnet().with_http(http).with_api_key(OkxApiKey::new("k", "s", "p"));
        VenueAdapter::connect(&a).await.unwrap();
        let e = VenueAdapter::place_order(&a, &sample_order()).await.expect_err("e");
        assert_eq!(e.kind(), kernel::ErrorKind::Invalid);
    }

    struct FixtureWs {
        frames: std::sync::Mutex<Vec<Bytes>>,
    }
    struct FixtureConn {
        frames: Vec<Bytes>,
        idx: usize,
    }

    #[async_trait]
    impl WsConnection for FixtureConn {
        async fn next_frame(&mut self) -> Result<Option<Bytes>, TransportError> {
            if self.idx >= self.frames.len() {
                return Ok(None);
            }
            let f = self.frames[self.idx].clone();
            self.idx += 1;
            Ok(Some(f))
        }
        async fn send_frame(&mut self, _frame: Bytes) -> Result<(), TransportError> {
            Ok(())
        }
        async fn close(&mut self) -> Result<(), TransportError> {
            Ok(())
        }
    }

    #[async_trait]
    impl WsConnector for FixtureWs {
        async fn connect(&self, _url: &str) -> Result<Box<dyn WsConnection>, TransportError> {
            let frames = self.frames.lock().expect("lock").clone();
            Ok(Box::new(FixtureConn { frames, idx: 0 }))
        }
    }

    #[tokio::test]
    async fn subscribe_ticks_parses_okx_ticker() {
        use futures_util::StreamExt;
        let frame = Bytes::from_static(
            br#"{"arg":{"channel":"tickers","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","bidPx":"100","askPx":"101","ts":"1700000000123"}]}"#,
        );
        let ws = Arc::new(FixtureWs { frames: std::sync::Mutex::new(vec![frame]) });
        let a = OkxAdapter::mainnet().with_ws(ws);
        VenueAdapter::connect(&a).await.unwrap();
        let mut s = VenueAdapter::subscribe_ticks(&a, "BTC-USDT").await.unwrap();
        let tick = s.next().await.expect("tick");
        assert_eq!(tick.symbol, "BTC-USDT");
        assert_eq!(tick.bid.as_decimal(), "100".parse().unwrap());
        assert_eq!(tick.ts, 1_700_000_000_123 * 1_000_000);
    }

    #[tokio::test]
    async fn subscribe_without_ws_empty() {
        use futures_util::StreamExt;
        let a = OkxAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let mut s = VenueAdapter::subscribe_trades(&a, "BTC-USDT").await.unwrap();
        assert!(s.next().await.is_none());
    }

    #[test]
    fn map_okx_state_variants() {
        assert_eq!(map_okx_state("live"), OrderStatus::Open);
        assert_eq!(map_okx_state("filled"), OrderStatus::Filled);
        assert_eq!(map_okx_state("canceled"), OrderStatus::Cancelled);
        assert_eq!(map_okx_state("partially_filled"), OrderStatus::PartiallyFilled);
    }

    #[test]
    fn api_key_builder() {
        let a = OkxAdapter::mainnet().with_api_key(OkxApiKey::new("k", "s", "p"));
        assert!(a.has_api_key());
        assert!(!OkxAdapter::mainnet().has_api_key());
    }
}
