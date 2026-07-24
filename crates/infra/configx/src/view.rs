//! 只读视图辅助。

use crate::{ConfigSnapshot, ConfigStore};

/// 从存储挑选子集键构建快照（缺失键跳过；读锁中毒折叠为空快照）。
///
/// 需要区分 poison 时使用 [`try_subset_snapshot`]。
#[must_use]
pub fn subset_snapshot(store: &ConfigStore, keys: &[&str]) -> ConfigSnapshot {
    try_subset_snapshot(store, keys).unwrap_or_default()
}

/// 从存储挑选子集键构建快照，并显式报告 store 读锁失败。
///
/// # Errors
///
/// 配置读锁中毒时返回 [`kernel::XError::invalid`]。
pub fn try_subset_snapshot(store: &ConfigStore, keys: &[&str]) -> kernel::XResult<ConfigSnapshot> {
    let full = store.try_snapshot()?;
    let entries = keys
        .iter()
        .filter_map(|key| full.get(key).map(|value| ((*key).to_string(), value.to_string())))
        .collect();
    Ok(ConfigSnapshot { entries })
}

/// 两个快照是否在给定 keys 上一致。
#[must_use]
pub fn snapshots_agree(a: &ConfigSnapshot, b: &ConfigSnapshot, keys: &[&str]) -> bool {
    keys.iter().all(|k| a.get(k) == b.get(k))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store_from_pairs;

    #[test]
    fn subset_and_agree() {
        let s = store_from_pairs([("a", "1"), ("b", "2"), ("c", "3")]).unwrap();
        let sub = subset_snapshot(&s, &["a", "c", "missing"]);
        assert_eq!(sub.len(), 2);
        assert_eq!(sub.get("a"), Some("1"));
        let full = ConfigSnapshot::capture(&s);
        assert!(snapshots_agree(&full, &sub, &["a", "c"]));
        assert!(!snapshots_agree(&full, &sub, &["b"]));
        for i in 0..40 {
            let k = format!("k{i}");
            let _ = s.set(&k, format!("v{i}"));
        }
        let many = subset_snapshot(&s, &["k0", "k1", "k2"]);
        assert_eq!(many.len(), 3);
        assert_eq!(try_subset_snapshot(&s, &["b"]).unwrap().get("b"), Some("2"));
    }

    #[test]
    fn empty_subset() {
        let s = ConfigStore::new();
        let sub = subset_snapshot(&s, &["x"]);
        assert!(sub.is_empty());
        assert!(snapshots_agree(&sub, &ConfigSnapshot::capture(&s), &[]));
    }
}
