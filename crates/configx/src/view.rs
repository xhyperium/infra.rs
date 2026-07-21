//! 只读视图辅助。

use crate::{ConfigSnapshot, ConfigStore};

/// 从存储挑选子集键构建快照（缺失键跳过）。
#[must_use]
pub fn subset_snapshot(store: &ConfigStore, keys: &[&str]) -> ConfigSnapshot {
    let mut pairs = Vec::new();
    for k in keys {
        if let Some(v) = store.get(k) {
            pairs.push(((*k).to_string(), v));
        }
    }
    // rebuild via capture of temporary store
    let tmp = ConfigStore::new();
    for (k, v) in pairs {
        let _ = tmp.set(k, v);
    }
    ConfigSnapshot::capture(&tmp)
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
    }

    #[test]
    fn empty_subset() {
        let s = ConfigStore::new();
        let sub = subset_snapshot(&s, &["x"]);
        assert!(sub.is_empty());
        assert!(snapshots_agree(&sub, &ConfigSnapshot::capture(&s), &[]));
    }
}
