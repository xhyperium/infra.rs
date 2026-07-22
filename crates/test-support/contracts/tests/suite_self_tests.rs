//! Suite 自测：用本 crate Fake 跑完整 conformance（SPEC-TESTKIT-002 §4.2）。

use async_trait::async_trait;
use bytes::Bytes;
use canonical::Tick;
use contract_testkit::{
    FakeAccountSource, FakeAnalyticsSink, FakeEventBus, FakeExecutionVenue, FakeInstrumentCatalog,
    FakeKeyValueStore, FakeMarketDataSource, FakeObjectStore, FakePubSub, FakeRepository,
    FakeTimeSeriesStore, FakeTxRunner, FakeVenueTimeSource, RecordingInstrumentation,
    RecordingTxRunner, assert_account_source, assert_analytics_sink, assert_event_bus,
    assert_event_bus_surface, assert_execution_venue, assert_instrument_catalog,
    assert_instrumentation, assert_key_value_store, assert_market_data_source, assert_object_store,
    assert_pub_sub_surface, assert_repository, assert_time_series_store, assert_tx_runner,
    assert_venue_time_source, default_symbol_meta, sample_order,
};
use contracts::{
    AnalyticsSink, BusMessage, ObjectStore, PubSub, TimeSeriesStore, VenueTimeSource,
    run_tx_lifecycle,
};
use decimalx::{Decimal, Price};
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use kernel::XError;

struct BrokenBatch2;

#[async_trait]
impl ObjectStore for BrokenBatch2 {
    async fn put_object(&self, _key: &str, _data: Bytes) -> kernel::XResult<()> {
        Ok(())
    }

    async fn get_object(&self, _key: &str) -> kernel::XResult<Bytes> {
        Ok(Bytes::from_static(b"wrong"))
    }
}

#[async_trait]
impl TimeSeriesStore for BrokenBatch2 {
    async fn write_series(&self, _table: &str, _points: Vec<Tick>) -> kernel::XResult<()> {
        Ok(())
    }

    async fn query_series(
        &self,
        _table: &str,
        _start: i64,
        _end: i64,
    ) -> kernel::XResult<Vec<Tick>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl AnalyticsSink for BrokenBatch2 {
    async fn sink(&self, _event: &str, _payload: Bytes) -> kernel::XResult<()> {
        Err(XError::unavailable("分析后端不可用"))
    }
}

#[async_trait]
impl PubSub for BrokenBatch2 {
    async fn pub_message(&self, _channel: &str, _msg: Bytes) -> kernel::XResult<()> {
        Err(XError::unavailable("发布后端不可用"))
    }

    async fn sub_channel(&self, _channel: &str) -> kernel::XResult<BoxStream<'static, BusMessage>> {
        Err(XError::unavailable("订阅后端不可用"))
    }
}

#[tokio::test]
async fn suite_key_value_store_on_fake() {
    let store = FakeKeyValueStore::new();
    assert_key_value_store(&store).await.expect("kv suite");
    assert_eq!(store.len().expect("len"), 1);
}

#[tokio::test]
async fn suite_event_bus_snapshot_and_portable_surface_on_fake() {
    let bus = FakeEventBus::new();
    assert_event_bus(&bus).await.expect("事件总线快照 suite");
    assert_event_bus_surface(&bus, "surface-orders", Bytes::from_static(b"o3"))
        .await
        .expect("事件总线 surface suite");
}

#[tokio::test]
async fn suite_batch2_portable_core_on_fakes() {
    let object = FakeObjectStore::new();
    assert_object_store(&object, "case-object", Bytes::from_static(b"payload"))
        .await
        .expect("对象存储 suite");

    let series = FakeTimeSeriesStore::new();
    let tick = Tick {
        symbol: "BTCUSDT".into(),
        bid: Price::new(Decimal::new(10_000, 2)),
        ask: Price::new(Decimal::new(10_100, 2)),
        ts: 1_700_000_000_000_000_000,
    };
    assert_time_series_store(&series, "case_series", tick).await.expect("时序存储 suite");

    let analytics = FakeAnalyticsSink::new();
    assert_analytics_sink(&analytics, "case-event", Bytes::from_static(b"analytics"))
        .await
        .expect("分析写入 suite");
    assert_eq!(analytics.events().expect("读取分析事件").len(), 1);

    let pub_sub = FakePubSub::new();
    assert_pub_sub_surface(&pub_sub, "case-channel", Bytes::from_static(b"message"))
        .await
        .expect("发布订阅 surface suite");
    let mut published = pub_sub.sub_channel("case-channel").await.expect("读取已发布消息");
    let message = published.next().await.expect("应记录一条已发布消息");
    assert_eq!(message.payload, Bytes::from_static(b"message"));
}

#[tokio::test]
async fn suite_batch2_rejects_empty_identifiers_and_backend_failures() {
    let object = FakeObjectStore::new();
    assert!(assert_object_store(&object, "", Bytes::from_static(b"x")).await.is_err());

    let series = FakeTimeSeriesStore::new();
    let tick = Tick {
        symbol: "BTCUSDT".into(),
        bid: Price::new(Decimal::new(10_000, 2)),
        ask: Price::new(Decimal::new(10_100, 2)),
        ts: 1_700_000_000_000_000_000,
    };
    assert!(assert_time_series_store(&series, "", tick.clone()).await.is_err());

    let analytics = FakeAnalyticsSink::new();
    assert!(assert_analytics_sink(&analytics, "", Bytes::from_static(b"x")).await.is_err());
    let pub_sub = FakePubSub::new();
    assert!(assert_pub_sub_surface(&pub_sub, "", Bytes::from_static(b"x")).await.is_err());

    assert!(
        assert_object_store(&BrokenBatch2, "broken-object", Bytes::from_static(b"expected"))
            .await
            .is_err()
    );
    assert!(assert_time_series_store(&BrokenBatch2, "broken_series", tick).await.is_err());
    assert!(
        assert_analytics_sink(&BrokenBatch2, "broken-event", Bytes::from_static(b"x"))
            .await
            .is_err()
    );
    assert!(
        assert_pub_sub_surface(&BrokenBatch2, "broken-channel", Bytes::from_static(b"x"))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn suite_tx_runner_on_fake_and_recording() {
    assert_tx_runner(&FakeTxRunner).await.expect("fake tx");

    let rec = RecordingTxRunner::new();
    let n = run_tx_lifecycle(&rec, || async move { Ok::<_, XError>(1u8) })
        .await
        .expect("记录事务提交成功");
    assert_eq!(n, 1);
    assert!(*rec.committed.lock().expect("lock"));
    assert!(!*rec.rolled_back.lock().expect("lock"));

    let rec = RecordingTxRunner::new();
    let _ =
        run_tx_lifecycle(&rec, || async move { Err::<(), _>(XError::invalid("业务失败")) }).await;
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
