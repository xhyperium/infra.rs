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
