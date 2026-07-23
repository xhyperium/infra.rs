#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # `configx` — L1 配置存储与多源合并
//!
//! 提供线程安全的内存 `String` key-value 存储、存在性校验，以及：
//!
//! | 模块 | 能力 |
//! |------|------|
//! | [`source`] | [`ConfigSource`]：内存 / 环境变量 / KEY=VALUE 文件 |
//! | [`layered`] | [`LayeredConfig`]：多层合并（后源覆盖前源） |
//! | [`watch`] | [`ConfigWatch`]：进程内通知 + 调用方显式触发的 reload |
//! | [`secret`] | [`SecretString`] 脱敏 + `set_secret` / `get_secret` |
//!
//! **非目标**：自动文件 watcher、类型化 schema、分布式配置中心、远端 secret manager。
//! 生产依赖仅 [`kernel`]；不依赖其他 L1。

use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

use kernel::{XError, XResult};

const CONFIG_LOCK_POISONED_CONTEXT: &str = "配置锁已中毒";

pub mod diff;
pub mod layered;
pub mod secret;
pub mod source;
pub mod view;
pub mod watch;
pub use diff::{ConfigDiff, diff_snapshots};
pub use layered::LayeredConfig;
pub use secret::{
    SECRET_KEY_PREFIX, SecretString, get_secret, is_secret_key, set_secret, try_get_secret,
};
pub use source::{ConfigSource, EnvSource, FileSource, MemorySource, parse_key_value_file};
pub use view::{snapshots_agree, subset_snapshot, try_subset_snapshot};
pub use watch::{ConfigChange, ConfigSubscription, ConfigWaitOutcome, ConfigWatch};

/// 线程安全的拥有型内存配置存储。
///
/// # 锁失败语义（不对称）
///
/// - 读锁中毒：兼容 API（如 [`get`](Self::get)）折叠为「缺失 / 空」语义；`try_*` API 显式返回错误
/// - 写锁中毒：返回 [`XError::invalid`]，上下文为“配置锁已中毒”
pub struct ConfigStore {
    data: RwLock<HashMap<String, String>>,
    #[cfg(test)]
    replace_hook: std::sync::Mutex<Option<std::sync::Arc<dyn Fn() + Send + Sync>>>,
}

