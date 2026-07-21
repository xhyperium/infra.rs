//! 最小 contract-testkit：进程内 Fake / Recording，驱动真实 trait 路径。
//!
//! 能力边界：内存参考实现，**不**模拟真实 DB / 网络 / 交易所。
//! 用户可见错误信息使用中文（宪章 §4.5）。

use crate::{
    BusMessage, EventBus, Instrumentation, KeyValueStore, Repository, TxContext, TxRunner,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ── Tx ────────────────────────────────────

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

    /// 注入 commit 失败（`ErrorKind::Transient`）。
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

/// 内存 [`TxRunner`] 参考实现。
#[derive(Debug, Default)]
pub struct FakeTxRunner;

#[async_trait]
impl TxRunner for FakeTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(FakeTxContext::new()))
    }
}

/// 可观察 commit/rollback 标志的 runner（合同测：证明编排真正驱动 [`TxContext`]）。
#[derive(Debug, Clone)]
pub struct RecordingTxRunner {
    /// 最近一次上下文是否已 commit。
    pub committed: Arc<Mutex<bool>>,
    /// 最近一次上下文是否已 rollback。
    pub rolled_back: Arc<Mutex<bool>>,
}

impl RecordingTxRunner {
    /// 新建，标志初始为 `false`。
    pub fn new() -> Self {
        Self { committed: Arc::new(Mutex::new(false)), rolled_back: Arc::new(Mutex::new(false)) }
    }
}

impl Default for RecordingTxRunner {
    fn default() -> Self {
        Self::new()
    }
}

struct RecordingTxContext {
    inner: FakeTxContext,
    committed: Arc<Mutex<bool>>,
    rolled_back: Arc<Mutex<bool>>,
}

#[async_trait]
impl TxContext for RecordingTxContext {
    async fn commit(&mut self) -> XResult<()> {
        self.inner.commit().await?;
        *self.committed.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.committed;
        *self.rolled_back.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.rolled_back;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        self.inner.rollback().await?;
        *self.committed.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.committed;
        *self.rolled_back.lock().map_err(|_| XError::internal("recording lock 中毒"))? =
            self.inner.rolled_back;
        Ok(())
    }
}

#[async_trait]
impl TxRunner for RecordingTxRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(RecordingTxContext {
            inner: FakeTxContext::new(),
            committed: Arc::clone(&self.committed),
            rolled_back: Arc::clone(&self.rolled_back),
        }))
    }
}

// ── EventBus ──────────────────────────────

/// 内存 EventBus 参考实现（at-most-once 进程内）。
#[derive(Debug, Default)]
pub struct FakeEventBus {
    inner: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
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
        let id = self.seq.fetch_add(1, Ordering::Relaxed).to_string();
        let mut g = self.inner.lock().map_err(|_| XError::internal("event bus lock 中毒"))?;
        g.entry(topic.to_string()).or_default().push(BusMessage { id, payload });
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = {
            let g = self.inner.lock().map_err(|_| XError::internal("event bus lock 中毒"))?;
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

// ── KeyValueStore ─────────────────────────

/// 内存 [`KeyValueStore`] 参考实现。
///
/// TTL：仅记录；本 fake **不**自动过期（最小面）。
#[derive(Debug, Default)]
pub struct FakeKeyValueStore {
    inner: Mutex<HashMap<String, (Vec<u8>, Option<Duration>)>>,
}

impl FakeKeyValueStore {
    /// 新建空存储。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前条目数（测试辅助）。
    pub fn len(&self) -> XResult<usize> {
        let g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        Ok(g.len())
    }

    /// 是否为空（测试辅助）。
    pub fn is_empty(&self) -> XResult<bool> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl KeyValueStore for FakeKeyValueStore {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        Ok(g.get(key).map(|(v, _)| v.clone()))
    }

    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        let mut g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        g.insert(key.to_string(), (val, ttl));
        Ok(())
    }
}

// ── Repository ────────────────────────────

/// 简单内存 [`Repository`]（`Id: Eq + Hash + Clone + Send + Sync`，`T: Clone + Send + Sync`）。
///
/// `save` 要求调用方提供 `id_of` 提取函数（构造时注入），避免强制 `T` 携带 id 字段。
pub struct FakeRepository<T, Id> {
    inner: Mutex<HashMap<Id, T>>,
    id_of: Box<dyn Fn(&T) -> Id + Send + Sync>,
}

impl<T, Id> FakeRepository<T, Id>
where
    T: Clone + Send + Sync + 'static,
    Id: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    /// 以 `id_of` 提取主键。
    pub fn new<F>(id_of: F) -> Self
    where
        F: Fn(&T) -> Id + Send + Sync + 'static,
    {
        Self { inner: Mutex::new(HashMap::new()), id_of: Box::new(id_of) }
    }

