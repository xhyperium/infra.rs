//! Exchange domain model: VenueAdapter trait, StreamType, OrderAmend, AccountInfo.

use async_trait::async_trait;
use domain_market::{InstrumentKey, OrderBook};
use domainx::{Decimal, Order, OrderId, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Re-export ExecutionReport from domainx since it appears in the public trait interface.
pub use domainx::ExecutionReport;

// ---------------------------------------------------------------------------
// StreamType
// ---------------------------------------------------------------------------

/// Type of market data stream.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StreamType {
    Ticker,
    Level1,
    Trade,
    Level2,
    MiniTicker,
}

// ---------------------------------------------------------------------------
// OrderAmend
// ---------------------------------------------------------------------------

/// Parameters for amending an existing order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAmend {
    /// ID of the order to amend.
    pub order_id: OrderId,
    /// New limit price (None = unchanged).
    pub price: Option<Decimal>,
    /// New quantity (None = unchanged).
    pub quantity: Option<Decimal>,
    /// New stop/trigger price (None = unchanged).
    pub stop_price: Option<Decimal>,
    /// New client-order ID (optional, exchange-dependent).
    pub new_client_order_id: Option<String>,
}

// ---------------------------------------------------------------------------
// AccountInfo / Balance
// ---------------------------------------------------------------------------

/// A single asset balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Asset identifier (e.g. "BTC", "USDT").
    pub asset: String,
    /// Free (available) balance.
    pub free: Decimal,
    /// Locked (in-order) balance.
    pub locked: Decimal,
}

/// Account information including balances and permission flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Account identifier.
    pub account_id: String,
    /// Asset balances.
    pub balances: Vec<Balance>,
    /// Whether trading is permitted.
    pub can_trade: bool,
    /// Whether withdrawals are permitted.
    pub can_withdraw: bool,
    /// Whether deposits are permitted.
    pub can_deposit: bool,
    /// Last update timestamp (Unix ms).
    pub update_time: Timestamp,
}

// ---------------------------------------------------------------------------
// InstrumentMeta / InstrumentStatus
// ---------------------------------------------------------------------------

/// Trading status of an instrument.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstrumentStatus {
    Trading,
    Break,
    Halt,
    Closed,
}

/// Metadata describing a tradable instrument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentMeta {
    /// The instrument identifier.
    pub symbol: String,
    /// Current trading status.
    pub status: InstrumentStatus,
    /// Base asset (e.g. "BTC").
    pub base_asset: String,
    /// Quote asset (e.g. "USDT").
    pub quote_asset: String,
    /// Number of decimal places for price.
    pub price_precision: u32,
    /// Number of decimal places for quantity.
    pub quantity_precision: u32,
    /// Minimum notional value (optional).
    pub min_notional: Option<Decimal>,
    /// Minimum order quantity (optional).
    pub min_quantity: Option<Decimal>,
    /// Quantity step size (optional).
    pub step_size: Option<Decimal>,
    /// Price tick size (optional).
    pub tick_size: Option<Decimal>,
}

// ---------------------------------------------------------------------------
// DE-ERR-001 / DE-CAP-001 / DE-PAGE-001 支撑类型
// ---------------------------------------------------------------------------

/// 限频作用域（DE-ERR-001）。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateLimitScope {
    Ip,
    Account,
    Endpoint,
    Global,
    Other(String),
}

/// 结构化限频/供应商错误上下文（DE-ERR-001）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitDetail {
    /// 人类可读说明。
    pub message: String,
    /// 建议退避毫秒（来自 Retry-After 或 provider）。
    pub retry_after_ms: Option<u64>,
    /// 限频作用域。
    pub scope: Option<RateLimitScope>,
    /// 供应商错误码。
    pub provider_code: Option<String>,
    /// HTTP 状态（若适用）。
    pub http_status: Option<u16>,
    /// 请求/链路 ID。
    pub request_id: Option<String>,
}

impl std::fmt::Display for RateLimitDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ms) = self.retry_after_ms {
            write!(f, " (retry_after_ms={ms})")?;
        }
        if let Some(code) = &self.provider_code {
            write!(f, " [code={code}]")?;
        }
        Ok(())
    }
}

