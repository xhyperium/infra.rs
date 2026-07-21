//! `taosx` — TDengine 时序存储适配器。
//!
//! - **默认**：[`TaosPool`] / [`TaosClient`] REST 生产客户端（端口 6041）。
//! - **feature `scaffold`**：`TaosAdapter` 进程内内存实现（**非**生产）。
//!
//! 实现 [`contracts::TimeSeriesStore`]（`Tick.ts` 为纳秒 epoch）。

#![forbid(unsafe_code)]

mod client;
mod config;

pub use client::{TaosClient, TaosExecResult, TaosPool};
pub use config::{TaosConfig, TsPrecision};

#[cfg(feature = "scaffold")]
mod adapter;
#[cfg(feature = "scaffold")]
pub use adapter::TaosAdapter;
