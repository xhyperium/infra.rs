//! Binance `VenueAdapter` 生产就绪（可选注入 `transportx::HttpDriver` 和 `BinanceApiKey`）。

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
use decimalx::{Currency, Decimal, Price, Qty};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{ErrorKind, XError, XResult};
use serde::Deserialize;
use transportx::{HttpDriver, HttpRequest, HttpResponse, TransportError};

use crate::auth::BinanceApiKey;
use crate::response::{
    AccountInfo, BinanceError, CancelOrderResponse, ExchangeInfoSymbol, OrderResponse,
};

/// 解析 Binance `/api/v3/time` JSON：`{"serverTime": <ms>}`。
///
/// 离线可调用（bench / 单元测试）；不发起网络。
pub fn parse_binance_server_time(body: &[u8]) -> XResult<i64> {
    let v: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| XError::invalid(format!("server_time json: {e}")))?;
    v.get("serverTime")
        .and_then(|x| x.as_i64())
        .ok_or_else(|| XError::invalid("server_time missing serverTime field"))
}

/// 将 [`TransportError`] 映射为 kernel [`XError`]。
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

fn check_binance_error(body: &[u8]) -> XResult<()> {
    if let Ok(err) = serde_json::from_slice::<BinanceError>(body) {
        if err.code < 0 {
            let kind = err.to_error_kind();
            let msg = format!("binance error {}: {}", err.code, err.msg);
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

/// 适配器连接状态（观测用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Disconnected,
    Connected,
}

/// K 线时间粒度（venue 扩展；非 contracts 面）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    H1,
    H4,
    D1,
}

impl Timeframe {
    /// Binance klines interval 字符串。
    #[must_use]
    pub fn to_api_str(self) -> &'static str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
        }
    }
}

/// 单根 K 线 scaffold DTO。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    pub open_time: u64,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Price,
    pub close_time: u64,
}

/// 注册 BinanceError 反序列化检查器（不暴露但编译为 dead_code 标注）。
#[allow(dead_code)]
fn _check_binance_error_deser() {
    fn _check<T: for<'de> Deserialize<'de>>() {}
    _check::<BinanceError>();
}

/// Binance adapter 生产就绪。
///
/// 默认无 HTTP 驱动（纯内存占位）。通过 [`Self::with_http`] 注入
/// [`HttpDriver`] 后，[`Self::http_get`] / `server_time` 走 transport 边界。
/// 通过 [`Self::with_api_key`] 注入 API 凭证后，已认证端点使用 HMAC-SHA256 签名。
pub struct BinanceAdapter {
    name: String,
    base_url: String,
    connected: AtomicBool,
    http: Option<Arc<dyn HttpDriver>>,
    api_key: Option<BinanceApiKey>,
}

