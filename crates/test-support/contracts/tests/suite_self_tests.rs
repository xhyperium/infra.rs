//! Suite 自测：用本 crate Fake 跑完整 conformance（SPEC-TESTKIT-002 §4.2）。

use contract_testkit::{
    FakeAccountSource, FakeAnalyticsSink, FakeEventBus, FakeExecutionVenue, FakeInstrumentCatalog,
    FakeKeyValueStore, FakeMarketDataSource, FakeObjectStore, FakePubSub, FakeRepository,
    FakeTimeSeriesStore, FakeTxRunner, FakeVenueTimeSource, FixtureNamespace,
    RecordingInstrumentation, RecordingTxRunner, assert_account_source,
    assert_analytics_sink_callable, assert_analytics_sink_observed, assert_event_bus,
    assert_event_bus_with_fixture, assert_execution_venue, assert_instrument_catalog,
    assert_instrumentation, assert_instrumentation_observed, assert_key_value_store,
    assert_key_value_store_isolated, assert_market_data_source, assert_object_store_with_fixture,
    assert_pub_sub_smoke, assert_repository, assert_time_series_store_with_fixture,
    assert_tx_runner, assert_venue_time_source, default_symbol_meta, sample_order,
};
use contracts::{VenueTimeSource, run_tx_commit_on_ok};
use kernel::XError;

#[tokio::test]
async fn suite_key_value_store_on_fake() {
    let store = FakeKeyValueStore::new();
    let fixture = FixtureNamespace::new("ctk_key_value_reference").expect("valid fixture");
    assert_key_value_store(&store).await.expect("legacy kv suite");
    assert_key_value_store_isolated(&store, &fixture).await.expect("isolated kv suite");
    assert_eq!(store.len().expect("len"), 2);
}

#[tokio::test]
async fn suite_event_bus_on_fake() {
    let bus = FakeEventBus::new();
    let fixture = FixtureNamespace::new("ctk_event_bus_reference").expect("valid fixture");
    assert_event_bus(&bus).await.expect("snapshot/replay profile suite");
    assert_event_bus_with_fixture(&bus, &fixture).await.expect("portable bus surface");
}

#[tokio::test]
async fn suite_object_store_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_object_reference").expect("valid fixture");
    assert_object_store_with_fixture(&FakeObjectStore::new(), &fixture)
        .await
        .expect("object store suite");
}

#[tokio::test]
async fn suite_time_series_store_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_time_series_reference").expect("valid fixture");
    assert_time_series_store_with_fixture(&FakeTimeSeriesStore::new(), &fixture)
        .await
        .expect("time series suite");
}

#[tokio::test]
async fn analytics_callable_suite_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_analytics_reference").expect("valid fixture");
    let sink = FakeAnalyticsSink::new();
    assert_analytics_sink_callable(&sink, &fixture).await.expect("analytics callable suite");
    assert_analytics_sink_observed(&sink, &fixture, || sink.events())
        .await
        .expect("analytics observed suite");
    let events = sink.events().expect("fake events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "ctk_analytics_reference__analytics_event");
}

#[tokio::test]
async fn pub_sub_smoke_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_pub_sub_reference").expect("valid fixture");
    assert_pub_sub_smoke(&FakePubSub::new(), &fixture).await.expect("pub sub smoke");
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
    rec.clear().expect("clear smoke events");
    let fixture = FixtureNamespace::new("ctk_instrumentation_reference").expect("valid fixture");
    assert_instrumentation_observed(&rec, &fixture, || rec.snapshot())
        .expect("observed instr suite");
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