/// Venue 能力矩阵（DE-CAP-001）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueCapabilities {
    /// 支持的产品线标签（自由字符串，避免强绑 domain_market ProductLine 演进）。
    pub product_lines: Vec<String>,
    /// 支持的行情流。
    pub streams: Vec<StreamType>,
    /// 是否支持交易（下单/撤单）。
    pub can_trade: bool,
    /// 是否支持 WebSocket 订阅。
    pub can_subscribe_ws: bool,
    /// 是否支持账户查询。
    pub can_query_account: bool,
    /// 是否支持改单。
    pub supports_amend: bool,
    /// 是否声明分页契约（见 Page / PageRequest）。
    pub supports_pagination: bool,
}

impl VenueCapabilities {
    /// 未知/未声明能力的保守默认：全 false、空列表。
    pub fn unknown() -> Self {
        Self {
            product_lines: vec![],
            streams: vec![],
            can_trade: false,
            can_subscribe_ws: false,
            can_query_account: false,
            supports_amend: false,
            supports_pagination: false,
        }
    }

    /// REST-only 公开数据源（如 Coinglass）的典型矩阵。
    pub fn rest_only_public() -> Self {
        Self {
            product_lines: vec!["aggregate".into()],
            streams: vec![],
            can_trade: false,
            can_subscribe_ws: false,
            can_query_account: false,
            supports_amend: false,
            supports_pagination: true,
        }
    }
}

/// 分页请求（DE-PAGE-001）。配合 `get_*_page` 使用；`Vec` 方法仍为单页兼容入口。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
}

/// 分页响应（DE-PAGE-001）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    /// 是否可能仍有后续页；`false` 表示完整。
    pub has_more: bool,
}

impl<T> Page<T> {
    pub fn single_page(items: Vec<T>) -> Self {
        Self { items, next_cursor: None, has_more: false }
    }

    /// 从供应商游标构造：有 next_cursor 则 has_more=true。
    pub fn from_cursor(items: Vec<T>, next_cursor: Option<String>) -> Self {
        let has_more = next_cursor.is_some();
        Self { items, next_cursor, has_more }
    }
}

/// DE-PAGE：`has_more == true` 时必须提供 `next_cursor`。
pub fn page_cursor_is_consistent<T>(page: &Page<T>) -> bool {
    if page.has_more { page.next_cursor.is_some() } else { true }
}

/// 应用 `PageRequest.limit` 截断 items。
pub fn apply_page_limit<T>(mut items: Vec<T>, limit: Option<u32>) -> Vec<T> {
    if let Some(lim) = limit {
        let n = lim as usize;
        if items.len() > n {
            items.truncate(n);
        }
    }
    items
}

// ---------------------------------------------------------------------------
// AdapterError
// ---------------------------------------------------------------------------

/// Errors that can occur during exchange adapter operations.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Authentication / API key failure.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limit exceeded（兼容扁平消息）。
    #[error("Rate limit: {0}")]
    RateLimit(String),

    /// 结构化限频（DE-ERR-001 首选）。
    #[error("Rate limit: {0}")]
    RateLimitDetailed(RateLimitDetail),

    /// Network / transport error.
    #[error("Network error: {0}")]
    Network(String),

    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Response parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Internal adapter error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// 该 venue 明确不支持该操作（例如 REST-only 不能订阅 WebSocket）。
    ///
    /// 不得伪装为 `Network` / `WebSocket` / `Internal`。
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

impl AdapterError {
    /// 构造结构化限频错误。
    pub fn rate_limit_detailed(detail: RateLimitDetail) -> Self {
        Self::RateLimitDetailed(detail)
    }

    /// 读取 retry_after_ms（仅结构化变体）。
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimitDetailed(d) => d.retry_after_ms,
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// VenueAdapter trait
// ---------------------------------------------------------------------------

/// Unified asynchronous trait for exchange venue adapters.
///
/// Implementations hide the protocol differences between exchanges
/// (REST vs WebSocket, auth schemes, message formats) behind a
/// single interface.
#[async_trait]
pub trait VenueAdapter: Send + Sync {
    /// 稳定 exchange / provider id（小写 key，如 `binance`、`coinglass`）。
    ///
    /// 默认 `"unknown"`；实现方应覆盖（DE-CAP-001）。
    fn exchange_id(&self) -> &str {
        "unknown"
    }

    /// 能力矩阵；默认全未知/关闭（DE-CAP-001）。
    fn capabilities(&self) -> VenueCapabilities {
        VenueCapabilities::unknown()
    }

    /// Establish a session with the exchange.
    async fn connect(&self) -> Result<(), AdapterError>;

    /// Disconnect and clean up the session.
    async fn disconnect(&self) -> Result<(), AdapterError>;

