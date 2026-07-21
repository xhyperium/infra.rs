//! `redisx` — redis storage adapter。
//!
//! 实现 [`infra_contracts::storage::StorageAdapter`]。

pub use adapter::RedisAdapter;
pub use infra_contracts::{AdapterState, Error, Result};

mod adapter;
