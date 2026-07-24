//! 多层配置合并：按注册顺序，**后注册源覆盖先注册源**。

use std::collections::HashMap;
use std::sync::Arc;

use kernel::XResult;

use crate::source::ConfigSource;
use crate::{ConfigStore, validate_key};

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
    ///
    /// # Errors
    ///
    /// 任一源加载失败或配置键校验失败时返回错误。
    pub fn load_merged(&self) -> XResult<HashMap<String, String>> {
        let mut merged = HashMap::new();
        for src in &self.sources {
            let map = src.load()?;
            for (k, v) in map {
                validate_key(&k)?;
                merged.insert(k, v);
            }
        }
        Ok(merged)
    }

    /// 合并结果写入目标 [`ConfigStore`]（不清空既有 key，仅覆盖/新增）。
    ///
    /// # Errors
    ///
    /// 源加载/校验失败或目标 store 写锁中毒时返回错误。
    pub fn apply_to(&self, store: &ConfigStore) -> XResult<usize> {
        let merged = self.load_merged()?;
        let n = merged.len();
        store.extend_pairs(merged)?;
        Ok(n)
    }

    /// 以合并结果原子替换目标全部配置。
    ///
    /// **先**完整加载并校验，成功后以单次写锁替换；读者只会看到旧快照或新快照。
    ///
    /// # Errors
    ///
    /// 源加载/校验失败或目标 store 写锁中毒时返回错误；失败时旧配置保持不变。
    pub fn reload_into(&self, store: &ConfigStore) -> XResult<usize> {
        let merged = self.load_merged()?;
        store.replace_entries(merged)
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
    use crate::ConfigSnapshot;
    use crate::source::MemorySource;
    use kernel::ErrorKind;
    use std::sync::{Barrier, TryLockError};
    use std::thread;

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
        let empty = LayeredConfig::default();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn reload_preserves_on_source_error() {
        use crate::source::FileSource;
        let store = ConfigStore::new();
        store.set("keep", "alive").unwrap();
        // 不存在的文件源 → load 失败，keep 应仍在
        let bad = Arc::new(FileSource::new("/no/such/configx-reload-fail.conf"));
        let layered = LayeredConfig::new().with_source(bad);
        let error = layered.reload_into(&store).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert_eq!(error.context(), "读取配置文件失败：路径=/no/such/configx-reload-fail.conf");
        assert_eq!(store.get("keep").as_deref(), Some("alive"));
    }

    #[test]
    fn reload_preserves_on_validation_error() {
        let store = ConfigStore::new();
        store.set("keep", "alive").unwrap();
        let invalid = Arc::new(MemorySource::from_pairs([("bad\nkey", "value")]));
        let layered = LayeredConfig::new().with_source(invalid);
        let error = layered.reload_into(&store).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert_eq!(error.context(), "配置键不能包含控制字符");
        assert_eq!(store.try_get("keep").unwrap().as_deref(), Some("alive"));
    }

    #[test]
    fn reload_readers_only_observe_complete_snapshots() {
        const ENTRY_COUNT: usize = 64;

        let old_pairs: Vec<_> =
            (0..ENTRY_COUNT).map(|i| (format!("key-{i}"), "old".to_string())).collect();
        let new_pairs: Vec<_> =
            (0..ENTRY_COUNT).map(|i| (format!("key-{i}"), "new".to_string())).collect();
        let store = Arc::new(ConfigStore::new());
        store.extend_pairs(old_pairs).unwrap();
        let before = store.try_snapshot().unwrap();
        assert_uniform_snapshot(&before, ENTRY_COUNT, "old");

        let writer_entered = Arc::new(Barrier::new(2));
        let release_writer = Arc::new(Barrier::new(2));
        let hook_entered = Arc::clone(&writer_entered);
        let hook_release = Arc::clone(&release_writer);
        store.set_replace_hook(Arc::new(move || {
            hook_entered.wait();
            hook_release.wait();
        }));

        let layered =
            LayeredConfig::new().with_source(Arc::new(MemorySource::from_pairs(new_pairs)));
        let writer_store = Arc::clone(&store);
        let writer = thread::spawn(move || layered.reload_into(&writer_store));

        writer_entered.wait();
        assert!(matches!(store.data.try_read(), Err(TryLockError::WouldBlock)));
        release_writer.wait();
        assert_eq!(writer.join().unwrap().unwrap(), ENTRY_COUNT);

        let after = store.try_snapshot().unwrap();
        assert_uniform_snapshot(&after, ENTRY_COUNT, "new");
    }

    fn assert_uniform_snapshot(snapshot: &ConfigSnapshot, expected_len: usize, expected: &str) {
        assert_eq!(snapshot.len(), expected_len);
        for index in 0..expected_len {
            assert_eq!(snapshot.get(&format!("key-{index}")), Some(expected));
        }
    }
}