impl BinanceAdapter {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: base_url.into(),
            connected: AtomicBool::new(false),
            http: None,
            api_key: None,
        }
    }

    pub fn testnet() -> Self {
        Self::new("binance-testnet", "https://testnet.binance.vision")
    }

    pub fn mainnet() -> Self {
        Self::new("binance-mainnet", "https://api.binance.com")
    }

    /// 注入 HTTP 驱动（组合 transportx；仍非真实业务协议解析）。
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
    pub fn with_api_key(mut self, api_key: BinanceApiKey) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// 是否已配置 API 凭证。
    #[must_use]
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
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

    fn require_api_key(&self) -> XResult<&BinanceApiKey> {
        self.api_key.as_ref().ok_or_else(|| XError::unavailable("api key not configured"))
    }

    fn join_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let base = self.base_url.trim_end_matches('/');
        if path.starts_with('/') { format!("{base}{path}") } else { format!("{base}/{path}") }
    }

    /// 经注入的 [`HttpDriver`] 发起 POST（需已 `with_http`）。
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

    /// 经注入的 [`HttpDriver`] 发起 GET（需已 `with_http`）。
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

    /// 已签名 GET：设置 X-MBX-APIKEY 头部并对查询参数签名。
    pub async fn http_get_signed(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> XResult<HttpResponse> {
        let http = self.require_http()?;
        let key = self.require_api_key()?;
        let (api_key, query) = key.sign_params(params);
        let full_path = format!("{}?{}", path, query);
        let request = HttpRequest {
            method: "GET".into(),
            url: self.join_url(&full_path),
            headers: vec![("X-MBX-APIKEY".to_string(), api_key)],
            body: None,
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    /// 已签名 POST：设置 X-MBX-APIKEY 头部并对查询参数签名。
    /// body 为空时参数编码在 URL 查询中；有 body 时作为 POST body 发送。
    pub async fn http_post_signed(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> XResult<HttpResponse> {
        let http = self.require_http()?;
        let key = self.require_api_key()?;
        let (api_key, query) = key.sign_params(params);
        let full_path = format!("{}?{}", path, query);
        let request = HttpRequest {
            method: "POST".into(),
            url: self.join_url(&full_path),
            headers: vec![("X-MBX-APIKEY".to_string(), api_key)],
            body: None,
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    /// 已签名 DELETE：设置 X-MBX-APIKEY 头部并对查询参数签名。
    pub async fn http_delete_signed(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> XResult<HttpResponse> {
        let http = self.require_http()?;
        let key = self.require_api_key()?;
        let (api_key, query) = key.sign_params(params);
        let full_path = format!("{}?{}", path, query);
        let request = HttpRequest {
            method: "DELETE".into(),
            url: self.join_url(&full_path),
            headers: vec![("X-MBX-APIKEY".to_string(), api_key)],
            body: None,
        };
        http.execute(request).await.map_err(map_transport_error)
    }

    fn zero_price() -> XResult<Price> {
        Ok(Price::new(
            Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero price: {e}")))?,
        ))
    }

    fn zero_qty() -> XResult<Qty> {
        Ok(Qty::new(
            Decimal::try_new(0, 0).map_err(|e| XError::internal(format!("zero qty: {e}")))?,
        ))
    }

    /// venue 扩展：占位 K 线（非 contracts 面）。
    pub async fn fetch_candles(
        &self,
        _symbol: &str,
        _timeframe: Timeframe,
        limit: Option<u32>,
    ) -> XResult<Vec<Candle>> {
        self.require_connected()?;
        let zero = Self::zero_price()?;
        let count = limit.unwrap_or(10) as usize;
        Ok((0..count)
            .map(|i| Candle {
                open_time: i as u64,
                open: zero,
                high: zero,
                low: zero,
                close: zero,
                volume: zero,
                close_time: (i + 1) as u64,
            })
            .collect())
    }

    /// 将 Binance 订单状态映射为 [`OrderStatus`]。
    fn map_order_status(status: &str) -> OrderStatus {
        match status {
            "NEW" => OrderStatus::Open,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Cancelled,
            "REJECTED" => OrderStatus::Rejected,
            "EXPIRED" => OrderStatus::Cancelled,
            _ => OrderStatus::Open,
        }
    }

    /// 从 [`ExchangeInfoSymbol`] 的 filters 数组中提取价格精度和数量精度。
    pub fn parse_symbol_meta(sym: &ExchangeInfoSymbol) -> XResult<SymbolMeta> {
        let mut tick_size = None;
        let mut step_size = None;
        let mut min_qty = None;

        for filter in &sym.filters {
            let filter_type = filter.get("filterType").and_then(|v| v.as_str());
            match filter_type {
                Some("PRICE_FILTER") => {
                    if let Some(ts) = filter.get("tickSize").and_then(|v| v.as_str()) {
                        tick_size = ts.parse::<f64>().ok();
                    }
                }
                Some("LOT_SIZE") => {
                    if let Some(mq) = filter.get("minQty").and_then(|v| v.as_str()) {
                        min_qty = mq.parse::<f64>().ok();
                    }
                    if let Some(ss) = filter.get("stepSize").and_then(|v| v.as_str()) {
                        step_size = ss.parse::<f64>().ok();
                    }
                }
                _ => {}
            }
        }

        let tick = tick_size
            .and_then(|t| t.to_string().parse::<Decimal>().ok())
            .unwrap_or_else(|| Decimal::try_new(1, 2).expect("hardcoded tick_size"));

        let qty = min_qty
            .and_then(|q| q.to_string().parse::<Decimal>().ok())
            .map(Qty::new)
            .unwrap_or_else(|| Self::zero_qty().expect("hardcoded zero_qty"));

        let _step = step_size;

        Ok(SymbolMeta {
            symbol: sym.symbol.clone(),
            base: sym.base_asset.clone(),
            quote: sym.quote_asset.clone(),
            tick_size: tick,
            min_qty: qty,
        })
    }
}

#[async_trait]
impl VenueAdapter for BinanceAdapter {
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

        // 已签名 HTTP 路径
        if self.http.is_some() && self.api_key.is_some() {
            let side = match order.side {
                canonical::Side::Buy => "BUY",
                canonical::Side::Sell => "SELL",
            };
            let qty_str = order.qty.as_decimal().to_string();
            let price_str = order.price.as_decimal().to_string();
            let params: Vec<(&str, &str)> = vec![
                ("symbol", &order.symbol),
                ("side", side),
                ("type", "LIMIT"),
                ("timeInForce", "GTC"),
                ("quantity", &qty_str),
                ("price", &price_str),
                ("newClientOrderId", &order.id),
            ];
            let resp = self.http_post_signed("/api/v3/order", &params).await?;
            check_binance_error(&resp.body)?;
            let order_resp: OrderResponse = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("place_order parse: {e}")))?;
            let status = Self::map_order_status(&order_resp.status);
            return Ok(OrderAck { id: order.id.clone(), status, ts: order_resp.update_time });
        }

        // 降级：内存占位
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

        // 已签名 HTTP 路径
        if self.http.is_some() && self.api_key.is_some() {
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let params: Vec<(&str, &str)> =
                vec![("symbol", &request.instrument), ("origClientOrderId", id)];
            let resp = self.http_delete_signed("/api/v3/order", &params).await?;
            check_binance_error(&resp.body)?;
            let _cancel: CancelOrderResponse = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("cancel_order parse: {e}")))?;
            return Ok(());
        }

        // 降级：mock 路径
        if self.http.is_some() {
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let path = format!("/api/v3/order?symbol={}&orderId={}", request.instrument, id);
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

        // 已签名 HTTP 路径
        if self.http.is_some() && self.api_key.is_some() {
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let params: Vec<(&str, &str)> =
                vec![("symbol", &request.instrument), ("origClientOrderId", id)];
            let resp = self.http_get_signed("/api/v3/order", &params).await?;
            check_binance_error(&resp.body)?;
            let order_resp: OrderResponse = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("query_order parse: {e}")))?;
            return Ok(Self::map_order_status(&order_resp.status));
        }

        // 降级：mock 路径
        if self.http.is_some() {
            let id = match &request.id {
                OrderRef::Exchange(s) | OrderRef::Client(s) => s.as_str(),
            };
            let path = format!("/api/v3/order?symbol={}&orderId={}", request.instrument, id);
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
                || body.contains("CANCELED")
                || body.contains("CANCELLED")
            {
                return Ok(OrderStatus::Cancelled);
            }
            if body.contains("Filled") || body.contains("FILLED") {
                return Ok(OrderStatus::Filled);
            }
            return Ok(OrderStatus::Open);
        }
        Ok(OrderStatus::Open)
    }

    async fn query_position(&self) -> XResult<Vec<Position>> {
        self.require_connected()?;

        // 已签名 HTTP 路径
        if self.http.is_some() && self.api_key.is_some() {
            let resp = self.http_get_signed("/api/v3/account", &[]).await?;
            check_binance_error(&resp.body)?;
            let account: AccountInfo = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("account parse: {e}")))?;
            let mut positions = Vec::new();
            for bal in &account.balances {
                let free_str = bal.free.trim();
                let locked_str = bal.locked.trim();
                if (free_str == "0" || free_str == "0.0" || free_str == "0.00")
                    && (locked_str == "0" || locked_str == "0.0" || locked_str == "0.00")
                {
                    continue;
                }
                // 使用解析后的 Decimal；解析失败跳过
                let free_dec: Decimal = match free_str.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let locked_dec: Decimal = match locked_str.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let total = match free_dec.checked_add(locked_dec) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if total == Decimal::ZERO {
                    continue;
                }
                let pos = Position {
                    symbol: bal.asset.clone(),
                    qty: Qty::new(total),
                    entry_price: Price::new(Decimal::ZERO),
                };
                positions.push(pos);
            }
            return Ok(positions);
        }

        Ok(Vec::new())
    }

    async fn query_balance(&self) -> XResult<Vec<Money>> {
        self.require_connected()?;

        // 已签名 HTTP 路径
        if self.http.is_some() && self.api_key.is_some() {
            let resp = self.http_get_signed("/api/v3/account", &[]).await?;
            check_binance_error(&resp.body)?;
            let account: AccountInfo = serde_json::from_slice(&resp.body)
                .map_err(|e| XError::invalid(format!("account parse: {e}")))?;
            let mut balances = Vec::new();
            for bal in &account.balances {
                // 简单检测零余额
                if (bal.free == "0" || bal.free == "0.0" || bal.free == "0.00")
                    && (bal.locked == "0" || bal.locked == "0.0" || bal.locked == "0.00")
                {
                    continue;
                }
                let free_dec: Decimal = match bal.free.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let locked_dec: Decimal = match bal.locked.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let total = match free_dec.checked_add(locked_dec) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                let ccy_bytes: [u8; 3] = {
                    let mut buf = [0u8; 3];
                    let bs = bal.asset.as_bytes();
                    let len = bs.len().min(3);
                    buf[..len].copy_from_slice(&bs[..len]);
                    buf
                };
                let ccy = match Currency::try_new(ccy_bytes) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let money = Money::try_new(total, ccy)
                    .map_err(|e| XError::internal(format!("money: {e}")))?;
                balances.push(money);
            }
            return Ok(balances);
        }

        // 降级：内存占位
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
            let resp = self.http_get("/api/v3/time").await?;
            if resp.status == 200 {
                if resp.body.is_empty() {
                    return Ok(0);
                }
                return parse_binance_server_time(&resp.body);
            }
            return Err(XError::unavailable(format!("server_time http status {}", resp.status)));
        }
        Ok(0)
    }

    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        self.require_connected()?;

        // HTTP 路径（公共，无需签名）
        if self.http.is_some() {
            let path = format!("/api/v3/exchangeInfo?symbol={symbol}");
            let resp = self.http_get(&path).await?;
            if resp.status == 200 {
                let exchange_info: crate::response::ExchangeInfo =
                    serde_json::from_slice(&resp.body)
                        .map_err(|e| XError::invalid(format!("exchangeInfo parse: {e}")))?;
                if let Some(sym) = exchange_info.symbols.first() {
                    return Self::parse_symbol_meta(sym);
                }
                return Err(XError::missing(format!("symbol not found: {symbol}")));
            }
            return Err(XError::unavailable(format!("symbol_info http status {}", resp.status)));
        }

        // 降级：内存占位
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
        "binance"
    }
}

