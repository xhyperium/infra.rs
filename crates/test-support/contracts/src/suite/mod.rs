//! Per-trait conformance suite（禁止一个大宏同时测所有能力）。

mod account_source;
mod event_bus;
mod execution_venue;
mod instrument_catalog;
mod instrumentation;
mod key_value_store;
mod market_data_source;
mod repository;
mod tx;
mod venue_time_source;

pub use account_source::assert_account_source;
pub use event_bus::assert_event_bus;
pub use execution_venue::assert_execution_venue;
pub use instrument_catalog::assert_instrument_catalog;
pub use instrumentation::assert_instrumentation;
pub use key_value_store::assert_key_value_store;
pub use market_data_source::assert_market_data_source;
pub use repository::assert_repository;
pub use tx::assert_tx_runner;
pub use venue_time_source::assert_venue_time_source;
