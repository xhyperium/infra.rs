//! SPEC-KERNEL-002 §11.4 — 编译期面。
//!
//! `static_assertions` 证明 trait 边界；`error` 模块的 rustdoc `compile_fail` 另从真实
//! 下游编译面证明字段私有、无 `Component`、时间类型无 `Default`、`ShutdownGuard`
//! 无 `Clone`。两者互补。

use kernel::{Clock, ErrorKind, MonotonicInstant, ShutdownGuard, SystemClock, Timestamp, XError};
use static_assertions::{assert_impl_all, assert_not_impl_any};
use std::time::Duration;

assert_impl_all!(SystemClock: Clock, Send, Sync, Clone);
assert_impl_all!(Timestamp: Copy, Send, Sync);
assert_impl_all!(ErrorKind: Copy, Send, Sync);
assert_impl_all!(XError: Send, Sync);

// 业务类型不得 Default（禁止哨兵 0 时间）
assert_not_impl_any!(Timestamp: Default);
// MonotonicInstant 亦禁止 Default
assert_not_impl_any!(MonotonicInstant: Default);
// SystemClock 非 Copy
assert_not_impl_any!(SystemClock: Copy);
// XError 不可 Clone（source chain）
assert_not_impl_any!(XError: Clone);
// ShutdownGuard 消费式 trigger，禁止 Clone
assert_not_impl_any!(ShutdownGuard: Clone);

#[test]
fn public_constructors_and_queries_compile() {
    let _ = XError::invalid("x");
    let _ = XError::missing("m");
    let _ = XError::conflict("c");
    let _ = XError::transient("t");
    let _ = XError::transient_after("t", Duration::from_millis(1));
    let _ = XError::unavailable("u");
    let _ = XError::cancelled("k");
    let _ = XError::deadline_exceeded("d");
    let _ = XError::invariant("i");
    let e = XError::internal("n").with_source(std::io::Error::other("s"));
    assert_eq!(e.kind(), ErrorKind::Internal);
    assert_eq!(e.context(), "n");
    let _ = SystemClock::new().now();
    let _ = Timestamp::from_unix_nanos(1).checked_add(Duration::from_nanos(1));
}

#[test]
fn test_all_public_items_accessible() {
    // error module
    let _ = XError::invalid("test");
    let _ = XError::missing("test");
    let _ = XError::conflict("test");
    let _ = XError::transient("test");
    let _ = XError::transient_after("test", std::time::Duration::from_secs(1));
    let _ = XError::unavailable("test");
    let _ = XError::cancelled("test");
    let _ = XError::deadline_exceeded("test");
    let _ = XError::invariant("test");
    let _ = XError::internal("test");

    let err = XError::invalid("test");
    let _ = err.kind();
    let _ = err.context();
    let _ = err.retry_after();
    let _ = err.is_retryable();
    let _ = err.is_bug();

    let _ = err.with_source(std::io::Error::other("source"));

    // clock module
    let clock = SystemClock::new();
    let _ = clock.now();
    let _ = clock.monotonic();

    let ts = Timestamp::from_unix_nanos(0);
    let _ = ts.as_unix_nanos();
    let _ = ts.checked_add(std::time::Duration::from_secs(1));
    let _ = ts.checked_sub(std::time::Duration::from_secs(1));
    let _ = ts.checked_duration_since(ts);

    let mi = MonotonicInstant::from_clock_elapsed(std::time::Duration::ZERO);
    let _ = mi.checked_duration_since(mi);

    // lifecycle module
    let _ = kernel::ComponentState::Created.can_transition_to(kernel::ComponentState::Starting);
    let _ = kernel::ComponentState::Created.try_transition(kernel::ComponentState::Starting);

    let (_guard, _signal) = kernel::ShutdownSignal::new();
}
