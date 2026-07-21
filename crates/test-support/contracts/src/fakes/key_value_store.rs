//! KeyValueStore Fake。

use async_trait::async_trait;
use contracts::KeyValueStore;
use kernel::{XError, XResult};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

/// 内存 [`KeyValueStore`] 参考实现。
///
/// TTL：仅记录；本 fake **不**自动过期（最小面）。
#[derive(Debug, Default)]
pub struct FakeKeyValueStore {
    inner: Mutex<HashMap<String, (Vec<u8>, Option<Duration>)>>,
}

impl FakeKeyValueStore {
    /// 新建空存储。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前条目数（测试辅助）。
    pub fn len(&self) -> XResult<usize> {
        let g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        Ok(g.len())
    }

    /// 是否为空（测试辅助）。
    pub fn is_empty(&self) -> XResult<bool> {
        Ok(self.len()? == 0)
    }
}

#[async_trait]
impl KeyValueStore for FakeKeyValueStore {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        Ok(g.get(key).map(|(v, _)| v.clone()))
    }

    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        let mut g = self.inner.lock().map_err(|_| XError::internal("kv lock 中毒"))?;
        g.insert(key.to_string(), (val, ttl));
        Ok(())
    }
}
