#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # `configx` — L1 内存字符串键值配置存储
//!
//! 当前 active 合同（0.1.0）提供线程安全的内存 `String` key-value 存储与
//! **存在性校验**辅助。**不是**多源加载、类型化 schema 或热更新系统。
//!
//! 生产依赖仅 [`kernel`]；不依赖其他 L1。

use std::collections::HashMap;
use std::sync::RwLock;

use kernel::{XError, XResult};

mod diff;
mod view;
pub use diff::{ConfigDiff, diff_snapshots};
pub use view::{snapshots_agree, subset_snapshot};

/// 线程安全的拥有型内存配置存储。
///
/// # 锁失败语义（不对称）
///
/// - 读锁中毒：查询类 API 折叠为「缺失 / 空」语义
/// - 写锁中毒：返回 [`XError::invalid`]，上下文含 `config lock poisoned`
pub struct ConfigStore {
    data: RwLock<HashMap<String, String>>,
}

impl ConfigStore {
    /// 创建空存储。
    #[must_use]
    pub fn new() -> Self {
        Self { data: RwLock::new(HashMap::new()) }
    }

    /// 按 key 克隆返回配置值。
    ///
    /// 返回 [`None`] 当且仅当：key 不存在，或读锁中毒。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        if let Ok(guard) = self.data.read() { guard.get(key).cloned() } else { None }
    }

    /// 插入或覆盖 key。
    pub fn set(&self, key: impl Into<String>, val: impl Into<String>) -> XResult<()> {
        if let Ok(mut guard) = self.data.write() {
            guard.insert(key.into(), val.into());
            Ok(())
        } else {
            Err(XError::invalid("config lock poisoned"))
        }
    }

    /// key 是否存在（读锁中毒视为不存在）。
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        if let Ok(guard) = self.data.read() { guard.contains_key(key) } else { false }
    }

    /// 移除 key；返回旧值（若有）。写锁中毒 → Invalid。
    pub fn remove(&self, key: &str) -> XResult<Option<String>> {
        if let Ok(mut guard) = self.data.write() {
            Ok(guard.remove(key))
        } else {
            Err(XError::invalid("config lock poisoned"))
        }
    }

    /// 当前条目数（读锁中毒 → 0）。
    #[must_use]
    pub fn len(&self) -> usize {
        if let Ok(guard) = self.data.read() { guard.len() } else { 0 }
    }

    /// 是否为空（读锁中毒 → true）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 当前全部 key（顺序未承诺；读锁中毒 → 空）。
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        if let Ok(guard) = self.data.read() { guard.keys().cloned().collect() } else { Vec::new() }
    }

    /// 清空全部条目。写锁中毒 → Invalid。
    pub fn clear(&self) -> XResult<()> {
        if let Ok(mut guard) = self.data.write() {
            guard.clear();
            Ok(())
        } else {
            Err(XError::invalid("config lock poisoned"))
        }
    }

    /// 批量写入键值对；任一步写锁失败则返回错误（已写入的不回滚）。
    pub fn extend_pairs<I, K, V>(&self, pairs: I) -> XResult<()>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in pairs {
            self.set(k, v)?;
        }
        Ok(())
    }

    /// 读取 key；缺失时返回 `default` 的拥有副本。
    #[must_use]
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置键存在性校验（schema 边界最小面，infra-s9t.7）。
///
/// **不是** 类型化 schema / 多源配置。仅检查内存 [`ConfigStore`] 是否包含必填 key。
pub fn require_keys(store: &ConfigStore, keys: &[&str]) -> XResult<()> {
    for k in keys {
        if store.get(k).is_none() {
            return Err(XError::invalid(format!("missing required config key: {k}")));
        }
    }
    Ok(())
}

