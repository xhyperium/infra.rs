//! 存储适配器合约。
//!
//! 定义所有存储适配器（redis、kafka、postgres 等）必须实现的统一接口。

use crate::{AdapterState, Result};

/// 存储适配器 trait
pub trait StorageAdapter: Send + Sync {
    /// 返回存储类型名称
    fn name(&self) -> &str;

    /// 连接
    fn connect(&mut self) -> Result<()>;

    /// 断开
    fn disconnect(&mut self) -> Result<()>;

    /// 当前状态
    fn state(&self) -> AdapterState;

    /// 写入数据
    fn write(&self, key: &str, value: &[u8]) -> Result<()>;

    /// 读取数据
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 删除数据
    fn delete(&self, key: &str) -> Result<()>;
}
