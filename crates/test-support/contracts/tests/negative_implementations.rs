//! 故意违反合同的实现必须被公开 conformance suite 拒绝。

use async_trait::async_trait;
use bytes::Bytes;
use canonical::{
    CancelOrderRequest, Money, Order, OrderAck, OrderBookSnapshot, OrderStatus, Position,
    SymbolMeta, Tick, Trade, VenueId,
};
use contract_testkit::{
    FixtureNamespace, assert_analytics_sink_callable, assert_analytics_sink_observed,
    assert_event_bus_with_fixture, assert_key_value_store_isolated,
    assert_object_store_with_fixture, assert_pub_sub_smoke, assert_time_series_store_with_fixture,
};
use contracts::{
    AccountSource, AnalyticsSink, BusMessage, EventBus, ExecutionVenue, InstrumentCatalog,
    Instrumentation, MarketDataSource, ObjectStore, PubSub, Repository, TimeSeriesStore, TxContext,
    TxRunner, VenueTimeSource,
};
use futures_core::stream::BoxStream;
use kernel::{XError, XResult};
use std::time::Duration;

struct CorruptingObjectStore;

#[async_trait]
impl ObjectStore for CorruptingObjectStore {
    async fn put_object(&self, _key: &str, _data: Bytes) -> XResult<()> {
        Ok(())
    }

    async fn get_object(&self, _key: &str) -> XResult<Bytes> {
        Ok(Bytes::from_static(b"corrupted"))
    }
}

#[tokio::test]
async fn object_store_suite_rejects_corrupted_roundtrip() {
    let fixture = FixtureNamespace::new("ctk_object_broken").expect("valid fixture");
    let failure = assert_object_store_with_fixture(&CorruptingObjectStore, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "ObjectStore");
    assert_eq!(failure.case, "roundtrip");
}

struct DroppingTimeSeriesStore;

#[async_trait]
impl TimeSeriesStore for DroppingTimeSeriesStore {
    async fn write_series(&self, _table: &str, _points: Vec<Tick>) -> XResult<()> {
        Ok(())
    }

    async fn query_series(&self, _table: &str, _start: i64, _end: i64) -> XResult<Vec<Tick>> {
        Ok(Vec::new())
    }
}

#[tokio::test]
async fn time_series_suite_rejects_dropped_write() {
    let fixture = FixtureNamespace::new("ctk_time_series_broken").expect("valid fixture");
    let failure = assert_time_series_store_with_fixture(&DroppingTimeSeriesStore, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "TimeSeriesStore");
    assert_eq!(failure.case, "read_after_write");
}

struct RejectingAnalyticsSink;

#[async_trait]
impl AnalyticsSink for RejectingAnalyticsSink {
    async fn sink(&self, _event: &str, _payload: Bytes) -> XResult<()> {
        Err(XError::unavailable("分析写入不可用"))
    }
}

#[tokio::test]
async fn analytics_callable_suite_rejects_sink_error() {
    let fixture = FixtureNamespace::new("ctk_analytics_broken").expect("valid fixture");
    let failure = assert_analytics_sink_callable(&RejectingAnalyticsSink, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "AnalyticsSink");
    assert_eq!(failure.case, "sink_callable");
}

struct DroppingAnalyticsSink;

#[async_trait]
impl AnalyticsSink for DroppingAnalyticsSink {
    async fn sink(&self, _event: &str, _payload: Bytes) -> XResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn analytics_observed_suite_rejects_dropped_event() {
    let fixture = FixtureNamespace::new("ctk_analytics_dropped").expect("valid fixture");
    let failure =
        assert_analytics_sink_observed(&DroppingAnalyticsSink, &fixture, || Ok(Vec::new()))
            .await
            .expect_err("silent drop must fail observed suite");

    assert_eq!(failure.contract, "AnalyticsSink");
    assert_eq!(failure.case, "observed_event_missing");
}

struct RejectingPubSub;

#[async_trait]
impl PubSub for RejectingPubSub {
    async fn pub_message(&self, _channel: &str, _msg: Bytes) -> XResult<()> {
        Err(XError::unavailable("发布订阅不可用"))
    }

    async fn sub_channel(&self, _channel: &str) -> XResult<BoxStream<'static, BusMessage>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

#[tokio::test]
async fn pub_sub_smoke_rejects_publish_error() {
    let fixture = FixtureNamespace::new("ctk_pub_sub_broken").expect("valid fixture");
    let failure = assert_pub_sub_smoke(&RejectingPubSub, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "PubSub");
    assert_eq!(failure.case, "publish");
}

struct RejectingEventBus;

#[async_trait]
impl EventBus for RejectingEventBus {
    async fn publish(&self, _topic: &str, _payload: Bytes) -> XResult<()> {
        Err(XError::unavailable("事件总线不可用"))
    }

    async fn subscribe(&self, _topic: &str) -> XResult<BoxStream<'static, BusMessage>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

#[tokio::test]
async fn event_bus_suite_rejects_publish_error_without_delivery_claims() {
    let fixture = FixtureNamespace::new("ctk_event_bus_broken").expect("valid fixture");
    let failure = assert_event_bus_with_fixture(&RejectingEventBus, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "EventBus");
    assert_eq!(failure.case, "surface_publish");
}

struct DroppingKeyValueStore;

#[async_trait]
impl contracts::KeyValueStore for DroppingKeyValueStore {
    async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn key_value_suite_rejects_dropped_set() {
    let fixture = FixtureNamespace::new("ctk_key_value_broken").expect("valid fixture");
    let failure = assert_key_value_store_isolated(&DroppingKeyValueStore, &fixture)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "KeyValueStore");
    assert_eq!(failure.case, "get_hit");
}

struct CommitFailingContext;

#[async_trait]
impl TxContext for CommitFailingContext {
    async fn commit(&mut self) -> XResult<()> {
        Err(XError::transient("事务提交失败"))
    }