/// 必填 key 必须存在且值非空（trim 后）。
pub fn require_nonempty(store: &ConfigStore, keys: &[&str]) -> XResult<()> {
    for k in keys {
        match store.get(k) {
            None => {
                return Err(XError::invalid(format!("missing required config key: {k}")));
            }
            Some(v) if v.trim().is_empty() => {
                return Err(XError::invalid(format!("config key has empty value: {k}")));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// 从 `(key, value)` 迭代器构建新存储。
pub fn store_from_pairs<I, K, V>(pairs: I) -> XResult<ConfigStore>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let store = ConfigStore::new();
    store.extend_pairs(pairs)?;
    Ok(store)
}

/// 只读配置快照（克隆自 [`ConfigStore`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSnapshot {
    entries: HashMap<String, String>,
}

impl ConfigSnapshot {
    /// 从存储拍快照（读锁中毒 → 空快照）。
    #[must_use]
    pub fn capture(store: &ConfigStore) -> Self {
        if let Ok(guard) = store.data.read() {
            Self { entries: guard.clone() }
        } else {
            Self { entries: HashMap::new() }
        }
    }

    /// 查询。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    /// 条目数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 全部 key。
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

/// 校验配置 key 本身是否可接受（非空、无控制字符）。
///
/// 不改变存储；供调用方在 `set` 前做门禁。
pub fn validate_key(key: &str) -> XResult<()> {
    if key.is_empty() {
        return Err(XError::invalid("config key must not be empty"));
    }
    if key.chars().any(|c| c.is_control()) {
        return Err(XError::invalid("config key must not contain control characters"));
    }
    if key.len() > 512 {
        return Err(XError::invalid("config key exceeds max length 512"));
    }
    Ok(())
}

/// `set` 前校验 key；通过后写入。
pub fn set_checked(
    store: &ConfigStore,
    key: impl Into<String>,
    val: impl Into<String>,
) -> XResult<()> {
    let key = key.into();
    validate_key(&key)?;
    store.set(key, val)
}

/// 合并：将 `overlay` 中所有键覆盖写入 `base`。
pub fn merge_into(base: &ConfigStore, overlay: &ConfigStore) -> XResult<usize> {
    // 一次读快照，避免 keys()/get() 之间中毒导致不可达分支
    let pairs: Vec<(String, String)> = if let Ok(guard) = overlay.data.read() {
        guard.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        Vec::new()
    };
    let mut n = 0usize;
    for (k, v) in pairs {
        base.set(k, v)?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;
    use std::thread;

    fn poison_store(store: &ConfigStore) {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = store.data.write().expect("healthy lock for intentional poison");
            panic!("intentional configx RwLock poison");
        }));
        assert!(result.is_err());
    }

    #[test]
    fn empty_and_default() {
        let store = ConfigStore::new();
        assert!(store.is_empty());
        assert_eq!(store.get("x"), None);
        let d = ConfigStore::default();
        d.set("k", "v").unwrap();
        assert_eq!(d.get("k").as_deref(), Some("v"));
    }

    #[test]
    fn set_get_overwrite_remove_clear() {
        let s = ConfigStore::new();
        s.set("a", "1").unwrap();
        s.set("a", "2").unwrap();
        assert_eq!(s.get("a").as_deref(), Some("2"));
        assert!(s.contains_key("a"));
        assert_eq!(s.remove("a").unwrap().as_deref(), Some("2"));
        s.extend_pairs([("x", "1"), ("y", "2")]).unwrap();
        assert_eq!(s.len(), 2);
        s.clear().unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn get_or_keys_snapshot_merge() {
        let s = store_from_pairs([("a", "1")]).unwrap();
        assert_eq!(s.get_or("a", "d"), "1");
        assert_eq!(s.get_or("m", "d"), "d");
        let mut keys = s.keys();
        keys.sort();
        assert_eq!(keys, vec!["a".to_string()]);
        let snap = ConfigSnapshot::capture(&s);
        assert_eq!(snap.get("a"), Some("1"));
        assert!(!snap.is_empty());
        let _ = format!("{snap:?}");
        let ov = store_from_pairs([("a", "9"), ("b", "2")]).unwrap();
        assert_eq!(merge_into(&s, &ov).unwrap(), 2);
        assert_eq!(s.get("a").as_deref(), Some("9"));
    }

    #[test]
    fn require_and_validate() {
        let s = ConfigStore::new();
        s.set("a", "1").unwrap();
        require_keys(&s, &["a"]).unwrap();
        assert!(require_keys(&s, &["b"]).is_err());
        s.set("e", "  ").unwrap();
        assert!(require_nonempty(&s, &["e"]).is_err());
        validate_key("ok").unwrap();
        assert!(validate_key("").is_err());
        assert!(validate_key("a\nb").is_err());
        set_checked(&s, "host", "h").unwrap();
        assert!(set_checked(&s, "", "v").is_err());
    }

    #[test]
    fn poison_semantics() {
        let store = ConfigStore::new();
        store.set("k", "v").unwrap();
        poison_store(&store);
        assert_eq!(store.get("k"), None);
        assert!(!store.contains_key("k"));
        assert_eq!(store.len(), 0);
        assert!(store.keys().is_empty());
        assert!(ConfigSnapshot::capture(&store).is_empty());
        let err = store.set("k", "v").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert!(store.remove("k").is_err());
        assert!(store.clear().is_err());
        assert!(store.extend_pairs([("a", "b")]).is_err());
    }

    #[test]
    fn concurrent_smoke() {
        let store = Arc::new(ConfigStore::new());
        store.set("n", "0").unwrap();
        let writers: Vec<_> = (0..4)
            .map(|i| {
                let s = Arc::clone(&store);
                thread::spawn(move || {
                    for j in 0..40 {
                        s.set(format!("w{i}"), format!("{j}")).unwrap();
                    }
                })
            })
            .collect();
        let readers: Vec<_> = (0..4)
            .map(|_| {
                let s = Arc::clone(&store);
                thread::spawn(move || {
                    for _ in 0..40 {
                        let _ = s.get("n");
                    }
                })
            })
            .collect();
        for h in writers.into_iter().chain(readers) {
            h.join().unwrap();
        }
        assert!(store.get("w0").is_some());
    }

    #[test]
    fn require_nonempty_missing_key() {
        let s = ConfigStore::new();
        let err = require_nonempty(&s, &["nope"]).unwrap_err();
        assert!(format!("{err}").contains("missing required config key: nope"));
    }

    #[test]
    fn merge_into_skips_when_overlay_read_poisons() {
        let base = ConfigStore::new();
        base.set("keep", "1").unwrap();
        let ov = ConfigStore::new();
        ov.set("x", "2").unwrap();
        poison_store(&ov);
        // keys() empty under poison → n=0
        assert_eq!(merge_into(&base, &ov).unwrap(), 0);
        assert_eq!(base.get("keep").as_deref(), Some("1"));
    }

    #[test]
    fn public_api_matrix_smoke() {
        let s = ConfigStore::new();
        set_checked(&s, "region", "ap").unwrap();
        set_checked(&s, "env", "dev").unwrap();
        require_keys(&s, &["region", "env"]).unwrap();
        require_nonempty(&s, &["region", "env"]).unwrap();
        assert_eq!(s.len(), 2);
        let snap = ConfigSnapshot::capture(&s);
        assert_eq!(snap.len(), 2);
        s.remove("env").unwrap();
        let ov = store_from_pairs([("region", "eu"), ("tier", "1")]).unwrap();
        merge_into(&s, &ov).unwrap();
        s.clear().unwrap();
        assert!(s.is_empty());
        assert!(validate_key(&"x".repeat(513)).is_err());
    }
}
