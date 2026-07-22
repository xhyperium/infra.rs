//! contracts —— 契约层 trait 出口（spec §4.3，R4，Additive Only）。
//!
//! 只放 trait/type 与 VenueAdapter 门禁辅助。
//! Fake/Recording 与 per-trait suite 见独立 crate **`contract-testkit`**
//!（`crates/test-support/contracts`，仅 dev-dep）。
//! 一旦发布不可修改签名，只能新增（Additive Only）。
//! 依赖白名单（R4）：kernel + canonical + async-trait/bytes/futures-core。
//!
//! ## Lint
//!
//! - `forbid(unsafe_code)` / `deny(unreachable_pub)` 已启用。
//! - `missing_docs`：已 `deny`（公开 trait 方法与字段须有文档）。
//!
//! # 生产入口建议
//!
//! - **执行路径**：优先 [`ExecutionVenue`]（结构化 cancel/query，**无** additive default）。
//! - [`VenueAdapter`] 是迁移 facade：`cancel_order_request` / `query_order_request`
//!   带 additive default（中文 `Invalid`）；树内 adapter **必须**覆盖（见 DEFER-8 门禁）。
//!
//! # 语义文档
//!
//! First-batch trait 语义见 `docs/contracts/`。

#![forbid(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(missing_docs)]

use async_trait::async_trait;
use bytes::Bytes;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::time::Duration;

pub mod live;
mod venue_gate;
pub use live::{
    AckedMessage, LiveContractProfile, LiveHandles, apply_ack, bus_publish, kv_roundtrip,
    kv_set_then_commit_separate_resources, kv_set_ttl, publish_without_delivery_attestation,
    repo_roundtrip, run_on_tx_context, tx_kv_set, venue_health, venue_place_and_query,
};
pub use venue_gate::{
    VENUE_CANCEL_REQUEST_DEFAULT_MSG, VENUE_QUERY_REQUEST_DEFAULT_MSG,
    is_default_cancel_order_request_error, is_default_query_order_request_error,
};

// ── storage 契约 ──────────────────────────

/// 键值存储（spec §4.3，redisx 实现）。
///
/// 语义文档：`docs/contracts/key_value_store.md`。
#[async_trait]
pub trait KeyValueStore: Send + Sync {
    /// 读取 key；不存在返回 `Ok(None)`。
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>>;
    /// 写入 key；`ttl` 为可选过期时间。
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
///
/// 语义文档：`docs/contracts/event_bus.md`。
#[async_trait]
pub trait EventBus: Send + Sync {
    /// 发布消息到 topic。
    async fn publish(&self, topic: &str, payload: Bytes) -> XResult<()>;
    /// 订阅 topic，返回消息流。
    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>>;
}

/// 仓储（spec §4.3，postgresx 实现）。
///
/// 语义文档：`docs/contracts/repository.md`。
#[async_trait]
pub trait Repository<T, Id>: Send + Sync {
    /// 按 id 查找实体。
    async fn find(&self, id: Id) -> XResult<Option<T>>;
    /// 保存/更新实体。
    async fn save(&self, entity: &T) -> XResult<()>;
}

/// 事务上下文：可显式 commit / rollback。
///
/// 合同：
/// - 业务成功路径应调用 [`TxContext::commit`]（或由编排层在 Ok 后调用）；
/// - 业务失败路径应调用 [`TxContext::rollback`]；
/// - 幂等：对同一上下文重复 `commit`/`rollback` 的行为由实现定义，但不得在
///   已终结后静默再次变更外部状态。
///
/// 语义文档：`docs/contracts/tx_context.md`。
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
///
/// 语义文档：`docs/contracts/tx_runner.md`。
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
    /// 写入时间序列点。
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()>;
    /// 按时间范围查询（纳秒 epoch）。
    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>>;
}

/// 对象存储（待新增，ossx 实现）。
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// 上传对象。
    async fn put_object(&self, key: &str, data: Bytes) -> XResult<()>;
    /// 下载对象。
    async fn get_object(&self, key: &str) -> XResult<Bytes>;
}

/// 分析数据汇聚（待新增，clickhousex 实现）。
#[async_trait]
pub trait AnalyticsSink: Send + Sync {
    /// 写入分析事件。
    async fn sink(&self, event: &str, payload: Bytes) -> XResult<()>;
}

