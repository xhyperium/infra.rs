//! `infra-contracts` — 适配器与可观测性契约 trait。
//!
//! - 交易所 / 存储适配器接口（`exchange` / `storage`）
//! - 可观测性注入点 [`Instrumentation`]（ADR-005，供 `observex` 实现）

pub mod exchange;
pub mod instrumentation;
pub mod storage;

mod error;
pub use error::{Error, Result};
pub use instrumentation::Instrumentation;

/// 适配器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AdapterState {
    /// 未初始化
    Uninitialized,
    /// 已连接
    Connected,
    /// 连接断开
    Disconnected,
    /// 已关闭
    Shutdown,
}
