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

mod failure;
mod fakes;
mod suite;

pub use failure::{ContractFailure, ContractResult, ensure};
pub use fakes::{
    FakeAccountSource, FakeEventBus, FakeExecutionVenue, FakeInstrumentCatalog, FakeKeyValueStore,
    FakeMarketDataSource, FakeRepository, FakeTxContext, FakeTxRunner, FakeVenueTimeSource,
    InstrEvent, RecordingInstrumentation, RecordingTxRunner, default_symbol_meta, sample_order,
};
pub use suite::{
    assert_account_source, assert_event_bus, assert_execution_venue, assert_instrument_catalog,
    assert_instrumentation, assert_key_value_store, assert_market_data_source, assert_repository,
    assert_tx_runner, assert_venue_time_source,
};