/// 发布订阅（可选，redisx 实现）。
///
/// 与 [`EventBus`] 类似，stream 项为 [`BusMessage`]；能力边界：至少 at-most-once。
#[async_trait]
pub trait PubSub: Send + Sync {
    /// 发布到 channel。
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()>;
    /// 订阅 channel。
    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>>;
}

// ── observability 契约（ADR-005）──────────

/// 可观测性注入点（ADR-005，observex 实现，resiliencx 消费）。
///
/// 语义文档：`docs/contracts/instrumentation.md`。
pub trait Instrumentation: Send + Sync {
    /// 记录一次重试（`attempt` 从 1 起或由调用方约定）。
    fn record_retry(&self, op: &str, attempt: u32);
    /// 记录熔断打开。
    fn record_circuit_open(&self, op: &str);
    /// 记录熔断关闭。
    fn record_circuit_close(&self, op: &str);
}

// ── venue 契约（ADR-001）──────────────────

/// 交易所适配器（ADR-001，/exchange/* 实现，domain_exchange 消费）。
///
/// 签名只引用 canonical / decimalx 的类型。
///
/// **迁移 facade**：新代码优先 [`ExecutionVenue`] + 能力拆分 trait。
/// `cancel_order_request` / `query_order_request` 的 additive default 返回中文
/// [`XError::invalid`]；树内 adapter 必须覆盖（DEFER-8 / CT-10）。
#[async_trait]
pub trait VenueAdapter: Send + Sync {
    /// 建立与交易所的会话/连接。
    async fn connect(&self) -> XResult<()>;
    /// 断开连接。
    async fn disconnect(&self) -> XResult<()>;
    /// 下单。
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
        Err(XError::invalid(VENUE_CANCEL_REQUEST_DEFAULT_MSG))
    }
    /// Structured query (preferred; CAN-ID Approved 2026-07-17).
    ///
    /// **Additive default**: returns an error so out-of-tree implementers keep compiling
    /// until they override. In-tree adapters must override.
    async fn query_order_request(&self, request: &CancelOrderRequest) -> XResult<OrderStatus> {
        let _ = request;
        Err(XError::invalid(VENUE_QUERY_REQUEST_DEFAULT_MSG))
    }
    /// 查询持仓。
    async fn query_position(&self) -> XResult<Vec<Position>>;
    /// 查询余额。
    async fn query_balance(&self) -> XResult<Vec<Money>>;
    /// 订阅 tick 流。
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>>;
    /// 订阅订单簿快照流。
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>>;
    /// 订阅成交流。
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>>;
    /// 交易所服务器时间（纳秒 epoch 或实现约定刻度；见对齐文档）。
    async fn server_time(&self) -> XResult<i64>;
    /// 查询交易对元数据。
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta>;
    /// 静态标识，无异步语义。
    fn venue_id(&self) -> &'static str;
}

/// Market-data capability extracted from [`VenueAdapter`].
///
/// 语义文档：`docs/contracts/market_data_source.md`。
#[async_trait]
pub trait MarketDataSource: Send + Sync {
    /// 订阅 tick 流。
    async fn subscribe_ticks(&self, symbol: &str) -> XResult<BoxStream<'static, Tick>>;
    /// 订阅订单簿快照流。
    async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>>;
    /// 订阅成交流。
    async fn subscribe_trades(&self, symbol: &str) -> XResult<BoxStream<'static, Trade>>;
}

/// 交易对元数据目录。
///
/// 语义文档：`docs/contracts/instrument_catalog.md`。
#[async_trait]
pub trait InstrumentCatalog: Send + Sync {
    /// 查询交易对元数据。
    async fn symbol_info(&self, symbol: &str) -> XResult<SymbolMeta>;
}

/// 执行能力（结构化 cancel/query；**推荐生产入口**）。
///
/// 与 [`VenueAdapter`] 不同：本 trait **无** additive default，实现方必须提供完整方法。
///
/// 语义文档：`docs/contracts/execution_venue.md`。
#[async_trait]
pub trait ExecutionVenue: Send + Sync {
    /// 下单。
    async fn place_order(&self, order: &Order) -> XResult<OrderAck>;
    /// 结构化撤单。
    async fn cancel_order(&self, request: &CancelOrderRequest) -> XResult<()>;
    /// 结构化查单。
    async fn query_order(&self, request: &CancelOrderRequest) -> XResult<OrderStatus>;
    /// 场所标识。
    fn venue_id(&self) -> VenueId;
}