    async fn rollback(&mut self) -> XResult<()> {
        Ok(())
    }
}

struct CommitFailingRunner;

#[async_trait]
impl TxRunner for CommitFailingRunner {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(CommitFailingContext))
    }
}

#[tokio::test]
async fn tx_runner_suite_rejects_commit_failure() {
    let failure = contract_testkit::assert_tx_runner(&CommitFailingRunner)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "TxRunner");
    assert_eq!(failure.case, "commit_path");
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    id: String,
    body: String,
}

struct DroppingRepository;

#[async_trait]
impl Repository<Row, String> for DroppingRepository {
    async fn find(&self, _id: String) -> XResult<Option<Row>> {
        Ok(None)
    }

    async fn save(&self, _entity: &Row) -> XResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn repository_suite_rejects_dropped_save() {
    let failure = contract_testkit::assert_repository(
        &DroppingRepository,
        Row { id: "row_1".into(), body: "before".into() },
        "row_1".into(),
        |row| row.body = "after".into(),
        |left, right| left == right,
    )
    .await
    .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "Repository");
    assert_eq!(failure.case, "find_hit");
}

struct NoopInstrumentation;

impl Instrumentation for NoopInstrumentation {
    fn record_retry(&self, _op: &str, _attempt: u32) {}

    fn record_circuit_open(&self, _op: &str) {}

    fn record_circuit_close(&self, _op: &str) {}
}

#[test]
fn instrumentation_observed_suite_rejects_noop() {
    let fixture = FixtureNamespace::new("ctk_instrumentation_broken").expect("valid fixture");
    let failure =
        contract_testkit::assert_instrumentation_observed(&NoopInstrumentation, &fixture, || {
            Ok(Vec::new())
        })
        .expect_err("no-op implementation must fail observed suite");

    assert_eq!(failure.contract, "Instrumentation");
    assert_eq!(failure.case, "retry_missing");
}

struct BrokenMarketDataSource;

#[async_trait]
impl MarketDataSource for BrokenMarketDataSource {
    async fn subscribe_ticks(&self, _symbol: &str) -> XResult<BoxStream<'static, Tick>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }

    async fn subscribe_orderbook(
        &self,
        _symbol: &str,
    ) -> XResult<BoxStream<'static, OrderBookSnapshot>> {
        Err(XError::unavailable("orderbook unavailable"))
    }

    async fn subscribe_trades(&self, _symbol: &str) -> XResult<BoxStream<'static, Trade>> {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

#[tokio::test]
async fn market_data_suite_rejects_orderbook_subscription_error() {
    let failure = contract_testkit::assert_market_data_source(&BrokenMarketDataSource, "BTCUSDT")
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "MarketDataSource");
    assert_eq!(failure.case, "subscribe_orderbook");
}

struct WrongSymbolCatalog;

#[async_trait]
impl InstrumentCatalog for WrongSymbolCatalog {
    async fn symbol_info(&self, _symbol: &str) -> XResult<SymbolMeta> {
        Ok(contract_testkit::default_symbol_meta("WRONG"))
    }
}

#[tokio::test]
async fn instrument_catalog_suite_rejects_wrong_symbol() {
    let failure = contract_testkit::assert_instrument_catalog(&WrongSymbolCatalog, "BTCUSDT")
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "InstrumentCatalog");
    assert_eq!(failure.case, "symbol_match");
}

struct EmptyAckExecutionVenue;

#[async_trait]
impl ExecutionVenue for EmptyAckExecutionVenue {
    async fn place_order(&self, _order: &Order) -> XResult<OrderAck> {
        Ok(OrderAck { id: String::new(), status: OrderStatus::Pending, ts: 0 })
    }

    async fn cancel_order(&self, _request: &CancelOrderRequest) -> XResult<()> {
        Ok(())
    }

    async fn query_order(&self, _request: &CancelOrderRequest) -> XResult<OrderStatus> {
        Ok(OrderStatus::Open)
    }

    fn venue_id(&self) -> VenueId {
        "broken".into()
    }
}

#[tokio::test]
async fn execution_venue_suite_rejects_empty_ack_id() {
    let order = contract_testkit::sample_order("order_1", "BTCUSDT");
    let failure = contract_testkit::assert_execution_venue(&EmptyAckExecutionVenue, &order)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "ExecutionVenue");
    assert_eq!(failure.case, "ack_id_nonempty");
}

struct BalanceFailingAccountSource;

#[async_trait]
impl AccountSource for BalanceFailingAccountSource {
    async fn query_position(&self) -> XResult<Vec<Position>> {
        Ok(Vec::new())
    }

    async fn query_balance(&self) -> XResult<Vec<Money>> {
        Err(XError::unavailable("balance unavailable"))
    }
}

#[tokio::test]
async fn account_source_suite_rejects_balance_error() {
    let failure = contract_testkit::assert_account_source(&BalanceFailingAccountSource)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "AccountSource");
    assert_eq!(failure.case, "query_balance");
}

struct FailingVenueTimeSource;

#[async_trait]
impl VenueTimeSource for FailingVenueTimeSource {
    async fn server_time(&self) -> XResult<i64> {
        Err(XError::unavailable("venue time unavailable"))
    }
}

#[tokio::test]
async fn venue_time_suite_rejects_source_error() {
    let failure = contract_testkit::assert_venue_time_source(&FailingVenueTimeSource)
        .await
        .expect_err("broken implementation must fail");

    assert_eq!(failure.contract, "VenueTimeSource");
    assert_eq!(failure.case, "server_time");
}
