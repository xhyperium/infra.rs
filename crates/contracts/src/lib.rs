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

/// 总线消息：具备 ID 与 payload；确认模型见 [`MessageAck`]。
///
/// 能力边界：本最小面表达 at-most-once 消费（无 redelivery 保证）。
/// at-least-once / 事务消息需扩展 trait（Additive Only）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusMessage {
    /// 实现定义的消息 ID（分区内唯一或全局唯一由后端约定）。
    pub id: String,
    /// 原始载荷。
    pub payload: Bytes,
}

/// 消息确认动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAck {
    /// 确认处理成功。
    Ack,
    /// 否定确认；是否重投递由后端能力决定（最小面不保证）。
    Nack,
}

/// 事件总线（spec §4.3，kafkax/natsx 实现）。
///
/// 合同：
/// - `publish` 失败必须返回可分类 [`XError`]（不得 panic）；
/// - `subscribe` 流项为 [`BusMessage`]，调用方可按 `id` 做幂等；
/// - 本 trait **不**内建 ack API（无 handle）；需要确认语义时实现后端扩展或使用包装。
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()>;
    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>>;
}

/// 仓储（spec §4.3，postgresx 实现）。
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    async fn find(&self, id: Id) -> XResult<Option<T>>;
    async fn save(&self, entity: &T) -> XResult<()>;
}

/// 事务上下文：可显式 commit / rollback。
///
/// 合同：
/// - 业务成功路径应调用 [`TxContext::commit`]（或由编排层在 Ok 后调用）；
/// - 业务失败路径应调用 [`TxContext::rollback`]；
/// - 幂等：对同一上下文重复 `commit`/`rollback` 的行为由实现定义，但不得在
///   已终结后静默再次变更外部状态。
#[async_trait]
pub trait TxContext: Send {
    /// 提交事务。
    async fn commit(&mut self) -> XResult<()>;
    /// 回滚事务。
    async fn rollback(&mut self) -> XResult<()>;
}

/// 事务运行器（postgresx 等实现）。
///
/// 生产合同：[`TxRunner::begin_tx`] 返回可测的 [`TxContext`]；
/// trait **对象安全**（`dyn TxRunner` 可用）。编排示例见 `run_tx_commit_on_ok`。
#[async_trait]
pub trait TxRunner: Send + Sync {
    /// 开启事务，返回上下文句柄。
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>>;
}

/// 参考编排：`Ok` → commit，`Err` → rollback（驱动真实 [`TxContext`] 路径）。
pub async fn run_tx_commit_on_ok<R, F, Fut>(runner: &dyn TxRunner, f: F) -> XResult<R>
where
    F: FnOnce(&mut dyn TxContext) -> Fut + Send,
    Fut: std::future::Future<Output = XResult<R>> + Send,
    R: Send,
{
    let mut ctx = runner.begin_tx().await?;
    match f(ctx.as_mut()).await {
        Ok(v) => {
            ctx.commit().await?;
            Ok(v)
        }
        Err(e) => {
            let _ = ctx.rollback().await;
            Err(e)
        }
    }
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

/// 发布订阅（可选，redisx 实现）。
///
/// 与 [`EventBus`] 类似，stream 项为 [`BusMessage`]；能力边界：至少 at-most-once。
#[async_trait]
pub trait PubSub: Send + Sync {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()>;
    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>>;
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
            "cancel_order_request 未实现；请覆盖 VenueAdapter::cancel_order_request（CAN-ID）",
        ))
    }
    /// Structured query (preferred; CAN-ID Approved 2026-07-17).
    ///
    /// **Additive default**: returns an error so out-of-tree implementers keep compiling
    /// until they override. In-tree adapters must override.
    async fn query_order_request(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        let _ = request;
        Err(XError::invalid(
            "query_order_request 未实现；请覆盖 VenueAdapter::query_order_request（CAN-ID）",
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

// ── contract-testkit（最小可运行入口）──────────────────

/// 内存参考事务：记录 commit/rollback，供合同测试驱动真实 trait 路径。
#[derive(Debug, Default)]
pub struct FakeTxContext {
    /// 是否已 commit。
    pub committed: bool,
    /// 是否已 rollback。
    pub rolled_back: bool,
    fail_commit: bool,
}

impl FakeTxContext {
    /// 新建干净上下文。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入 commit 失败。
    pub fn with_commit_failure(mut self) -> Self {
        self.fail_commit = true;
        self
    }
}

#[async_trait]
impl TxContext for FakeTxContext {
    async fn commit(&mut self) -> XResult<()> {
        if self.fail_commit {
            return Err(XError::transient("事务提交失败（注入）"));
        }
        self.committed = true;
        self.rolled_back = false;
        Ok(())
    }
    async fn rollback(&mut self) -> XResult<()> {
        self.rolled_back = true;
        self.committed = false;
        Ok(())
    }
}

/// 内存 TxRunner 参考实现。
#[derive(Debug, Default)]
pub struct FakeTxRunner;

#[async_trait]
impl TxRunner for FakeTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(FakeTxContext::new()))
    }
}

