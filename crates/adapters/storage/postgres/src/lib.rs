//! `postgresx` — Postgres adapter。
//!
//! - scaffold：[`PostgresAdapter`]（内存 + `FakeTxContext`）
//! - mock 验证入口：[`ObservingPostgresAdapter`] / [`MockPostgresBackend`]
//!   （staged 写入 + commit 边界；**非**真实 Postgres）

mod adapter;
mod mock;

pub use adapter::{PostgresAdapter, Record};
pub use mock::{MockPostgresBackend, MockTxContext, ObservingPostgresAdapter, TxObservability};
