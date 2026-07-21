//! `postgresx` — Postgres adapter scaffold。
//!
//! 实现 [`contracts::Repository`] 与 [`contracts::TxRunner`]。

mod adapter;

pub use adapter::{PostgresAdapter, Record};