    /// 当前实体数。
    pub fn len(&self) -> XResult<usize> {
        let g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        Ok(g.len())
    }

    /// 是否为空。
    pub fn is_empty(&self) -> XResult<bool> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl<T, Id> Repository<T, Id> for FakeRepository<T, Id>
where
    T: Clone + Send + Sync + 'static,
    Id: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
{
    async fn find(&self, id: Id) -> XResult<Option<T>> {
        let g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        Ok(g.get(&id).cloned())
    }

    async fn save(&self, entity: &T) -> XResult<()> {
        let id = (self.id_of)(entity);
        let mut g = self.inner.lock().map_err(|_| XError::internal("repository lock 中毒"))?;
        g.insert(id, entity.clone());
        Ok(())
    }
}

// ── Instrumentation ───────────────────────

/// 单条可观测记录（Recording）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstrEvent {
    /// 重试。
    Retry { op: String, attempt: u32 },
    /// 熔断打开。
    CircuitOpen { op: String },
    /// 熔断关闭。
    CircuitClose { op: String },
}

/// 可观察的 [`Instrumentation`]：将调用写入内存向量。
#[derive(Debug, Default, Clone)]
pub struct RecordingInstrumentation {
    events: Arc<Mutex<Vec<InstrEvent>>>,
}

impl RecordingInstrumentation {
    /// 新建空记录器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 已记录事件的快照。
    pub fn snapshot(&self) -> XResult<Vec<InstrEvent>> {
        let g = self.events.lock().map_err(|_| XError::internal("instr lock 中毒"))?;
        Ok(g.clone())
    }

    /// 清空记录。
    pub fn clear(&self) -> XResult<()> {
        let mut g = self.events.lock().map_err(|_| XError::internal("instr lock 中毒"))?;
        g.clear();
        Ok(())
    }
}

impl Instrumentation for RecordingInstrumentation {
    fn record_retry(&self, op: &str, attempt: u32) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::Retry { op: op.to_string(), attempt });
        }
    }

    fn record_circuit_open(&self, op: &str) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::CircuitOpen { op: op.to_string() });
        }
    }

    fn record_circuit_close(&self, op: &str) {
        if let Ok(mut g) = self.events.lock() {
            g.push(InstrEvent::CircuitClose { op: op.to_string() });
        }
    }
}

// ── Venue override 门禁辅助 ───────────────

/// [`crate::VenueAdapter::cancel_order_request`] 未覆盖时的默认中文错误上下文。
pub const VENUE_CANCEL_REQUEST_DEFAULT_MSG: &str =
    "cancel_order_request 未实现；请覆盖 VenueAdapter::cancel_order_request（CAN-ID）";

/// [`crate::VenueAdapter::query_order_request`] 未覆盖时的默认中文错误上下文。
pub const VENUE_QUERY_REQUEST_DEFAULT_MSG: &str =
    "query_order_request 未实现；请覆盖 VenueAdapter::query_order_request（CAN-ID）";

/// 判断是否为 additive default 的 cancel 未实现错误。
pub fn is_default_cancel_order_request_error(err: &XError) -> bool {
    err.kind() == kernel::ErrorKind::Invalid
        && err.context().contains("cancel_order_request 未实现")
}

