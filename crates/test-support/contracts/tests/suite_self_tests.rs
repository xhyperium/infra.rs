//! Suite 自测：用本 crate Fake 跑完整 conformance（SPEC-TESTKIT-002 §4.2）。

use contract_testkit::{
    FakeAccountSource, FakeEventBus, FakeExecutionVenue, FakeInstrumentCatalog, FakeKeyValueStore,
    FakeMarketDataSource, FakeRepository, FakeTxRunner, FakeVenueTimeSource,
    RecordingInstrumentation, RecordingTxRunner, assert_account_source, assert_event_bus,
    assert_execution_venue, assert_instrument_catalog, assert_instrumentation,
    assert_key_value_store, assert_market_data_source, assert_repository, assert_tx_runner,
    assert_venue_time_source, default_symbol_meta, sample_order,
};
use contracts::{VenueTimeSource, run_tx_commit_on_ok};
use kernel::XError;

#[tokio::test]
async fn suite_key_value_store_on_fake() {
    let store = FakeKeyValueStore::new();
    assert_key_value_store(&store).await.expect("kv suite");
    assert_eq!(store.len().expect("len"), 1);
}

#[tokio::test]
async fn suite_event_bus_on_fake() {
    let bus = FakeEventBus::new();
    assert_event_bus(&bus).await.expect("bus suite");
}

#[tokio::test]
async fn suite_tx_runner_on_fake_and_recording() {
    assert_tx_runner(&FakeTxRunner).await.expect("fake tx");

    let rec = RecordingTxRunner::new();
    let n = run_tx_commit_on_ok(&rec, |_ctx| async move { Ok::<_, XError>(1u8) })
        .await
        .expect("rec commit");
    assert_eq!(n, 1);
    assert!(*rec.committed.lock().expect("lock"));
    assert!(!*rec.rolled_back.lock().expect("lock"));

    let rec = RecordingTxRunner::new();
    let _ =
        run_tx_commit_on_ok(&rec, |_ctx| async move { Err::<(), _>(XError::invalid("业务失败")) })
            .await;
    assert!(*rec.rolled_back.lock().expect("lock"));
    assert!(!*rec.committed.lock().expect("lock"));
}

#[tokio::test]
async fn suite_repository_on_fake() {
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

#[test]
fn suite_instrumentation_on_recording() {
    let rec = RecordingInstrumentation::new();
    assert_instrumentation(&rec).expect("instr suite");
    let snap = rec.snapshot().expect("snap");
    assert_eq!(snap.len(), 3);
}

#[tokio::test]
async fn suite_market_data_source_on_fake() {
    let src = FakeMarketDataSource;
    assert_market_data_source(&src, "BTCUSDT").await.expect("mds suite");
}

#[tokio::test]
async fn suite_instrument_catalog_on_fake() {
    let cat = FakeInstrumentCatalog::new().with_symbol(default_symbol_meta("BTCUSDT"));
    assert_instrument_catalog(&cat, "BTCUSDT").await.expect("catalog suite");
}

#[tokio::test]
async fn suite_execution_venue_on_fake() {
    let venue = FakeExecutionVenue::new("mock");
    let order = sample_order("1", "BTCUSDT");
    assert_execution_venue(&venue, &order).await.expect("exec suite");
    assert_eq!(venue.last_order().expect("last").id, "1");
}

#[tokio::test]
async fn suite_account_and_time_on_fake() {
    let acct = FakeAccountSource::new();
    assert_account_source(&acct).await.expect("account suite");
    let time = FakeVenueTimeSource::new(1_700_000_000_000_000_000);
    assert_venue_time_source(&time).await.expect("time suite");
    assert_eq!(time.server_time().await.expect("t"), 1_700_000_000_000_000_000);
}
