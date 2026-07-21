#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # `configx` — L1 内存字符串键值配置存储
//!
//! 当前 active 合同（0.1.0）只提供线程安全的内存 `String` key-value 存储。
//! **不是**多源加载、schema 校验或热更新系统；那些上位目标在 SSOT 中仍为 Unknown。
//!
//! 生产依赖仅 [`kernel`]（package `xhyper-kernel`），不依赖 `observex` 或其他 L1。

use std::collections::HashMap;
use std::sync::RwLock;

use kernel::{XError, XResult};

/// 线程安全的拥有型内存配置存储。
///
/// 内部为私有的 `RwLock<HashMap<String, String>>`。公开面仅有
/// [`new`](ConfigStore::new) / [`get`](ConfigStore::get) / [`set`](ConfigStore::set)
/// 与 [`Default`]。
///
/// # 锁失败语义（不对称）
///
/// - 读锁中毒：折叠为 [`None`]（无法区分「缺失」与「中毒」）
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
    /// 调用方不持有内部锁或引用。
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        // 显式 if/else：llvm-cov --branch 可计数（读中毒 / 健康），且无 expect/unwrap
        if let Ok(guard) = self.data.read() { guard.get(key).cloned() } else { None }
    }

    /// 插入或覆盖 key。
    ///
    /// 写锁中毒时返回 `XError::Invalid`，上下文为 `config lock poisoned`。
    pub fn set(&self, key: impl Into<String>, val: impl Into<String>) -> XResult<()> {
        // 显式 if/else：llvm-cov --branch 可计数（写中毒 / 健康），且无 expect/unwrap
        if let Ok(mut guard) = self.data.write() {
            guard.insert(key.into(), val.into());
            Ok(())
        } else {
            Err(XError::invalid("config lock poisoned"))
        }
    }
}

impl Default for ConfigStore {
    /// 等价于 [`ConfigStore::new`]。
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::ErrorKind;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;
    use std::thread;

    /// 通过同模块私有字段注入真实写/读锁中毒（与 kernel poison 测法一致）。
    fn poison_store(store: &ConfigStore) {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            // 健康锁上持有写锁后 panic → 毒化 RwLock（setup 路径不需要 into_inner）
            let _guard = store.data.write().expect("healthy lock for intentional poison");
            panic!("intentional configx RwLock poison");
        }));
        assert!(result.is_err());
    }

    #[test]
    fn empty_store_returns_none() {
        let store = ConfigStore::new();
        assert_eq!(store.get("any"), None);
        assert_eq!(store.get(""), None);
    }

    #[test]
    fn default_equals_empty_new() {
        let store = ConfigStore::default();
        assert_eq!(store.get("k"), None);
        store.set("k", "v").expect("set on default");
        assert_eq!(store.get("k").as_deref(), Some("v"));
    }

    #[test]
    fn set_then_get_returns_owned_clone() {
        let store = ConfigStore::new();
        store.set("host", "localhost").expect("set");
        let a = store.get("host").expect("get");
        let b = store.get("host").expect("get again");
        assert_eq!(a, "localhost");
        assert_eq!(b, "localhost");
        // 拥有返回值：可独立 drop，互不影响
        drop(a);
        assert_eq!(b, "localhost");
    }

    #[test]
    fn set_overwrites_same_key() {
        let store = ConfigStore::new();
        store.set("port", "8080").expect("first set");
        store.set("port", "9090").expect("overwrite");
        assert_eq!(store.get("port").as_deref(), Some("9090"));
    }

    #[test]
    fn multi_key_isolation() {
        let store = ConfigStore::new();
        store.set("a", "1").expect("a");
        store.set("b", "2").expect("b");
        store.set("a", "3").expect("a overwrite");
        assert_eq!(store.get("a").as_deref(), Some("3"));
        assert_eq!(store.get("b").as_deref(), Some("2"));
        assert_eq!(store.get("c"), None);
    }

    #[test]
    fn read_lock_poison_folds_to_none() {
        let store = ConfigStore::new();
        store.set("k", "v").expect("pre-poison set");
        poison_store(&store);
        // 读锁中毒：折叠为 None（不 panic）
        assert_eq!(store.get("k"), None);
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn write_lock_poison_returns_invalid() {
        let store = ConfigStore::new();
        poison_store(&store);
        let err = store.set("k", "v").expect_err("poisoned write must fail");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        let ctx = err.context();
        assert!(ctx.contains("config lock poisoned"));
    }

    #[test]
    fn concurrent_read_write_smoke() {
        let store = Arc::new(ConfigStore::new());
        store.set("n", "0").expect("seed");

        let writers: Vec<_> = (0..4)
            .map(|i| {
                let s = Arc::clone(&store);
                thread::spawn(move || {
                    for j in 0..50 {
                        s.set(format!("w{i}"), format!("{j}")).expect("concurrent set");
                    }
                })
            })
            .collect();

        let readers: Vec<_> = (0..4)
            .map(|_| {
                let s = Arc::clone(&store);
                thread::spawn(move || {
                    for _ in 0..50 {
                        let _ = s.get("n");
                        let _ = s.get("w0");
                    }
                })
            })
            .collect();

        for h in writers.into_iter().chain(readers) {
            h.join().expect("thread join");
        }

        // 至少能读到最后写入过的 writer key
        assert!(store.get("w0").is_some());
        assert_eq!(store.get("n").as_deref(), Some("0"));
    }
}
