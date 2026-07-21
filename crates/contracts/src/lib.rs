//! contracts —— 契约层 trait 出口（spec §4.3，R4，Additive Only）。
//!
//! 只放 trait/type，不放实现。一旦发布不可修改签名，只能新增（Additive Only）。
//! 依赖白名单（R4）：kernel + canonical + async-trait/bytes/futures-core。
use async_trait::async_trait;
use bytes::Bytes;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::time::Duration;

// ── storage 契约 ──────────────────────────

/// 键值存储（spec §4.3，redisx 实现）。
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>>;
    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()>;
}

/// 事件总线（spec §4.3，kafkax/natsx 实现）。
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()>;
    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, Bytes>>;
}

/// 仓储（spec §4.3，postgresx 实现）。
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    async fn find(&self, id: Id) -> XResult<Option<T>>;
    async fn save(&self, entity: &T) -> XResult<()>;
}

/// 事务运行器（待新增，postgresx 实现）。
#[async_trait]
pub trait TxRunner: Send + Sync {
    async fn run_tx<F, R>(&self, f: F) -> XResult<R>
    where
        F: std::future::Future<Output = XResult<R>> + Send,
        R: Send;
}

/// 时序数据存储（待新增，ADR-003，taosx 实现，native/rest 双 feature）。
#[async_trait]
pub trait TimeSeriesStore: Send + Sync {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()>;
    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>>;
}

/// 对象存储（待新增，ossx 实现）。
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put_object(&self, key: &str, data: Bytes) -> XResult<()>;
    async fn get_object(&self, key: &str) -> XResult<Bytes>;
}

/// 分析数据汇聚（待新增，clickhousex 实现）。
#[async_trait]
pub trait AnalyticsSink: Send + Sync {
    async fn sink(&self, event: &str, payload: Bytes) -> XResult<()>;
}

/// 发布订阅（待新增可选，redisx 实现）。
#[async_trait]
pub trait PubSub: Send + Sync {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()>;
    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, Bytes>>;
}

// ── observability 契约（ADR-005）──────────

/// 可观测性注入点（ADR-005，observex 实现，resiliencx 消费）。
pub trait Instrumentation: Send + Sync {
    fn record_retry(&self, op: &str, attempt: u32);
    fn record_circuit_open(&self, op: &str);
    fn record_circuit_close(&self, op: &str);
}

// ── venue 契约（ADR-001）──────────────────

/// 交易所适配器（ADR-001，/exchange/* 实现，domain_exchange 消费）。
/// 签名只引用 canonical / decimalx 的类型。
#[async_trait]
pub trait VenueAdapter: Send + Sync {
    async fn connect(&self) -> XResult<()>;
    async fn disconnect(&self) -> XResult<()>;
    async fn place_order(&self, order: &Order) -> XResult<OrderAck>;
    /// Legacy cancel by opaque wire id string (often `{symbol}:{exchange_id}`).
    ///
    /// Prefer [`Self::cancel_order_request`].
    #[deprecated(note = "use cancel_order_request(&CancelOrderRequest) (CAN-ID)")]
    async fn cancel_order(&self, id: &str) -> XResult<()>;
    /// Legacy query by opaque wire id string.
    ///
    /// Prefer [`Self::query_order_request`].
    #[deprecated(note = "use query_order_request(&CancelOrderRequest) (CAN-ID)")]
    async fn query_order(&self, id: &str) -> XResult<OrderStatus>;
    /// Structured cancel (preferred; CAN-ID Approved 2026-07-17).
    ///
    /// **Additive default**: returns an error so out-of-tree implementers keep compiling
    /// until they override. In-tree adapters must override.
    async fn cancel_order_request(&self, request: &CancelOrderRequest) -> XResult<()> {
        let _ = request;
        Err(XError::invalid(
            "cancel_order_request not implemented; override VenueAdapter::cancel_order_request (CAN-ID)",
        ))
    }
    /// Structured query (preferred; CAN-ID Approved 2026-07-17).
    ///
    /// **Additive default**: returns an error so out-of-tree implementers keep compiling
    /// until they override. In-tree adapters must override.
    async fn query_order_request(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        let _ = request;
        Err(XError::invalid(
            "query_order_request not implemented; override VenueAdapter::query_order_request (CAN-ID)",
        ))
    }
    async fn query_position(&self) -> XResult<Vec<Position>>;
    async fn query_balance(&self) -> XResult<Vec<Money>>;
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>>;
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>>;
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>>;
    async fn server_time(&self) -> XResult<i64>;
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta>;
    /// 静态标识，无异步语义。
    fn venue_id(&self) -> &'static str;
}

/// Market-data capability extracted from [`VenueAdapter`].
#[async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>>;
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>>;
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>>;
}

#[async_trait]
pub trait InstrumentCatalog: Send + Sync {
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta>;
}

/// Execution capability with structured cancellation.
#[async_trait]
pub trait ExecutionVenue: Send + Sync {
    async fn place_order(&self, order: &Order) -> XResult<OrderAck>;
    async fn cancel_order(&self, request: &CancelOrderRequest) -> XResult<()>;
    async fn query_order(&self, request: &CancelOrderRequest) -> XResult<OrderStatus>;
    fn venue_id(&self) -> VenueId;
}

#[async_trait]
pub trait AccountSource: Send + Sync {
    async fn query_position(&self) -> XResult<Vec<Position>>;
    async fn query_balance(&self) -> XResult<Vec<Money>>;
}

#[async_trait]
pub trait VenueTimeSource: Send + Sync {
    async fn server_time(&self) -> XResult<i64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 编译期验证：所有 trait 可被实现（mock 空实现）。
    struct MockKv;
    #[async_trait]
    impl KeyValueStore for MockKv {
        async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
            Ok(None)
        }
        async fn set(&self, _key: &str, _val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
            Ok(())
        }
    }

    #[test]
    fn keyvaluestore_trait_implementable() {
        let _m = MockKv;
    }

    struct MockInstr;
    impl Instrumentation for MockInstr {
        fn record_retry(&self, _op: &str, _attempt: u32) {}
        fn record_circuit_open(&self, _op: &str) {}
        fn record_circuit_close(&self, _op: &str) {}
    }

    #[test]
    fn instrumentation_trait_implementable() {
        let _m = MockInstr;
    }
}
