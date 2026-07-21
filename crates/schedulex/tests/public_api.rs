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
