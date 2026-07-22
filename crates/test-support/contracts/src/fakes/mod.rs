//! 进程内 Fake / Recording：驱动真实 trait 路径（非真实 DB / 网络 / 交易所）。
//!
//! 用户可见错误信息使用中文（宪章 §4.5）。

mod event_bus;
mod instrumentation;
mod key_value_store;
mod repository;
mod tx;
mod venue_caps;

pub use event_bus::FakeEventBus;
pub use instrumentation::{InstrEvent, RecordingInstrumentation};
pub use key_value_store::FakeKeyValueStore;
pub use repository::FakeRepository;
pub use tx::{FakeTxContext, FakeTxRunner, RecordingTxRunner};
pub use venue_caps::{
    FakeAccountSource, FakeExecutionVenue, FakeInstrumentCatalog, FakeMarketDataSource,
    FakeVenueTimeSource, default_symbol_meta, sample_order,
};

mod batch2;
pub use batch2::{FakeAnalyticsSink, FakeObjectStore, FakePubSub, FakeTimeSeriesStore};
