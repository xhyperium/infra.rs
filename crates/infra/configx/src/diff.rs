//! 配置差异视图（内存快照对比）。

use crate::ConfigSnapshot;
use std::collections::BTreeSet;

/// 两个快照之间的 key 级差异。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigDiff {
    /// 仅左侧有。
    pub only_left: Vec<String>,
    /// 仅右侧有。
    pub only_right: Vec<String>,
    /// 两侧都有但值不同。
    pub changed: Vec<String>,
}

impl ConfigDiff {
    /// 是否无差异。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.only_left.is_empty() && self.only_right.is_empty() && self.changed.is_empty()
    }

    /// 变更键总数。
    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.only_left.len() + self.only_right.len() + self.changed.len()
    }
}

/// 计算 `left` 相对 `right` 的差异。
#[must_use]
pub fn diff_snapshots(left: &ConfigSnapshot, right: &ConfigSnapshot) -> ConfigDiff {
    let lk: BTreeSet<_> = left.keys().into_iter().collect();
    let rk: BTreeSet<_> = right.keys().into_iter().collect();
    let mut out = ConfigDiff::default();
    for k in lk.difference(&rk) {
        out.only_left.push(k.clone());
    }
    for k in rk.difference(&lk) {
        out.only_right.push(k.clone());
    }
    for k in lk.intersection(&rk) {
        if left.get(k) != right.get(k) {
            out.changed.push(k.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfigSnapshot, store_from_pairs};

    #[test]
    fn diff_detects_add_remove_change() {
        let a = store_from_pairs([("k", "1"), ("x", "a")]).unwrap();
        let b = store_from_pairs([("k", "2"), ("y", "b")]).unwrap();
        let d = diff_snapshots(&ConfigSnapshot::capture(&a), &ConfigSnapshot::capture(&b));
        assert!(!d.is_empty());
        assert!(d.only_left.contains(&"x".to_string()));
        assert!(d.only_right.contains(&"y".to_string()));
        assert!(d.changed.contains(&"k".to_string()));
        assert_eq!(d.total_changes(), 3);
        let same = ConfigSnapshot::capture(&a);
        assert!(diff_snapshots(&same, &same).is_empty());
        let _ = format!("{d:?}");
    }

    #[test]
    fn diff_empty_to_full() {
        let empty = ConfigSnapshot::capture(&crate::ConfigStore::new());
        let full = ConfigSnapshot::capture(&store_from_pairs([("a", "1")]).unwrap());
        let d = diff_snapshots(&empty, &full);
        assert_eq!(d.only_right, vec!["a".to_string()]);
        assert!(d.only_left.is_empty());
        assert!(d.changed.is_empty());
    }
}