/// 内存 EventBus 参考实现（at-most-once 进程内）。
#[derive(Debug, Default)]
pub struct FakeEventBus {
    inner: std::sync::Mutex<std::collections::HashMap<String, Vec<BusMessage>>>,
    seq: std::sync::atomic::AtomicU64,
}

impl FakeEventBus {
    /// 新建空总线。
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EventBus for FakeEventBus {
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()> {
        let id = self.seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_string();
        let mut g = self.inner.lock().map_err(|_| XError::internal("event bus lock poisoned"))?;
        g.entry(topic.to_string()).or_default().push(BusMessage { id, payload });
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = {
            let g = self.inner.lock().map_err(|_| XError::internal("event bus lock poisoned"))?;
            g.get(topic).cloned().unwrap_or_default()
        };
        Ok(Box::pin(VecBusStream { inner: msgs.into_iter() }))
    }
}

/// 简单的一次性消息流（contract-testkit 内部）。
struct VecBusStream {
    inner: std::vec::IntoIter<BusMessage>,
}

impl futures_core::Stream for VecBusStream {
    type Item = BusMessage;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.inner.next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::{CancelOrderRequest, OrderRef, Side};
    use decimalx::{Decimal, Price, Qty};
    use futures_core::Stream;

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

    #[tokio::test]
    async fn keyvaluestore_methods_callable() {
        let m = MockKv;
        assert!(m.get("k").await.unwrap().is_none());
        m.set("k", vec![1], None).await.unwrap();
    }

    struct MockInstr;
    impl Instrumentation for MockInstr {
        fn record_retry(&self, _op: &str, _attempt: u32) {}
        fn record_circuit_open(&self, _op: &str) {}
        fn record_circuit_close(&self, _op: &str) {}
    }

    #[test]
    fn instrumentation_methods_callable() {
        let m = MockInstr;
        m.record_retry("op", 1);
        m.record_circuit_open("op");
        m.record_circuit_close("op");
        let d: &dyn Instrumentation = &m;
        d.record_retry("op", 2);
    }

