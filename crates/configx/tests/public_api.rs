//! 公开面集成测试：从 crate 边界驱动 shipped `ConfigStore`。
//!
//! 覆盖 active SSOT §4 不变量与 §3 公开 API 面（不含毒锁——毒锁在单元测试同模块注入）。

use configx::ConfigStore;
use kernel::ErrorKind;

#[test]
fn consumer_path_set_get() {
    let store = ConfigStore::new();
    assert_eq!(store.get("app.name"), None);

    store.set("app.name", "infra-rs").expect("set must succeed on healthy store");
    assert_eq!(store.get("app.name").as_deref(), Some("infra-rs"));
}

#[test]
fn overwrite_and_isolation_from_outside() {
    let store = ConfigStore::default();
    store.set("a", "1").expect("a");
    store.set("b", "2").expect("b");
    store.set("a", "9").expect("overwrite a");

    assert_eq!(store.get("a").as_deref(), Some("9"));
    assert_eq!(store.get("b").as_deref(), Some("2"));
    assert_eq!(store.get("z"), None);
}

#[test]
fn set_accepts_owned_and_borrowed() {
    let store = ConfigStore::new();
    let key = String::from("owned-key");
    let val = String::from("owned-val");
    store.set(key, val).expect("owned");
    store.set("borrowed", "val").expect("borrowed");
    assert_eq!(store.get("owned-key").as_deref(), Some("owned-val"));
    assert_eq!(store.get("borrowed").as_deref(), Some("val"));
}

#[test]
fn get_returns_owned_string_not_borrow() {
    let store = ConfigStore::new();
    store.set("k", "v").expect("set");
    let owned: String = store.get("k").expect("get");
    // 调用方拿到拥有字符串，可自由 move
    let moved = owned;
    assert_eq!(moved, "v");
    // 存储侧仍可读
    assert_eq!(store.get("k").as_deref(), Some("v"));
}

#[test]
fn healthy_set_is_ok_not_error() {
    let store = ConfigStore::new();
    match store.set("ok", "yes") {
        Ok(()) => {}
        Err(e) => panic!("unexpected error kind={:?} ctx={}", e.kind(), e.context()),
    }
    // 健康路径绝不是 Invalid
    assert_ne!(ErrorKind::Invalid, ErrorKind::Missing);
}

#[test]
fn public_new_helpers_require_nonempty_snapshot_diff() {
    use configx::{
        ConfigSnapshot, diff_snapshots, require_nonempty, set_checked, store_from_pairs,
        subset_snapshot,
    };
    let s = store_from_pairs([("a", "1"), ("b", "2")]).unwrap();
    set_checked(&s, "c", "3").unwrap();
    require_nonempty(&s, &["a", "b", "c"]).unwrap();
    let snap = ConfigSnapshot::capture(&s);
    let sub = subset_snapshot(&s, &["a", "c"]);
    assert_eq!(sub.len(), 2);
    let d = diff_snapshots(&snap, &sub);
    assert!(d.only_left.contains(&"b".to_string()));
}

#[test]
fn result_read_and_snapshot_distinguish_healthy_missing() {
    use configx::{
        ConfigWaitOutcome, ConfigWatch, SecretString, set_secret, try_get_secret,
        try_subset_snapshot,
    };
    use std::sync::Arc;
    use std::time::Duration;

    let store = ConfigStore::new();
    assert_eq!(store.try_get("missing").unwrap(), None);
    store.set("present", "value").unwrap();
    assert_eq!(store.try_get("present").unwrap().as_deref(), Some("value"));
    let snapshot = store.try_snapshot().unwrap();
    assert_eq!(snapshot.get("present"), Some("value"));
    assert_eq!(try_subset_snapshot(&store, &["present"]).unwrap().get("present"), Some("value"));

    set_secret(&store, "token", &SecretString::new("secret-value")).unwrap();
    assert_eq!(try_get_secret(&store, "token").unwrap().unwrap().expose(), "secret-value");

    let watch = Arc::new(ConfigWatch::new());
    let mut subscription = watch.subscribe();
    assert_eq!(
        subscription.wait_timeout_outcome(Duration::ZERO).unwrap(),
        ConfigWaitOutcome::TimedOut
    );
}