/// 判断是否为 additive default 的 query 未实现错误。
pub fn is_default_query_order_request_error(err: &XError) -> bool {
    err.kind() == kernel::ErrorKind::Invalid && err.context().contains("query_order_request 未实现")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run_tx_commit_on_ok;
    use futures_core::Stream;
    use std::pin::Pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

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

    #[tokio::test]
    async fn recording_tx_runner_commit_and_rollback() {
        let runner = RecordingTxRunner::new();
        let out = run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, XError>(7u8) })
            .await
            .expect("ok");
        assert_eq!(out, 7);
        assert!(*runner.committed.lock().expect("lock"));
        assert!(!*runner.rolled_back.lock().expect("lock"));

        let runner = RecordingTxRunner::new();
        let err = run_tx_commit_on_ok(&runner, |_ctx| async move {
            Err::<(), _>(XError::invalid("业务失败"))
        })
        .await
        .unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(*runner.rolled_back.lock().expect("lock"));
        assert!(!*runner.committed.lock().expect("lock"));
    }

    #[tokio::test]
    async fn fake_kv_get_set_roundtrip() {
        let kv = FakeKeyValueStore::new();
        assert!(kv.get("k").await.unwrap().is_none());
        kv.set("k", b"v".to_vec(), Some(Duration::from_secs(1))).await.unwrap();
        assert_eq!(kv.get("k").await.unwrap().as_deref(), Some(b"v".as_ref()));
        assert_eq!(kv.len().unwrap(), 1);
    }

    #[tokio::test]
    async fn fake_repository_save_find() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Entity {
            id: u64,
            name: String,
        }
        let repo = FakeRepository::new(|e: &Entity| e.id);
        assert!(repo.find(1).await.unwrap().is_none());
        repo.save(&Entity { id: 1, name: "a".into() }).await.unwrap();
        let got = repo.find(1).await.unwrap().expect("found");
        assert_eq!(got.name, "a");
        assert_eq!(repo.len().unwrap(), 1);
    }

    #[test]
    fn recording_instrumentation_records_events() {
        let instr = RecordingInstrumentation::new();
        let d: &dyn Instrumentation = &instr;
        d.record_retry("place", 1);
        d.record_circuit_open("place");
        d.record_circuit_close("place");
        let snap = instr.snapshot().unwrap();
        assert_eq!(
            snap,
            vec![
                InstrEvent::Retry { op: "place".into(), attempt: 1 },
                InstrEvent::CircuitOpen { op: "place".into() },
                InstrEvent::CircuitClose { op: "place".into() },
            ]
        );
    }

    #[tokio::test]
    async fn fake_event_bus_publish_subscribe() {
        let bus = FakeEventBus::new();
        bus.publish("t", Bytes::from_static(b"p")).await.unwrap();
        let mut stream = bus.subscribe("t").await.unwrap();
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
                assert!(!msg.id.is_empty());
                assert_eq!(msg.payload.as_ref(), b"p");
            }
            _ => panic!("expected message"),
        }
    }

    #[test]
    fn default_venue_error_helpers() {
        let e = XError::invalid(VENUE_CANCEL_REQUEST_DEFAULT_MSG);
        assert!(is_default_cancel_order_request_error(&e));
        assert!(!is_default_query_order_request_error(&e));
        let e2 = XError::invalid(VENUE_QUERY_REQUEST_DEFAULT_MSG);
        assert!(is_default_query_order_request_error(&e2));
        let e3 = XError::unavailable("未连接");
        assert!(!is_default_cancel_order_request_error(&e3));
    }

    #[tokio::test]
    async fn fake_event_bus_poison_returns_internal() {
        let bus = FakeEventBus::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = bus.inner.lock().expect("lock");
            panic!("poison bus");
        }));
        match bus.publish("t", Bytes::from_static(b"x")).await {
            Err(e) => assert_eq!(e.kind(), kernel::ErrorKind::Internal),
            Ok(()) => panic!("publish 应在 lock 中毒后失败"),
        }
        match bus.subscribe("t").await {
            Err(e) => assert_eq!(e.kind(), kernel::ErrorKind::Internal),
            Ok(_) => panic!("subscribe 应在 lock 中毒后失败"),
        }
    }
}