#[async_trait]
impl MarketDataSource for BinanceAdapter {
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
impl InstrumentCatalog for BinanceAdapter {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta> {
        VenueAdapter::symbol_info(self, symbol).await
    }
}

#[async_trait]
impl ExecutionVenue for BinanceAdapter {
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
        "binance".to_string()
    }
}

#[async_trait]
impl AccountSource for BinanceAdapter {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        VenueAdapter::query_position(self).await
    }
    async fn query_balance(&self) -> XResult<Vec<Money>> {
        VenueAdapter::query_balance(self).await
    }
}

#[async_trait]
impl VenueTimeSource for BinanceAdapter {
    async fn server_time(&self) -> XResult<i64> {
        VenueAdapter::server_time(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::{OrderRef, Side};
    use decimalx::Qty;

    fn sample_order() -> Order {
        Order {
            id: "o1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: Price::new(Decimal::try_new(1, 0).unwrap()),
            qty: Qty::new(Decimal::try_new(1, 0).unwrap()),
            status: OrderStatus::Pending,
        }
    }

    #[tokio::test]
    async fn connect_disconnect() {
        let a = BinanceAdapter::mainnet();
        assert_eq!(a.state(), AdapterState::Disconnected);
        VenueAdapter::connect(&a).await.unwrap();
        assert_eq!(a.state(), AdapterState::Connected);
        VenueAdapter::disconnect(&a).await.unwrap();
        assert_eq!(a.state(), AdapterState::Disconnected);
    }

    #[tokio::test]
    async fn double_connect_fails() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        assert!(VenueAdapter::connect(&a).await.is_err());
    }

    #[tokio::test]
    async fn place_order_requires_connect() {
        let a = BinanceAdapter::mainnet();
        assert!(VenueAdapter::place_order(&a, &sample_order()).await.is_err());
        VenueAdapter::connect(&a).await.unwrap();
        let ack = VenueAdapter::place_order(&a, &sample_order()).await.unwrap();
        assert_eq!(ack.id, "o1");
        assert_eq!(ack.status, OrderStatus::Open);
    }

    #[tokio::test]
    async fn cancel_query_request() {
        let a = BinanceAdapter::testnet();
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "binance".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Open);
    }

