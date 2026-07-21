//! `clickhousex` — ClickHouse 分析汇聚适配器。
//!
//! - **默认**：[`ClickHousePool`] / [`ClickHouseClient`] HTTP 生产客户端（端口 8123）。
//! - **feature `scaffold`**：`ClickHouseAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::AnalyticsSink`]。

#![forbid(unsafe_code)]

mod client;
mod config;

pub use client::{ANALYTICS_TABLE, ClickHouseClient, ClickHousePool};
pub use config::ClickHouseConfig;

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::ClickHouseAdapter;
