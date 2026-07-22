//! First-batch trait 合同套件：委托 `contract-testkit` suite（禁止空断言）。

use contract_testkit::{
    FakeEventBus, FakeKeyValueStore, FakeRepository, FakeTxRunner, FixtureNamespace,
    RecordingInstrumentation, RecordingTxRunner, assert_event_bus, assert_instrumentation,
    assert_instrumentation_observed, assert_key_value_store, assert_repository, assert_tx_runner,
};
use contracts::run_tx_lifecycle;
use kernel::XError;

#[tokio::test]
async fn key_value_store_trait_get_set() {
    let store = FakeKeyValueStore::new();
    assert_key_value_store(&store).await.expect("kv suite");
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
    assert_repository(
        &repo,
        Row { id: "a".into(), body: "x".into() },
        "a".into(),
        |r| r.body = "y".into(),
        |a, b| a == b,
    )
    .await
    .expect("repo suite");
}

#[tokio::test]
async fn tx_runner_and_context_commit_rollback_paths() {
    assert_tx_runner(&FakeTxRunner).await.expect("fake tx suite");

    let runner = RecordingTxRunner::new();
    let n = run_tx_lifecycle(&runner, || async move { Ok::<_, XError>(11u32) })
        .await
        .expect("commit path");
    assert_eq!(n, 11);
    assert!(*runner.committed.lock().expect("lock"), "必须真实调用 commit");
    assert!(!*runner.rolled_back.lock().expect("lock"));

    let runner = RecordingTxRunner::new();
    let _ =
        run_tx_lifecycle(&runner, || async move { Err::<(), _>(XError::invalid("业务校验失败")) })
            .await;
    assert!(*runner.rolled_back.lock().expect("lock"), "失败路径必须 rollback");
    assert!(!*runner.committed.lock().expect("lock"));
}

#[tokio::test]
async fn event_bus_publish_subscribe_ids() {
    let bus = FakeEventBus::new();
    assert_event_bus(&bus).await.expect("bus suite");
}

#[test]
fn instrumentation_recording_surface() {
    let rec = RecordingInstrumentation::new();
    assert_instrumentation(&rec).expect("instr suite");
    rec.clear().expect("clear smoke events");
    let fixture = FixtureNamespace::new("ctk_contracts_instrumentation").expect("valid fixture");
    assert_instrumentation_observed(&rec, &fixture, || rec.snapshot())
        .expect("observed instr suite");
    assert_eq!(rec.snapshot().expect("snap").len(), 3);
}
