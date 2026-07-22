//! Live profile/handles 与轻量 helper 的公共 fail-closed 合同。

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{
    EventBus, KeyValueStore, LiveContractProfile, LiveHandles, TxContext, TxRunner, bus_publish,
    kv_set_then_commit_separate_resources, publish_without_delivery_attestation, tx_kv_set,
};
use futures_core::stream::BoxStream;
use kernel::{ErrorKind, XError, XResult};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[test]
fn profile_capabilities_without_matching_handle_types_fail_closed() {
    let mut repo = LiveContractProfile::none();
    repo.repo = true;
    let error = LiveHandles::empty(repo).validate().expect_err("repo 不能由当前 handles 证明");
    assert_eq!(error.kind(), ErrorKind::Missing);
    assert!(error.context().contains("repo"));

    let mut account = LiveContractProfile::none();
    account.account = true;
    let error =
        LiveHandles::empty(account).validate().expect_err("account 不能由 venue handle 推断");
    assert!(error.context().contains("account"));

    let mut venue_time = LiveContractProfile::none();
    venue_time.venue_time = true;
    let error =
        LiveHandles::empty(venue_time).validate().expect_err("venue_time 不能由 venue handle 推断");
    assert!(error.context().contains("venue_time"));
}

struct FailingBus;

#[async_trait]
impl EventBus for FailingBus {
    async fn publish(&self, _topic: &str, _payload: Bytes) -> XResult<()> {
        Err(XError::transient("publish failed"))
    }

    async fn subscribe(&self, _topic: &str) -> XResult<BoxStream<'static, contracts::BusMessage>> {
        Err(XError::invalid("本测试不订阅"))
    }
}

#[tokio::test]
async fn publish_helper_propagates_producer_failure_without_claiming_delivery() {
    let error =
        publish_without_delivery_attestation(&FailingBus, "orders", Bytes::from_static(b"payload"))
            .await
            .expect_err("publish failure must propagate");
    assert_eq!(error.kind(), ErrorKind::Transient);
}

struct UnusedKv;

#[async_trait]
impl KeyValueStore for UnusedKv {
    async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
        Err(XError::invalid("begin 失败时不应执行 set"))
    }
}

struct BeginFailRunner;

#[async_trait]
impl TxRunner for BeginFailRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Err(XError::transient("begin failed"))
    }
}

#[tokio::test]
async fn separate_resource_helper_propagates_begin_failure() {
    let error = kv_set_then_commit_separate_resources(
        &BeginFailRunner,
        Arc::new(UnusedKv),
        "k".into(),
        b"v".to_vec(),
    )
    .await
    .expect_err("begin failure must propagate");
    assert_eq!(error.kind(), ErrorKind::Transient);
}

struct RecordingTxContext {
    committed: Arc<AtomicBool>,
    rolled_back: Arc<AtomicBool>,
    fail_commit: bool,
    fail_rollback: bool,
}

#[async_trait]
impl TxContext for RecordingTxContext {
    async fn commit(&mut self) -> XResult<()> {
        self.committed.store(true, Ordering::SeqCst);
        if self.fail_commit { Err(XError::unavailable("commit failed")) } else { Ok(()) }
    }

    async fn rollback(&mut self) -> XResult<()> {
        self.rolled_back.store(true, Ordering::SeqCst);
        if self.fail_rollback { Err(XError::unavailable("rollback failed")) } else { Ok(()) }
    }
}

struct RecordingRunner {
    committed: Arc<AtomicBool>,
    rolled_back: Arc<AtomicBool>,
    fail_commit: bool,
    fail_rollback: bool,
}

#[async_trait]
impl TxRunner for RecordingRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(RecordingTxContext {
            committed: Arc::clone(&self.committed),
            rolled_back: Arc::clone(&self.rolled_back),
            fail_commit: self.fail_commit,
            fail_rollback: self.fail_rollback,
        }))
    }
}

struct SetOutcomeKv {
    called: Arc<AtomicBool>,
    fail: bool,
}

#[async_trait]
impl KeyValueStore for SetOutcomeKv {
    async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
        self.called.store(true, Ordering::SeqCst);
        if self.fail { Err(XError::transient("set failed")) } else { Ok(()) }
    }
}

#[tokio::test]
async fn separate_resource_helper_rolls_back_on_set_failure_and_keeps_primary_error() {
    let committed = Arc::new(AtomicBool::new(false));
    let rolled_back = Arc::new(AtomicBool::new(false));
    let runner = RecordingRunner {
        committed: Arc::clone(&committed),
        rolled_back: Arc::clone(&rolled_back),
        fail_commit: false,
        fail_rollback: true,
    };
    let set_called = Arc::new(AtomicBool::new(false));
    let error = kv_set_then_commit_separate_resources(
        &runner,
        Arc::new(SetOutcomeKv { called: Arc::clone(&set_called), fail: true }),
        "k".into(),
        b"v".to_vec(),
    )
    .await
    .expect_err("set 失败必须传播");

    assert_eq!(error.kind(), ErrorKind::Transient, "rollback 失败不得覆盖原始 set 错误");
    assert!(set_called.load(Ordering::SeqCst));
    assert!(rolled_back.load(Ordering::SeqCst));
    assert!(!committed.load(Ordering::SeqCst));
}

#[tokio::test]
async fn separate_resource_helper_exposes_commit_failure_without_claiming_atomic_undo() {
    let committed = Arc::new(AtomicBool::new(false));
    let rolled_back = Arc::new(AtomicBool::new(false));
    let runner = RecordingRunner {
        committed: Arc::clone(&committed),
        rolled_back: Arc::clone(&rolled_back),
        fail_commit: true,
        fail_rollback: false,
    };
    let set_called = Arc::new(AtomicBool::new(false));
    let error = kv_set_then_commit_separate_resources(
        &runner,
        Arc::new(SetOutcomeKv { called: Arc::clone(&set_called), fail: false }),
        "k".into(),
        b"v".to_vec(),
    )
    .await
    .expect_err("commit 失败必须传播");

    assert_eq!(error.kind(), ErrorKind::Unavailable);
    assert!(set_called.load(Ordering::SeqCst), "独立 KV set 已先完成，不能声称原子撤销");
    assert!(committed.load(Ordering::SeqCst));
    assert!(!rolled_back.load(Ordering::SeqCst));
}

#[tokio::test]
async fn compatibility_aliases_preserve_the_accurate_helpers_error_surface() {
    let publish_error = bus_publish(&FailingBus, "orders", Bytes::from_static(b"payload"))
        .await
        .expect_err("兼容别名必须传播 publish 失败");
    assert_eq!(publish_error.kind(), ErrorKind::Transient);

    let tx_error = tx_kv_set(&BeginFailRunner, Arc::new(UnusedKv), "k".into(), b"v".to_vec())
        .await
        .expect_err("兼容别名必须传播 begin 失败");
    assert_eq!(tx_error.kind(), ErrorKind::Transient);
}
