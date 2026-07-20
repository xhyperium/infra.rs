//! API 编译检查。
//!
//! 此文件仅验证 kernel crate 被下游使用时能成功编译。

use kernel::*;

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
    let _ = ComponentState::Created.can_transition_to(ComponentState::Starting);
    let _ = ComponentState::Created.try_transition(ComponentState::Starting);

    let (_guard, _signal) = ShutdownSignal::new();
}
