//! `clickhousex` — analytics sink scaffold。
//!
//! 实现 [`contracts::AnalyticsSink`]。

mod adapter;
pub use adapter::ClickHouseAdapter;
