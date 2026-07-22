//! 多层配置合并：按注册顺序，**后注册源覆盖先注册源**。

use std::collections::HashMap;
use std::sync::Arc;

use kernel::XResult;

use crate::ConfigStore;
use crate::source::ConfigSource;

/// 多层配置合并器。
///
/// # 优先级
///
/// `sources` 按插入顺序排列：**越靠后优先级越高**（同 key 时后者覆盖前者）。
pub struct LayeredConfig {
    sources: Vec<Arc<dyn ConfigSource>>,
}

impl LayeredConfig {
    /// 空层。
    #[must_use]
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// 追加一层（最高优先级变为该层）。
    pub fn push(&mut self, source: Arc<dyn ConfigSource>) {
        self.sources.push(source);
    }

    /// 链式追加。
    #[must_use]
    pub fn with_source(mut self, source: Arc<dyn ConfigSource>) -> Self {
        self.push(source);
        self
    }

    /// 层数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// 是否无层。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// 加载并合并全部源；后层覆盖前层。
    pub fn load_merged(&self) -> XResult<HashMap<String, String>> {
        let mut merged = HashMap::new();
        for src in &self.sources {
            let map = src.load()?;
            for (k, v) in map {
                merged.insert(k, v);
            }
        }
        Ok(merged)
    }

    /// 合并结果写入目标 [`ConfigStore`]（不清空既有 key，仅覆盖/新增）。
    pub fn apply_to(&self, store: &ConfigStore) -> XResult<usize> {
        let merged = self.load_merged()?;
        let n = merged.len();
        store.extend_pairs(merged)?;
        Ok(n)
    }

    /// 清空目标后写入合并结果。
    pub fn reload_into(&self, store: &ConfigStore) -> XResult<usize> {
        store.clear()?;
        self.apply_to(store)
    }
}

impl Default for LayeredConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LayeredConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayeredConfig").field("layers", &self.sources.len()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::MemorySource;

    #[test]
    fn later_overrides_earlier() {
        let low = Arc::new(MemorySource::from_pairs([("a", "1"), ("b", "2")]));
        let high = Arc::new(MemorySource::from_pairs([("b", "9"), ("c", "3")]));
        let layered = LayeredConfig::new().with_source(low).with_source(high);
        assert_eq!(layered.len(), 2);
        let m = layered.load_merged().unwrap();
        assert_eq!(m.get("a").map(String::as_str), Some("1"));
        assert_eq!(m.get("b").map(String::as_str), Some("9"));
        assert_eq!(m.get("c").map(String::as_str), Some("3"));
        let store = ConfigStore::new();
        store.set("keep", "x").unwrap();
        assert_eq!(layered.apply_to(&store).unwrap(), 3);
        assert_eq!(store.get("keep").as_deref(), Some("x"));
        assert_eq!(store.get("b").as_deref(), Some("9"));
        layered.reload_into(&store).unwrap();
        assert_eq!(store.get("keep"), None);
        assert_eq!(store.get("a").as_deref(), Some("1"));
        let _ = format!("{:?}", layered);
    }
}
