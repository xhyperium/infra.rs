//! Suite 自测：用本 crate Fake 跑完整 conformance（SPEC-TESTKIT-002 §4.2）。

use async_trait::async_trait;
use canonical::Tick;
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
use contracts::{TimeSeriesStore, VenueTimeSource, run_tx_lifecycle};
use kernel::{XError, XResult};
use std::sync::Mutex;

#[derive(Default)]
struct HalfOpenTimeSeriesStore {
    points: Mutex<Vec<Tick>>,
}

#[async_trait]
impl TimeSeriesStore for HalfOpenTimeSeriesStore {
    async fn write_series(&self, _table: &str, points: Vec<Tick>) -> XResult<()> {
        self.points.lock().expect("时序点锁应可用").extend(points);
        Ok(())
    }

    async fn query_series(&self, _table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        Ok(self
            .points
            .lock()
            .expect("时序点锁应可用")
            .iter()
            .filter(|point| point.ts >= start && point.ts < end)
            .cloned()
            .collect())
    }
}

#[tokio::test]
async fn suite_key_value_store_on_fake() {
    let store = FakeKeyValueStore::new();
    let fixture = FixtureNamespace::new("ctk_key_value_reference").expect("fixture 应合法");
    assert_key_value_store(&store).await.expect("兼容 KV suite 应通过");
    assert_key_value_store_isolated(&store, &fixture).await.expect("隔离 KV suite 应通过");
    assert_eq!(store.len().expect("应读取长度"), 2);
}

#[tokio::test]
async fn suite_event_bus_on_fake() {
    let bus = FakeEventBus::new();
    let fixture = FixtureNamespace::new("ctk_event_bus_reference").expect("fixture 应合法");
    assert_event_bus(&bus).await.expect("快照回放 profile suite 应通过");
    assert_event_bus_with_fixture(&bus, &fixture).await.expect("可移植总线 surface 应通过");
}

#[tokio::test]
async fn suite_object_store_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_object_reference").expect("fixture 应合法");
    assert_object_store_with_fixture(&FakeObjectStore::new(), &fixture)
        .await
        .expect("对象存储 suite 应通过");
}

#[tokio::test]
async fn suite_time_series_store_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_time_series_reference").expect("fixture 应合法");
    assert_time_series_store_with_fixture(&FakeTimeSeriesStore::new(), &fixture)
        .await
        .expect("时序存储 suite 应通过");

    let half_open = HalfOpenTimeSeriesStore::default();
    assert_time_series_store_with_fixture(&half_open, &fixture)
        .await
        .expect("半开区间后端也应通过可移植窗口 suite");
}

#[tokio::test]
async fn profile_modules_separate_portable_window_from_closed_point() {
    let fixture = FixtureNamespace::new("ctk_profile_modules").expect("fixture 应合法");
    contract_testkit::portable::assert_time_series_store_with_fixture(
        &HalfOpenTimeSeriesStore::default(),
        &fixture,
    )
    .await
    .expect("portable profile 必须接受半开区间后端");

    let _portable_window = contract_testkit::portable::assert_time_series_store_in_window;
    let _closed_point = contract_testkit::closed_point::assert_time_series_store;
}

#[tokio::test]
async fn analytics_callable_suite_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_analytics_reference").expect("fixture 应合法");
    let sink = FakeAnalyticsSink::new();
    assert_analytics_sink_callable(&sink, &fixture).await.expect("分析写入可调用 suite 应通过");
    assert_analytics_sink_observed(&sink, &fixture, || sink.events())
        .await
        .expect("分析写入 observed suite 应通过");
    let events = sink.events().expect("应读取 Fake 事件");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0, "ctk_analytics_reference__analytics_event");
}

#[tokio::test]
async fn pub_sub_smoke_on_reference_fake() {
    let fixture = FixtureNamespace::new("ctk_pub_sub_reference").expect("fixture 应合法");
    assert_pub_sub_smoke(&FakePubSub::new(), &fixture).await.expect("发布订阅 smoke 应通过");
}

#[tokio::test]
async fn suite_tx_runner_on_fake_and_recording() {
    assert_tx_runner(&FakeTxRunner).await.expect("Fake 事务 suite 应通过");

    let rec = RecordingTxRunner::new();
    let n = run_tx_lifecycle(&rec, || async move { Ok::<_, XError>(1u8) })
        .await
        .expect("记录事务应提交");
    assert_eq!(n, 1);
    assert!(*rec.committed.lock().expect("提交锁应可用"));
    assert!(!*rec.rolled_back.lock().expect("回滚锁应可用"));

    let rec = RecordingTxRunner::new();
    let _ =
        run_tx_lifecycle(&rec, || async move { Err::<(), _>(XError::invalid("业务失败")) }).await;
    assert!(*rec.rolled_back.lock().expect("回滚锁应可用"));
    assert!(!*rec.committed.lock().expect("提交锁应可用"));
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
    .expect("仓储 suite 应通过");
}

#[test]
fn suite_instrumentation_on_recording() {
    let rec = RecordingInstrumentation::new();
    assert_instrumentation(&rec).expect("可观测 smoke 应通过");
    rec.clear().expect("应清除 smoke 事件");
    let fixture = FixtureNamespace::new("ctk_instrumentation_reference").expect("fixture 应合法");
    assert_instrumentation_observed(&rec, &fixture, || rec.snapshot())
        .expect("可观测 observed suite 应通过");
    let snap = rec.snapshot().expect("应读取快照");
    assert_eq!(snap.len(), 3);
}

#[tokio::test]
async fn suite_market_data_source_on_fake() {
    let src = FakeMarketDataSource;
    assert_market_data_source(&src, "BTCUSDT").await.expect("行情源 suite 应通过");
}

#[tokio::test]
async fn suite_instrument_catalog_on_fake() {
    let cat = FakeInstrumentCatalog::new().with_symbol(default_symbol_meta("BTCUSDT"));
    assert_instrument_catalog(&cat, "BTCUSDT").await.expect("品种目录 suite 应通过");
}

#[tokio::test]
async fn suite_execution_venue_on_fake() {
    let venue = FakeExecutionVenue::new("mock");
    let order = sample_order("1", "BTCUSDT");
    assert_execution_venue(&venue, &order).await.expect("执行场所 suite 应通过");
    assert_eq!(venue.last_order().expect("应记录最后订单").id, "1");
}

#[tokio::test]
async fn suite_account_and_time_on_fake() {
    let acct = FakeAccountSource::new();
    assert_account_source(&acct).await.expect("账户源 suite 应通过");
    let time = FakeVenueTimeSource::new(1_700_000_000_000_000_000);
    assert_venue_time_source(&time).await.expect("场所时间源 suite 应通过");
    assert_eq!(time.server_time().await.expect("应读取服务器时间"), 1_700_000_000_000_000_000);
}
