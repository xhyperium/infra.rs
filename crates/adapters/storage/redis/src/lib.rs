//! `redisx` storage adapter scaffold (in-memory).

mod adapter;
mod error;

pub use adapter::RedisAdapter;
pub use error::{Error, Result};

/// 适配器状态（本 crate 本地）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterState {
    Uninitialized,
    Connected,
    Disconnected,
    Shutdown,
}

/// 最小存储适配器面（本 crate 本地 trait）。
pub trait StorageAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn state(&self) -> AdapterState;
    fn write(&self, key: &str, value: &[u8]) -> Result<()>;
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn delete(&self, key: &str) -> Result<()>;
}