impl ConfigStore {
    /// 创建空存储。
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            #[cfg(test)]
            replace_hook: std::sync::Mutex::new(None),
        }
    }

    /// 按 key 克隆返回配置值。
    ///
    /// 返回 [`None`] 当且仅当：key 不存在，或读锁中毒。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        self.try_get(key).unwrap_or_default()
    }

    /// 按 key 克隆返回配置值，并显式报告读锁失败。
    ///
    /// 与 [`get`](Self::get) 不同，本方法只用 [`None`] 表示 key 缺失。
    ///
    /// # Errors
    ///
    /// 配置读锁中毒时返回 [`XError::invalid`]。
    pub fn try_get(&self, key: &str) -> XResult<Option<String>> {
        let guard = self.data.read().map_err(|_| XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))?;
        Ok(guard.get(key).cloned())
    }

    /// 插入或覆盖 key。
    ///
    /// # Errors
    ///
    /// 配置写锁中毒时返回 [`XError::invalid`]。
    pub fn set(&self, key: impl Into<String>, val: impl Into<String>) -> XResult<()> {
        if let Ok(mut guard) = self.data.write() {
            guard.insert(key.into(), val.into());
            Ok(())
        } else {
            Err(XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))
        }
    }

    /// key 是否存在（读锁中毒视为不存在）。
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        if let Ok(guard) = self.data.read() { guard.contains_key(key) } else { false }
    }

    /// 移除 key；返回旧值（若有）。写锁中毒 → Invalid。
    ///
    /// # Errors
    ///
    /// 配置写锁中毒时返回 [`XError::invalid`]。
    pub fn remove(&self, key: &str) -> XResult<Option<String>> {
        if let Ok(mut guard) = self.data.write() {
            Ok(guard.remove(key))
        } else {
            Err(XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))
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
    ///
    /// # Errors
    ///
    /// 配置写锁中毒时返回 [`XError::invalid`]。
    pub fn clear(&self) -> XResult<()> {
        if let Ok(mut guard) = self.data.write() {
            guard.clear();
            Ok(())
        } else {
            Err(XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))
        }
    }

    /// 批量写入键值对。
    ///
    /// 全部键值先在锁外收集，再以单次写锁提交；读者不会观察到部分提交。
    ///
    /// # Errors
    ///
    /// 配置写锁中毒时返回 [`XError::invalid`]，且批次不会部分提交。
    pub fn extend_pairs<I, K, V>(&self, pairs: I) -> XResult<()>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let entries = pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.extend_entries(entries)
    }

    /// 读取 key；缺失时返回 `default` 的拥有副本。
    #[must_use]
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    /// 拍摄完整配置快照，并显式报告读锁失败。
    ///
    /// # Errors
    ///
    /// 配置读锁中毒时返回 [`XError::invalid`]。
    pub fn try_snapshot(&self) -> XResult<ConfigSnapshot> {
        ConfigSnapshot::try_capture(self)
    }

    fn extend_entries(&self, entries: HashMap<String, String>) -> XResult<()> {
        let mut guard =
            self.data.write().map_err(|_| XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))?;
        guard.extend(entries);
        Ok(())
    }

    pub(crate) fn replace_entries(&self, entries: HashMap<String, String>) -> XResult<usize> {
        let count = entries.len();
        let mut guard =
            self.data.write().map_err(|_| XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))?;
        #[cfg(test)]
        if let Some(hook) = self.replace_hook.lock().expect("test replace hook lock").as_ref() {
            hook();
        }
        *guard = entries;
        Ok(count)
    }

    #[cfg(test)]
    fn set_replace_hook(&self, hook: std::sync::Arc<dyn Fn() + Send + Sync>) {
        *self.replace_hook.lock().expect("test replace hook lock") = Some(hook);
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置键存在性校验（schema 边界最小面，infra-s9t.7）。
///
/// **不是** 类型化 schema。仅检查内存 [`ConfigStore`] 是否包含必填 key。
///
/// # Errors
///
/// 配置读锁中毒或任一必填键缺失时返回 [`XError::invalid`]。
pub fn require_keys(store: &ConfigStore, keys: &[&str]) -> XResult<()> {
    let snapshot = store.try_snapshot()?;
    for k in keys {
        if snapshot.get(k).is_none() {
            return Err(XError::invalid(format!("缺少必填配置键：{k}")));
        }
    }
    Ok(())
}

/// 必填 key 必须存在且值非空（trim 后）。
///
/// # Errors
///
/// 配置读锁中毒、必填键缺失或值为空时返回 [`XError::invalid`]。
pub fn require_nonempty(store: &ConfigStore, keys: &[&str]) -> XResult<()> {
    let snapshot = store.try_snapshot()?;
    for k in keys {
        match snapshot.get(k) {
            None => {
                return Err(XError::invalid(format!("缺少必填配置键：{k}")));
            }
            Some(v) if v.trim().is_empty() => {
                return Err(XError::invalid(format!("配置键的值为空：{k}")));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// 从 `(key, value)` 迭代器构建新存储。
///
/// # Errors
///
/// 内部批量写入失败时返回 [`XError::invalid`]。
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
///
/// `Debug` 会把 `secret:` 前缀键对应的值显示为 `***`；普通读取仍返回原值。
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ConfigSnapshot {
    entries: HashMap<String, String>,
}

impl ConfigSnapshot {
    /// 从存储拍快照（读锁中毒 → 空快照）。
    #[must_use]
    pub fn capture(store: &ConfigStore) -> Self {
        Self::try_capture(store).unwrap_or_default()
    }

    /// 从存储拍快照，并显式报告读锁失败。
    ///
    /// # Errors
    ///
    /// 配置读锁中毒时返回 [`XError::invalid`]。
    pub fn try_capture(store: &ConfigStore) -> XResult<Self> {
        let guard = store.data.read().map_err(|_| XError::invalid(CONFIG_LOCK_POISONED_CONTEXT))?;
        Ok(Self { entries: guard.clone() })
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

struct RedactedEntries<'a>(&'a HashMap<String, String>);

impl fmt::Debug for RedactedEntries<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_unstable_by_key(|(key, _)| *key);
        let mut map = f.debug_map();
        for (key, value) in entries {
            if secret::is_secret_key(key) {
                map.entry(key, &"***");
            } else {
                map.entry(key, value);
            }
        }
        map.finish()
    }
}

impl fmt::Debug for ConfigSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigSnapshot").field("entries", &RedactedEntries(&self.entries)).finish()
    }
}

/// 校验配置 key 本身是否可接受（非空、无控制字符）。
///
/// 不改变存储；供调用方在 `set` 前做门禁。
///
/// # Errors
///
/// key 为空、包含控制字符或超过 512 字节时返回 [`XError::invalid`]。
pub fn validate_key(key: &str) -> XResult<()> {
    if key.is_empty() {
        return Err(XError::invalid("配置键不能为空"));
    }
    if key.chars().any(|c| c.is_control()) {
        return Err(XError::invalid("配置键不能包含控制字符"));
    }
    if key.len() > 512 {
        return Err(XError::invalid("配置键长度超过 512 字节"));
    }
    Ok(())
}

/// `set` 前校验 key；通过后写入。
///
/// # Errors
///
/// key 校验失败或配置写锁中毒时返回 [`XError::invalid`]。
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
///
/// # Errors
///
/// overlay 读锁或 base 写锁中毒时返回 [`XError::invalid`]，base 不会部分提交。
pub fn merge_into(base: &ConfigStore, overlay: &ConfigStore) -> XResult<usize> {
    let snapshot = overlay.try_snapshot()?;
    let count = snapshot.len();
    base.extend_entries(snapshot.entries)?;
    Ok(count)
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
        assert_invalid_context(require_keys(&s, &["b"]).unwrap_err(), "缺少必填配置键：b");
        s.set("e", "  ").unwrap();
        assert_invalid_context(require_nonempty(&s, &["e"]).unwrap_err(), "配置键的值为空：e");
        validate_key("ok").unwrap();
        assert_invalid_context(validate_key("").unwrap_err(), "配置键不能为空");
        assert_invalid_context(validate_key("a\nb").unwrap_err(), "配置键不能包含控制字符");
        set_checked(&s, "host", "h").unwrap();
        assert_invalid_context(set_checked(&s, "", "v").unwrap_err(), "配置键不能为空");
    }

    #[test]
    fn poison_semantics() {
        let store = ConfigStore::new();
        store.set("k", "v").unwrap();
        poison_store(&store);
        assert_eq!(store.get("k"), None);
        assert_invalid_context(store.try_get("k").unwrap_err(), CONFIG_LOCK_POISONED_CONTEXT);
        assert!(!store.contains_key("k"));
        assert_eq!(store.len(), 0);
        assert!(store.keys().is_empty());
        assert!(ConfigSnapshot::capture(&store).is_empty());
        assert_invalid_context(store.try_snapshot().unwrap_err(), CONFIG_LOCK_POISONED_CONTEXT);
        assert_invalid_context(
            ConfigSnapshot::try_capture(&store).unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
        assert_invalid_context(
            try_get_secret(&store, "token").unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
        assert_invalid_context(
            try_subset_snapshot(&store, &["k"]).unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
        let err = store.set("k", "v").unwrap_err();
        assert_invalid_context(err, CONFIG_LOCK_POISONED_CONTEXT);
        assert_invalid_context(store.remove("k").unwrap_err(), CONFIG_LOCK_POISONED_CONTEXT);
        assert_invalid_context(store.clear().unwrap_err(), CONFIG_LOCK_POISONED_CONTEXT);
        assert_invalid_context(
            store.extend_pairs([("a", "b")]).unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
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
        assert_invalid_context(err, "缺少必填配置键：nope");
    }

    #[test]
    fn merge_into_reports_when_overlay_read_poisons() {
        let base = ConfigStore::new();
        base.set("keep", "1").unwrap();
        let ov = ConfigStore::new();
        ov.set("x", "2").unwrap();
        poison_store(&ov);
        assert_invalid_context(merge_into(&base, &ov).unwrap_err(), CONFIG_LOCK_POISONED_CONTEXT);
        assert_eq!(base.get("keep").as_deref(), Some("1"));
    }

    #[test]
    fn production_validation_reports_poison() {
        let store = ConfigStore::new();
        store.set("required", "value").unwrap();
        poison_store(&store);
        assert_invalid_context(
            require_keys(&store, &["required"]).unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
        assert_invalid_context(
            require_nonempty(&store, &["required"]).unwrap_err(),
            CONFIG_LOCK_POISONED_CONTEXT,
        );
    }

    #[test]
    fn snapshot_debug_redacts_secret_values() {
        let store =
            store_from_pairs([("plain", "visible"), ("secret:token", "never-print-this")]).unwrap();
        let debug = format!("{:?}", ConfigSnapshot::try_capture(&store).unwrap());
        assert!(debug.contains("visible"));
        assert!(debug.contains("secret:token"));
        assert!(debug.contains("***"));
        assert!(!debug.contains("never-print-this"));
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
        assert_invalid_context(
            validate_key(&"x".repeat(513)).unwrap_err(),
            "配置键长度超过 512 字节",
        );
    }

    #[test]
    fn multi_source_merge_priority_and_secret() {
        use std::sync::Arc;
        let low = Arc::new(MemorySource::from_pairs([("k", "low"), ("only_low", "1")]));
        let high = Arc::new(MemorySource::from_pairs([("k", "high")]));
        let layered = LayeredConfig::new().with_source(low).with_source(high);
        let m = layered.load_merged().unwrap();
        assert_eq!(m.get("k").map(String::as_str), Some("high"));
        assert_eq!(m.get("only_low").map(String::as_str), Some("1"));
        let secret = SecretString::new("s3cr3t");
        assert!(format!("{secret:?}").contains("***"));
        assert!(!format!("{secret:?}").contains("s3cr3t"));
        let store = ConfigStore::new();
        set_secret(&store, "token", &secret).unwrap();
        assert_eq!(get_secret(&store, "token").unwrap().expose(), "s3cr3t");
    }

    fn assert_invalid_context(error: XError, expected: &str) {
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert_eq!(error.context(), expected);
    }
}