    /// Subscribe to ticker (24hr rolling stats) updates.
    async fn subscribe_ticker(&self, instrument: &InstrumentKey) -> Result<(), AdapterError>;

    /// Subscribe to order book (depth) updates.
    async fn subscribe_order_book(&self, instrument: &InstrumentKey) -> Result<(), AdapterError>;

    /// Subscribe to trade (fill) updates.
    async fn subscribe_trades(&self, instrument: &InstrumentKey) -> Result<(), AdapterError>;

    /// Place a new order.
    async fn place_order(&self, order: &Order) -> Result<ExecutionReport, AdapterError>;

    /// Cancel an open order.
    async fn cancel_order(
        &self,
        order_id: &OrderId,
        instrument: &InstrumentKey,
    ) -> Result<(), AdapterError>;

    /// Amend (modify) an existing order.
    async fn amend_order(&self, amend: &OrderAmend) -> Result<ExecutionReport, AdapterError>;

    /// Get a single order by ID.
    async fn get_order(
        &self,
        order_id: &OrderId,
        instrument: &InstrumentKey,
    ) -> Result<Order, AdapterError>;

    /// Get all open orders for an instrument.
    async fn get_open_orders(&self, instrument: &InstrumentKey)
    -> Result<Vec<Order>, AdapterError>;

    /// Get account information (balances, permissions).
    async fn get_account_info(&self) -> Result<AccountInfo, AdapterError>;

    /// Get metadata for all tradable instruments（单页，不保证全量）。
    async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError>;

    /// Get a snapshot of the order book.
    async fn get_order_book(
        &self,
        instrument: &InstrumentKey,
        limit: Option<u32>,
    ) -> Result<OrderBook, AdapterError>;

    /// 分页未结订单（DE-PAGE-001）。
    ///
    /// 默认：调用 `get_open_orders`，忽略 cursor，应用 limit，单页返回。
    async fn get_open_orders_page(
        &self,
        instrument: &InstrumentKey,
        request: PageRequest,
    ) -> Result<Page<Order>, AdapterError> {
        let items = self.get_open_orders(instrument).await?;
        let items = apply_page_limit(items, request.limit);
        Ok(Page::single_page(items))
    }

