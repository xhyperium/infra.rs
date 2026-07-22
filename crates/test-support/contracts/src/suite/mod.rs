//! Per-trait conformance suite（禁止一个大宏同时测所有能力）。

mod account_source;
mod analytics_sink;
mod event_bus;
mod execution_venue;
mod instrument_catalog;
mod instrumentation;
mod key_value_store;
mod market_data_source;
mod object_store;
mod pub_sub;
mod repository;
mod time_series_store;
mod tx;
mod venue_time_source;

pub use account_source::assert_account_source;
pub use analytics_sink::{
    assert_analytics_sink, assert_analytics_sink_callable, assert_analytics_sink_observed,
};
pub use event_bus::{assert_event_bus, assert_event_bus_surface, assert_event_bus_with_fixture};
pub use execution_venue::assert_execution_venue;
pub use instrument_catalog::assert_instrument_catalog;
pub use instrumentation::{assert_instrumentation, assert_instrumentation_observed};
pub use key_value_store::{assert_key_value_store, assert_key_value_store_isolated};
pub use market_data_source::assert_market_data_source;
pub use object_store::{assert_object_store, assert_object_store_with_fixture};
pub use pub_sub::{assert_pub_sub_smoke, assert_pub_sub_surface};
pub use repository::assert_repository;
pub use time_series_store::{assert_time_series_store, assert_time_series_store_with_fixture};
pub use tx::assert_tx_runner;
pub use venue_time_source::assert_venue_time_source;