    struct MockVenue;
    #[async_trait]
    impl VenueAdapter for MockVenue {
        async fn connect(&self) -> XResult<()> {
            Ok(())
        }
        async fn disconnect(&self) -> XResult<()> {
            Ok(())
        }
        async fn place_order(&self, _order: &Order) -> XResult<OrderAck> {
            Err(XError::invalid("not implemented"))
        }
        async fn cancel_order(&self, _id: &str) -> XResult<()> {
            Ok(())
        }
        async fn query_order(&self, _id: &str) -> XResult<OrderStatus> {
            Ok(OrderStatus::Pending)
        }
        async fn query_position(&self) -> XResult<Vec<Position>> {
            Ok(vec![])
        }
        async fn query_balance(&self) -> XResult<Vec<Money>> {
            Ok(vec![])
        }
        async fn subscribe_ticks(&self, _symbol: &str) -> XResult<BoxStream<'static, Tick>> {
            Err(XError::invalid("not implemented"))
        }
        async fn subscribe_orderbook(
            &self,
            _symbol: &str,
        ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
            Err(XError::invalid("not implemented"))
        }
        async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
            Err(XError::invalid("not implemented"))
        }
        async fn server_time(&self) -> XResult<i64> {
            Ok(0)
        }
        async fn symbol_info(&self, _symbol: &str) -> XResult<SymbolMeta> {
            Err(XError::invalid("not implemented"))
        }
        fn venue_id(&self) -> &'static str {
            "mock"
        }
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn venue_adapter_default_request_methods_error() {
        let v = MockVenue;
        let req = CancelOrderRequest {
            venue: "mock".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("x".into()),
        };
        let e1 = v.cancel_order_request(&req).await.unwrap_err();
        assert_eq!(e1.kind(), kernel::ErrorKind::Invalid);
        let e2 = v.query_order_request(&req).await.unwrap_err();
        assert_eq!(e2.kind(), kernel::ErrorKind::Invalid);
        assert_eq!(v.venue_id(), "mock");
        v.connect().await.unwrap();
        v.disconnect().await.unwrap();
        v.cancel_order("id").await.unwrap();
        assert_eq!(v.query_order("id").await.unwrap(), OrderStatus::Pending);
        assert!(v.query_position().await.unwrap().is_empty());
        assert!(v.query_balance().await.unwrap().is_empty());
        assert_eq!(v.server_time().await.unwrap(), 0);
        assert!(v.subscribe_ticks("BTCUSDT").await.is_err());
        assert!(v.subscribe_orderbook("BTCUSDT").await.is_err());
        assert!(v.subscribe_trades("BTCUSDT").await.is_err());
        assert!(v.symbol_info("BTCUSDT").await.is_err());
        let order = Order {
            id: "1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: Price::new(Decimal::new(1, 0)),
            qty: Qty::new(Decimal::new(1, 0)),
            status: OrderStatus::Pending,
        };
        assert!(v.place_order(&order).await.is_err());
    }
    #[tokio::test]
    async fn tx_runner_commit_path() {
        let runner = FakeTxRunner;
        let out =
            run_tx_commit_on_ok(&runner, |_ctx| async move { Ok(42u32) }).await.expect("commit ok");
        assert_eq!(out, 42);
    }

    #[tokio::test]
    async fn tx_runner_err_triggers_rollback_path() {
        let runner = FakeTxRunner;
        let err = run_tx_commit_on_ok(&runner, |_ctx| async move {
            Err::<u32, _>(XError::invalid("业务失败"))
        })
        .await
        .unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(err.context().contains("业务失败"));
    }

    #[tokio::test]
    async fn tx_runner_is_object_safe() {
        let runner: &dyn TxRunner = &FakeTxRunner;
        let mut ctx = runner.begin_tx().await.unwrap();
        ctx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn fake_tx_context_commit_success_sets_flags() {
        let mut ctx = FakeTxContext::new();
        ctx.commit().await.unwrap();
        assert!(ctx.committed);
        assert!(!ctx.rolled_back);
    }

    #[tokio::test]
    async fn fake_tx_context_commit_failure_injection() {
        let mut ctx = FakeTxContext::new().with_commit_failure();
        let err = ctx.commit().await.unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Transient);
        assert!(!ctx.committed);
        ctx.rollback().await.unwrap();
        assert!(ctx.rolled_back);
        assert!(!ctx.committed);
    }

    /// 可观察 commit/rollback 的 runner：证明 `run_tx_commit_on_ok` 真正驱动 [`TxContext`]。
    struct RecordingTxRunner {
        committed: std::sync::Arc<std::sync::Mutex<bool>>,
        rolled_back: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    impl RecordingTxRunner {
        fn new() -> Self {
            Self {
                committed: std::sync::Arc::new(std::sync::Mutex::new(false)),
                rolled_back: std::sync::Arc::new(std::sync::Mutex::new(false)),
            }
        }
    }

    struct RecordingTxContext {
        inner: FakeTxContext,
        committed: std::sync::Arc<std::sync::Mutex<bool>>,
        rolled_back: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl TxContext for RecordingTxContext {
        async fn commit(&mut self) -> XResult<()> {
            self.inner.commit().await?;
            *self.committed.lock().expect("lock") = self.inner.committed;
            *self.rolled_back.lock().expect("lock") = self.inner.rolled_back;
            Ok(())
        }
        async fn rollback(&mut self) -> XResult<()> {
            self.inner.rollback().await?;
            *self.committed.lock().expect("lock") = self.inner.committed;
            *self.rolled_back.lock().expect("lock") = self.inner.rolled_back;
            Ok(())
        }
    }

    #[async_trait]
    impl TxRunner for RecordingTxRunner {
        async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
            Ok(Box::new(RecordingTxContext {
                inner: FakeTxContext::new(),
                committed: std::sync::Arc::clone(&self.committed),
                rolled_back: std::sync::Arc::clone(&self.rolled_back),
            }))
        }
    }

    #[tokio::test]
    async fn run_tx_commit_on_ok_drives_real_commit() {
        let runner = RecordingTxRunner::new();
        let out = run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, XError>(7u8) })
            .await
            .expect("ok path");
        assert_eq!(out, 7);
        assert!(*runner.committed.lock().expect("lock"), "编排必须调用 commit");
        assert!(!*runner.rolled_back.lock().expect("lock"));
    }

    #[tokio::test]
    async fn run_tx_err_path_calls_rollback_on_context() {
        let runner = RecordingTxRunner::new();
        let err = run_tx_commit_on_ok(&runner, |_ctx| async move {
            Err::<(), _>(XError::invalid("回滚路径"))
        })
        .await
        .unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(*runner.rolled_back.lock().expect("lock"), "失败路径必须 rollback");
        assert!(!*runner.committed.lock().expect("lock"));
    }

    #[tokio::test]
    async fn event_bus_publish_subscribe_message_contract() {
        let bus = FakeEventBus::new();
        bus.publish("orders", Bytes::from_static(b"hi")).await.unwrap();
        let mut stream = bus.subscribe("orders").await.unwrap();
        use std::pin::Pin;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        fn dummy_raw_waker() -> RawWaker {
            fn no(_: *const ()) {}
            fn clone(_: *const ()) -> RawWaker {
                dummy_raw_waker()
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no, no, no);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(Some(msg)) => {
                assert_eq!(msg.payload.as_ref(), b"hi");
                assert!(!msg.id.is_empty());
            }
            other => panic!("expected message, got {other:?}"),
        }
    }

    #[test]
    fn message_ack_and_bus_message_surface() {
        let m = BusMessage { id: "1".into(), payload: Bytes::from_static(b"x") };
        assert_eq!(m.id, "1");
        assert_eq!(MessageAck::Ack, MessageAck::Ack);
        assert_ne!(MessageAck::Ack, MessageAck::Nack);
    }
}
