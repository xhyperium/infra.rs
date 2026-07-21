//! `infra-contracts` — 适配器合约 trait。
//!
//! 定义交易所适配器和存储适配��必须实现的接口。

pub mod exchange;
pub mod storage;

mod error;
pub use error::{Error, Result};

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