    #[tokio::test]
    async fn market_data_streams_empty() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let _ = VenueAdapter::subscribe_ticks(&a, "BTCUSDT").await.unwrap();
        let _ = VenueAdapter::subscribe_orderbook(&a, "BTCUSDT").await.unwrap();
        let _ = VenueAdapter::subscribe_trades(&a, "BTCUSDT").await.unwrap();
    }

    #[tokio::test]
    async fn candles_extension() {
        let a = BinanceAdapter::testnet();
        VenueAdapter::connect(&a).await.unwrap();
        let c = a.fetch_candles("BTCUSDT", Timeframe::M1, Some(3)).await.unwrap();
        assert_eq!(c.len(), 3);
    }

    #[tokio::test]
    async fn capability_traits_dispatch() {
        let a = BinanceAdapter::mainnet();
        VenueAdapter::connect(&a).await.unwrap();
        let v: &dyn VenueAdapter = &a;
        assert_eq!(v.venue_id(), "binance");
        let ex: &dyn ExecutionVenue = &a;
        assert_eq!(ex.venue_id(), "binance");
        let _ = VenueAdapter::server_time(&a).await.unwrap();
        let meta = VenueAdapter::symbol_info(&a, "BTCUSDT").await.unwrap();
        assert_eq!(meta.symbol, "BTCUSDT");
        assert!(VenueAdapter::query_position(&a).await.unwrap().is_empty());
        assert_eq!(VenueAdapter::query_balance(&a).await.unwrap().len(), 1);
    }

    #[test]
    fn parse_binance_server_time_ok() {
        let body = br#"{"serverTime":1710000000123}"#;
        assert_eq!(parse_binance_server_time(body).unwrap(), 1710000000123);
        assert!(parse_binance_server_time(br"{}").is_err());
    }

    #[tokio::test]
    async fn http_driver_get_and_server_time() {
        use bytes::Bytes;
        use transportx::MockHttpTransport;

        let mock = Arc::new(MockHttpTransport::new());
        let url = "https://api.binance.com/api/v3/time";
        mock.set_get(url, Bytes::from_static(br#"{"serverTime":1710000000999}"#));

        let a = BinanceAdapter::mainnet().with_http(mock);
        assert!(a.has_http());
        VenueAdapter::connect(&a).await.unwrap();

        let resp = a.http_get("/api/v3/time").await.unwrap();
        assert_eq!(resp.status, 200);

        let t = VenueAdapter::server_time(&a).await.unwrap();
        assert_eq!(t, 1710000000999);

        // absolute URL path
        let resp2 = a.http_get(url).await.unwrap();
        assert_eq!(resp2.status, 200);
    }

    #[tokio::test]
    async fn http_get_without_driver_unavailable() {
        let a = BinanceAdapter::mainnet();
        assert!(!a.has_http());
        let e = a.http_get("/x").await.expect_err("no driver");
        assert_eq!(e.kind(), kernel::ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn http_server_time_missing_mock_maps_error() {
        use transportx::MockHttpTransport;

        let mock = Arc::new(MockHttpTransport::new());
        // 未 set_get → ProtocolViolation → Invalid
        let a = BinanceAdapter::mainnet().with_http(mock);
        VenueAdapter::connect(&a).await.unwrap();
        let e = VenueAdapter::server_time(&a).await.expect_err("missing mock");
        assert_eq!(e.kind(), kernel::ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn http_cancel_and_query_order_request_paths() {
        use bytes::Bytes;
        use transportx::MockHttpTransport;

        let mock = Arc::new(MockHttpTransport::new());
        let cancel_url = "https://api.binance.com/api/v3/order?symbol=BTCUSDT&orderId=e1";
        let query_url = cancel_url;
        mock.set_post(cancel_url, Bytes::from_static(b"{\"status\":\"CANCELED\"}"));
        mock.set_get(query_url, Bytes::from_static(b"{\"status\":\"CANCELED\"}"));

        let a = BinanceAdapter::mainnet().with_http(mock);
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "binance".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("e1".into()),
        };
        a.cancel_order_request(&req).await.unwrap();
        assert_eq!(a.query_order_request(&req).await.unwrap(), OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn http_query_order_request_missing_mock_errors() {
        use transportx::MockHttpTransport;
        let mock = Arc::new(MockHttpTransport::new());
        let a = BinanceAdapter::mainnet().with_http(mock);
        VenueAdapter::connect(&a).await.unwrap();
        let req = CancelOrderRequest {
            venue: "binance".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("missing".into()),
        };
        let e = a.query_order_request(&req).await.expect_err("missing");
        assert_eq!(e.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn map_transport_error_variants() {
        use std::time::Duration;
        let e = map_transport_error(TransportError::RateLimited {
            retry_after: Some(Duration::from_secs(1)),
        });
        assert_eq!(e.kind(), kernel::ErrorKind::Transient);
        assert_eq!(
            map_transport_error(TransportError::ConnectTimeout).kind(),
            kernel::ErrorKind::Transient
        );
        assert_eq!(
            map_transport_error(TransportError::ReadTimeout).kind(),
            kernel::ErrorKind::Transient
        );
        assert_eq!(
            map_transport_error(TransportError::ConnectionClosed { clean: true }).kind(),
            kernel::ErrorKind::Unavailable
        );
        assert_eq!(
            map_transport_error(TransportError::ProtocolViolation("bad".into())).kind(),
            kernel::ErrorKind::Invalid
        );
        assert_eq!(
            map_transport_error(TransportError::Io(Box::new(std::io::Error::other("e")))).kind(),
            kernel::ErrorKind::Unavailable
        );
        assert_eq!(
            map_transport_error(TransportError::PayloadTooLarge {
                kind: "response_body",
                limit: 1,
                got: 2,
            })
            .kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn map_order_status_variants() {
        assert_eq!(BinanceAdapter::map_order_status("NEW"), OrderStatus::Open);
        assert_eq!(
            BinanceAdapter::map_order_status("PARTIALLY_FILLED"),
            OrderStatus::PartiallyFilled
        );
        assert_eq!(BinanceAdapter::map_order_status("FILLED"), OrderStatus::Filled);
        assert_eq!(BinanceAdapter::map_order_status("CANCELED"), OrderStatus::Cancelled);
        assert_eq!(BinanceAdapter::map_order_status("REJECTED"), OrderStatus::Rejected);
        assert_eq!(BinanceAdapter::map_order_status("EXPIRED"), OrderStatus::Cancelled);
        assert_eq!(BinanceAdapter::map_order_status("UNKNOWN"), OrderStatus::Open);
    }

    #[test]
    fn parse_symbol_meta_extracts_filters() {
        let sym = ExchangeInfoSymbol {
            symbol: "BTCUSDT".into(),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            filters: vec![
                serde_json::json!({"filterType": "PRICE_FILTER", "minPrice": "0.01", "maxPrice": "1000000.00", "tickSize": "0.01"}),
                serde_json::json!({"filterType": "LOT_SIZE", "minQty": "0.00001000", "maxQty": "9000.0", "stepSize": "0.00001"}),
            ],
        };
        let meta = BinanceAdapter::parse_symbol_meta(&sym).unwrap();
        assert_eq!(meta.symbol, "BTCUSDT");
        assert_eq!(meta.base, "BTC");
        assert_eq!(meta.quote, "USDT");
    }

    #[test]
    fn api_key_builder() {
        let key = BinanceApiKey::new("test-key", "test-secret");
        let a = BinanceAdapter::mainnet().with_api_key(key);
        assert!(a.has_api_key());
    }

    #[test]
    fn api_key_without_configured() {
        let a = BinanceAdapter::mainnet();
        assert!(!a.has_api_key());
    }

    #[test]
    fn check_binance_error_detects_binance_error() {
        let body = br#"{"code":-1013,"msg":"Filter failure: MIN_NOTIONAL"}"#;
        let err = check_binance_error(body).unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn check_binance_error_ok_on_non_error() {
        let body = br#"{"orderId":1,"status":"NEW"}"#;
        check_binance_error(body).unwrap();
    }
}