    /// 分页标的元数据（DE-PAGE-001）。
    ///
    /// 默认：调用 `get_instruments`，忽略 cursor，应用 limit，单页返回。
    async fn get_instruments_page(
        &self,
        request: PageRequest,
    ) -> Result<Page<InstrumentMeta>, AdapterError> {
        let items = self.get_instruments().await?;
        let items = apply_page_limit(items, request.limit);
        Ok(Page::single_page(items))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_type() {
        match StreamType::Ticker {
            StreamType::Ticker
            | StreamType::Level1
            | StreamType::Trade
            | StreamType::Level2
            | StreamType::MiniTicker => {}
        }
    }

    #[test]
    fn test_order_amend() {
        let amend = OrderAmend {
            order_id: "o1".into(),
            price: Some(Decimal::new(51000, 0)),
            quantity: None,
            stop_price: None,
            new_client_order_id: None,
        };
        assert_eq!(amend.order_id, "o1");
        assert_eq!(amend.price, Some(Decimal::new(51000, 0)));
    }

    #[test]
    fn test_balance() {
        let bal =
            Balance { asset: "BTC".into(), free: Decimal::new(1, 0), locked: Decimal::new(5, 1) };
        assert_eq!(bal.asset, "BTC");
        assert!(bal.free > bal.locked);
    }

    #[test]
    fn test_account_info() {
        let info = AccountInfo {
            account_id: "acc1".into(),
            balances: vec![
                Balance { asset: "BTC".into(), free: Decimal::new(1, 0), locked: Decimal::ZERO },
                Balance {
                    asset: "USDT".into(),
                    free: Decimal::new(50000, 0),
                    locked: Decimal::new(10000, 0),
                },
            ],
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            update_time: 1_000_000_000_000,
        };
        assert!(info.can_trade);
        assert_eq!(info.balances.len(), 2);
    }

    #[test]
    fn test_instrument_meta() {
        let meta = InstrumentMeta {
            symbol: "BTCUSDT".into(),
            status: InstrumentStatus::Trading,
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            price_precision: 2,
            quantity_precision: 6,
            min_notional: Some(Decimal::new(10, 0)),
            min_quantity: Some(Decimal::new(1, 4)),
            step_size: Some(Decimal::new(1, 4)),
            tick_size: Some(Decimal::new(1, 2)),
        };
        assert_eq!(meta.base_asset, "BTC");
        assert_eq!(meta.price_precision, 2);
    }

    #[test]
    fn test_adapter_error_display() {
        let err = AdapterError::InvalidRequest("bad price".into());
        assert_eq!(err.to_string(), "Invalid request: bad price");

        let err = AdapterError::RateLimit("too many requests".into());
        assert_eq!(err.to_string(), "Rate limit: too many requests");

        let err = AdapterError::rate_limit_detailed(RateLimitDetail {
            message: "429".into(),
            retry_after_ms: Some(1200),
            scope: Some(RateLimitScope::Ip),
            provider_code: Some("-1003".into()),
            http_status: Some(429),
            request_id: Some("req-1".into()),
        });
        assert!(err.to_string().contains("429"));
        assert_eq!(err.retry_after_ms(), Some(1200));
    }

    /// Mock adapter for testing the VenueAdapter trait compiles.
    struct MockAdapter;

    #[async_trait]
    impl VenueAdapter for MockAdapter {
        async fn connect(&self) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn disconnect(&self) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn subscribe_ticker(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn subscribe_order_book(
            &self,
            _instrument: &InstrumentKey,
        ) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn subscribe_trades(&self, _instrument: &InstrumentKey) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn place_order(&self, order: &Order) -> Result<ExecutionReport, AdapterError> {
            Ok(ExecutionReport {
                report_id: "r1".into(),
                order_id: order.order_id.clone(),
                exec_type: domainx::ExecType::New,
                order_status: domainx::OrderStatus::New,
                instrument: order.instrument.clone(),
                side: order.side.clone(),
                order_type: order.order_type.clone(),
                price: order.price,
                quantity: order.quantity,
                last_filled_price: None,
                last_filled_quantity: None,
                cumulative_filled_quantity: Decimal::ZERO,
                remaining_quantity: order.quantity,
                commission: None,
                trade_id: None,
                reject_reason: None,
                occurred_at: 1_000_000_000_000,
            })
        }

        async fn cancel_order(
            &self,
            _order_id: &OrderId,
            _instrument: &InstrumentKey,
        ) -> Result<(), AdapterError> {
            Ok(())
        }

        async fn amend_order(&self, _amend: &OrderAmend) -> Result<ExecutionReport, AdapterError> {
            Ok(ExecutionReport {
                report_id: "r2".into(),
                order_id: "o1".into(),
                exec_type: domainx::ExecType::Replaced,
                order_status: domainx::OrderStatus::New,
                instrument: "BTCUSDT".into(),
                side: domainx::OrderSide::Buy,
                order_type: domainx::OrderType::Limit,
                price: Some(Decimal::new(51000, 0)),
                quantity: Decimal::new(1, 0),
                last_filled_price: None,
                last_filled_quantity: None,
                cumulative_filled_quantity: Decimal::ZERO,
                remaining_quantity: Decimal::new(1, 0),
                commission: None,
                trade_id: None,
                reject_reason: None,
                occurred_at: 1_000_000_000_000,
            })
        }

        async fn get_order(
            &self,
            _order_id: &OrderId,
            _instrument: &InstrumentKey,
        ) -> Result<Order, AdapterError> {
            Err(AdapterError::Internal("not implemented".into()))
        }

        async fn get_open_orders(
            &self,
            _instrument: &InstrumentKey,
        ) -> Result<Vec<Order>, AdapterError> {
            Ok(vec![])
        }

        async fn get_account_info(&self) -> Result<AccountInfo, AdapterError> {
            Ok(AccountInfo {
                account_id: "mock".into(),
                balances: vec![],
                can_trade: true,
                can_withdraw: false,
                can_deposit: false,
                update_time: 1_000_000_000_000,
            })
        }

        async fn get_instruments(&self) -> Result<Vec<InstrumentMeta>, AdapterError> {
            Ok(vec![])
        }

        async fn get_order_book(
            &self,
            _instrument: &InstrumentKey,
            _limit: Option<u32>,
        ) -> Result<OrderBook, AdapterError> {
            Err(AdapterError::Internal("not implemented".into()))
        }
    }

    #[tokio::test]
    async fn test_mock_adapter_connect() {
        let adapter = MockAdapter;
        assert!(adapter.connect().await.is_ok());
        assert!(adapter.disconnect().await.is_ok());
    }
}