/// 账户/持仓查询能力。
///
/// 语义文档：`docs/contracts/account_source.md`。
#[async_trait]
pub trait AccountSource: Send + Sync {
    /// 查询持仓。
    async fn query_position(&self) -> XResult<Vec<Position>>;
    /// 查询余额。
    async fn query_balance(&self) -> XResult<Vec<Money>>;
}

/// 交易所服务器时间源。
///
/// 语义文档：`docs/contracts/venue_time_source.md`。
#[async_trait]
pub trait VenueTimeSource: Send + Sync {
    /// 交易所服务器时间。
    async fn server_time(&self) -> XResult<i64>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::{CancelOrderRequest, OrderRef, Side};
    use decimalx::{Decimal, Price, Qty};
    use futures_core::Stream;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    // 注意：本单元测试 **禁止** 依赖 contract-testkit（dev-dep 环会造成
    // contracts 双版本，Fake 实现的 trait 与本 crate cfg(test) 类型不兼容）。
    // Fake/suite 覆盖见 integration tests + `cargo test -p contract-testkit`。

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
            Err(XError::invalid("未实现"))
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
            Err(XError::invalid("未实现"))
        }
        async fn subscribe_orderbook(
            &self,
            _symbol: &str,
        ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
            Err(XError::invalid("未实现"))
        }
        async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
            Err(XError::invalid("未实现"))
        }
        async fn server_time(&self) -> XResult<i64> {
            Ok(0)
        }
        async fn symbol_info(&self, _symbol: &str) -> XResult<SymbolMeta> {
            Err(XError::invalid("未实现"))
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
        assert!(is_default_cancel_order_request_error(&e1));
        assert_eq!(e1.kind(), kernel::ErrorKind::Invalid);
        let e2 = v.query_order_request(&req).await.unwrap_err();
        assert!(is_default_query_order_request_error(&e2));
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

    #[derive(Default)]
    struct StubTx {
        committed: bool,
        rolled_back: bool,
    }

    #[async_trait]
    impl TxContext for StubTx {
        async fn commit(&mut self) -> XResult<()> {
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

    struct StubTxRunner;
    #[async_trait]
    impl TxRunner for StubTxRunner {
        async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
            Ok(Box::new(StubTx::default()))
        }
    }

    #[tokio::test]
    async fn tx_runner_commit_path() {
        let runner = StubTxRunner;
        let out =
            run_tx_commit_on_ok(&runner, |_ctx| async move { Ok(42u32) }).await.expect("commit ok");
        assert_eq!(out, 42);
    }

    #[tokio::test]
    async fn tx_runner_err_triggers_rollback_path() {
        let runner = StubTxRunner;
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
        let runner: &dyn TxRunner = &StubTxRunner;
        let mut ctx = runner.begin_tx().await.unwrap();
        ctx.commit().await.unwrap();
    }

    struct OnceMsg(Option<BusMessage>);
    impl Stream for OnceMsg {
        type Item = BusMessage;
        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(self.0.take())
        }
    }

    struct StubBus;
    #[async_trait]
    impl EventBus for StubBus {
        async fn publish(&self, _topic: &str, _payload: Bytes) -> XResult<()> {
            Ok(())
        }
        async fn subscribe(&self, _topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
            Ok(Box::pin(OnceMsg(Some(BusMessage {
                id: "1".into(),
                payload: Bytes::from_static(b"hi"),
            }))))
        }
    }

    #[tokio::test]
    async fn event_bus_publish_subscribe_message_contract() {
        let bus = StubBus;
        bus.publish("orders", Bytes::from_static(b"hi")).await.unwrap();
        let _stream = bus.subscribe("orders").await.unwrap();
    }

    #[test]
    fn message_ack_and_bus_message_surface() {
        let m = BusMessage { id: "1".into(), payload: Bytes::from_static(b"x") };
        assert_eq!(m.id, "1");
        assert_eq!(MessageAck::Ack, MessageAck::Ack);
        assert_ne!(MessageAck::Ack, MessageAck::Nack);
    }

    #[tokio::test]
    async fn event_bus_stream_poll_clone_waker() {
        let bus = StubBus;
        bus.publish("t", Bytes::from_static(b"p")).await.unwrap();
        let mut stream = bus.subscribe("t").await.unwrap();
        use std::task::Waker;
        let waker = Waker::noop();
        let waker2 = waker.clone();
        let mut cx = Context::from_waker(&waker2);
        match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(Some(msg)) => assert_eq!(msg.payload.as_ref(), b"hi"),
            _ => panic!("expected Some"),
        }
        match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(None) => {}
            _ => panic!("expected None"),
        }
    }
}
