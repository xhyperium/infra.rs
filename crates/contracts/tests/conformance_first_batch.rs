//! First-batch trait 合同套件：驱动真实 trait 方法（禁止空断言）。

use bytes::Bytes;
use contracts::{
    EventBus, FakeEventBus, FakeKeyValueStore, FakeRepository, FakeTxRunner, InstrEvent,
    Instrumentation, KeyValueStore, RecordingInstrumentation, RecordingTxRunner, Repository,
    TxRunner, run_tx_commit_on_ok,
};
use futures_core::Stream;
use kernel::{ErrorKind, XError};
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

fn dummy_waker() -> Waker {
    fn no(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw()
    }
    fn dummy_raw() -> RawWaker {
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no, no, no);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe { Waker::from_raw(dummy_raw()) }
}

#[tokio::test]
async fn key_value_store_trait_get_set() {
    let store = FakeKeyValueStore::new();
    let kv: &dyn KeyValueStore = &store;
    assert!(kv.get("missing").await.expect("get").is_none());
    kv.set("k", b"hello".to_vec(), Some(Duration::from_secs(30))).await.expect("set");
    let v = kv.get("k").await.expect("get2").expect("hit");
    assert_eq!(v, b"hello");
    assert_eq!(store.len().expect("len"), 1);
}

#[tokio::test]
async fn repository_trait_save_find() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Row {
        id: String,
        body: String,
    }
    let repo = FakeRepository::new(|r: &Row| r.id.clone());
    let r: &dyn Repository<Row, String> = &repo;
    assert!(r.find("a".into()).await.expect("find").is_none());
    r.save(&Row { id: "a".into(), body: "x".into() }).await.expect("save");
    let got = r.find("a".into()).await.expect("find2").expect("row");
    assert_eq!(got.body, "x");
    // upsert
    r.save(&Row { id: "a".into(), body: "y".into() }).await.expect("upsert");
    assert_eq!(r.find("a".into()).await.unwrap().unwrap().body, "y");
}

#[tokio::test]
async fn tx_runner_and_context_commit_rollback_paths() {
    let runner = RecordingTxRunner::new();
    let dyn_runner: &dyn TxRunner = &runner;
    let n = run_tx_commit_on_ok(dyn_runner, |_ctx| async move { Ok::<_, XError>(11u32) })
        .await
        .expect("commit path");
    assert_eq!(n, 11);
    assert!(*runner.committed.lock().expect("lock"), "必须真实调用 commit");
    assert!(!*runner.rolled_back.lock().expect("lock"));

    let runner = RecordingTxRunner::new();
    let err = run_tx_commit_on_ok(&runner, |_ctx| async move {
        Err::<(), _>(XError::invalid("业务校验失败"))
    })
    .await
    .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.context().contains("业务校验失败"));
    assert!(*runner.rolled_back.lock().expect("lock"), "失败路径必须 rollback");
    assert!(!*runner.committed.lock().expect("lock"));

    // FakeTxRunner 对象安全 begin_tx
    let simple: &dyn TxRunner = &FakeTxRunner;
    let mut ctx = simple.begin_tx().await.expect("begin");
    ctx.commit().await.expect("commit");
}

#[tokio::test]
async fn event_bus_publish_subscribe_ids() {
    let bus = FakeEventBus::new();
    let dyn_bus: &dyn EventBus = &bus;
    dyn_bus.publish("orders", Bytes::from_static(b"o1")).await.expect("pub1");
    dyn_bus.publish("orders", Bytes::from_static(b"o2")).await.expect("pub2");
    let mut stream = dyn_bus.subscribe("orders").await.expect("sub");
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let m1 = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(m)) => m,
        other => panic!("expected first message: {other:?}"),
    };
    let m2 = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(m)) => m,
        other => panic!("expected second message: {other:?}"),
    };
    assert_ne!(m1.id, m2.id, "消息 id 应递增区分");
    assert_eq!(m1.payload.as_ref(), b"o1");
    assert_eq!(m2.payload.as_ref(), b"o2");
    assert!(matches!(Pin::new(&mut stream).poll_next(&mut cx), Poll::Ready(None)));
}

#[test]
fn instrumentation_recording_trait_path() {
    let rec = RecordingInstrumentation::new();
    let instr: &dyn Instrumentation = &rec;
    instr.record_retry("cancel_order", 2);
    instr.record_circuit_open("cancel_order");
    instr.record_circuit_close("cancel_order");
    let snap = rec.snapshot().expect("snap");
    assert_eq!(snap.len(), 3);
    assert_eq!(snap[0], InstrEvent::Retry { op: "cancel_order".into(), attempt: 2 });
    assert_eq!(snap[1], InstrEvent::CircuitOpen { op: "cancel_order".into() });
    assert_eq!(snap[2], InstrEvent::CircuitClose { op: "cancel_order".into() });
    rec.clear().expect("clear");
    assert!(rec.snapshot().unwrap().is_empty());
}
