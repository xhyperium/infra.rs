//! `kafkax` — kafka storage adapter。
//!
//! 实现 [`infra_contracts::storage::StorageAdapter`]。

pub use adapter::KafkaAdapter;
pub use infra_contracts::{AdapterState, Error, Result};

mod adapter;
