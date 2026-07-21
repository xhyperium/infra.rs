//! 公开 API 集成测试：从 crate 根调用 shipped 函数（非 re-implement oracle）。

use schedulex::Scheduler;
use std::collections::HashSet;

#[test]
fn public_schedule_list_cancel_roundtrip() {
    let mut s = Scheduler::new();
    s.schedule("alpha");
    s.schedule("beta");

    let set: HashSet<_> = s.list().into_iter().collect();
    assert_eq!(set, HashSet::from(["alpha".to_string(), "beta".to_string()]));

    assert!(s.cancel("alpha"));
    assert!(!s.cancel("alpha"));

    let left: HashSet<_> = s.list().into_iter().collect();
    assert_eq!(left, HashSet::from(["beta".to_string()]));
}

#[test]
fn public_default_empty_and_schedule_owned_string() {
    let mut s = Scheduler::default();
    assert!(s.list().is_empty());
    s.schedule(String::from("owned-id"));
    assert_eq!(s.list(), vec!["owned-id".to_string()]);
}

#[test]
fn public_registry_helpers_stats_bulk() {
    use schedulex::{
        NO_HARD_CAPACITY, Scheduler, is_busy, schedule_checked_many, schedule_filtering, stats,
    };
    let mut s = Scheduler::new();
    assert_eq!(schedule_checked_many(&mut s, &["a", "b"]).unwrap(), 2);
    let (ok, bad) = schedule_filtering(&mut s, &["c", ""]);
    assert_eq!(ok, 1);
    assert_eq!(bad.len(), 1);
    assert!(is_busy(&s, 3));
    assert_eq!(stats(&s).len, 3);
    assert!(NO_HARD_CAPACITY.is_none());
    s.schedule_normalized("  d  ").unwrap();
    assert!(s.contains("d"));
}
