//! contract-testkit —— 可复用契约 Fake / Recording + per-trait conformance suite。
//!
//! | 项 | 值 |
//! |----|-----|
//! | package | `contract-testkit` |
//! | path | `crates/test-support/contracts` |
//! | plane | **test-support**（`publish = false`） |
//! | 规范 | SPEC-TESTKIT-002 §3.2 |
//!
//! # 使用边界
//!
//! - **仅** 作为业务 / adapter crate 的 `[dev-dependencies]`
//! - **禁止** 进入 production normal graph（含 feature 泄漏）
//! - 与 `testkit`（ManualClock）分离：本 crate 不提供时钟
//!
//! # 公开面
//!
//! - Fake / Recording：[`FakeKeyValueStore`]、[`FakeEventBus`]、[`FakeTxRunner`]…
//! - Suite：[`assert_key_value_store`]、[`assert_event_bus`]…
//! - 失败类型：[`ContractFailure`] / [`ContractResult`]

#![forbid(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(missing_docs)]

mod backend;
mod failure;
mod fakes;
mod fixture;
mod suite;

pub use backend::{BackendProfile, classify_backends, classify_backends_with};
pub use failure::{ContractFailure, ContractResult, ensure};
pub use fakes::{
    FakeAccountSource, FakeAnalyticsSink, FakeEventBus, FakeExecutionVenue, FakeInstrumentCatalog,
    FakeKeyValueStore, FakeMarketDataSource, FakeObjectStore, FakePubSub, FakeRepository,
    FakeTimeSeriesStore, FakeTxContext, FakeTxRunner, FakeVenueTimeSource, InstrEvent,
    RecordingInstrumentation, RecordingTxRunner, default_symbol_meta, sample_order,
};
pub use fixture::FixtureNamespace;
pub use suite::{
    assert_account_source, assert_analytics_sink, assert_analytics_sink_callable,
    assert_analytics_sink_observed, assert_event_bus, assert_event_bus_surface,
    assert_event_bus_with_fixture, assert_execution_venue, assert_instrument_catalog,
    assert_instrumentation, assert_instrumentation_observed, assert_key_value_store,
    assert_key_value_store_isolated, assert_market_data_source, assert_object_store,
    assert_object_store_with_fixture, assert_pub_sub_smoke, assert_pub_sub_surface,
    assert_repository, assert_time_series_store, assert_time_series_store_in_window,
    assert_time_series_store_with_fixture, assert_tx_runner, assert_venue_time_source,
};

/// 不承诺 replay、必达、排序或后端专有语义的可移植核心 suites。
pub mod portable {
    pub use crate::{
        assert_analytics_sink, assert_event_bus_surface, assert_object_store,
        assert_pub_sub_surface, assert_time_series_store_in_window,
        assert_time_series_store_with_fixture,
    };

    /// ClosedPoint 兼容入口；跨后端的新代码推荐使用
    /// [`assert_time_series_store_in_window`]。
    ///
    /// 本符号仅为保持 0.1.2 的公开路径，不代表零宽闭区间可移植到所有后端。
    ///
    /// # Errors
    ///
    /// 后端调用失败或查询结果缺少写入点时返回 [`crate::ContractFailure`]。
    pub async fn assert_time_series_store(
        store: &dyn contracts::TimeSeriesStore,
        unique_table: &str,
        point: canonical::Tick,
    ) -> crate::ContractResult {
        crate::assert_time_series_store(store, unique_table, point).await
    }
}

/// 仅适用于支持零宽闭区间查询的 TimeSeries ClosedPoint profile。
pub mod closed_point {
    pub use crate::assert_time_series_store;
}

/// 仅适用于进程内快照/回放 Fake 的 profile suites。
pub mod snapshot {
    pub use crate::assert_event_bus;
}
