//! `okxx` — OKX exchange adapter。
//!
//! 实现 [`infra_contracts::exchange::ExchangeAdapter`]。

pub use adapter::OkxAdapter;
pub use infra_contracts::{AdapterState, Error, Result};

mod adapter;
