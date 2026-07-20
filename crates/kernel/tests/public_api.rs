//! Public API smoke: ErrorKind / Clock / Shutdown surface (SPEC-KERNEL-002 §11).
use std::time::Duration;

use kernel::{
    BoxError, Clock, ClockError, ComponentState, ErrorKind, MonotonicInstant, ShutdownSignal,
    SystemClock, Timestamp, XError, XResult,
};

#[test]
fn test_public_api_basics() {
    // Error
    let err: XError = XError::invalid("test");
    let _kind: ErrorKind = err.kind();
    let _ctx: &str = err.context();
    let _retry: Option<std::time::Duration> = err.retry_after();
    let _boxed: BoxError = Box::new(std::io::Error::other("oops"));
    let _result: XResult<()> = Err(err);

    // Clock
    let clock = SystemClock::new();
    let _ts: Result<Timestamp, ClockError> = clock.now();
    let _mono: MonotonicInstant = clock.monotonic();

    // Lifecycle
    let _state: ComponentState = ComponentState::Created;
    let (_guard, _signal) = ShutdownSignal::new();
}

#[test]
fn error_kind_query_surface() {
    assert_eq!(XError::missing("x").kind(), ErrorKind::Missing);
    assert!(XError::transient("t").is_retryable());
    assert!(XError::invariant("i").is_bug());
    assert_eq!(
        XError::transient_after("a", Duration::from_millis(1)).retry_after(),
        Some(Duration::from_millis(1))
    );
}

/// 杀变体：`context()` → 固定值；`Debug::fmt` → 空。
#[test]
fn error_context_and_debug_are_observably_correct() {
    const TOKEN: &str = "unique-context-token-not-xyzzy";
    let owned = XError::invalid(TOKEN);
    assert_eq!(owned.context(), TOKEN);
    assert_ne!(owned.context(), "xyzzy");

    let e = XError::missing("needle-in-debug-output");
    let debug = format!("{e:?}");
    assert!(debug.starts_with("XError"), "debug={debug}");
    assert!(debug.contains("needle-in-debug-output"), "debug={debug}");
    assert!(debug.contains("kind"), "debug={debug}");
    assert!(debug.contains("context"), "debug={debug}");
    assert!(debug.len() > 16, "debug={debug}");
}

/// SystemClock 公共面烟雾：墙钟可读 + 单调可推进。
///
/// **interval smoke only / not correctness proof** — 忙等待仅制造可测间隔；
/// 生命周期正确性见 loom（`lifecycle_concurrency_loom`），非本测。
#[test]
fn clock_contract_system() {
    let c = SystemClock::new();
    assert!(c.now().unwrap().as_unix_nanos() > 0);
    let a = c.monotonic();
    // Busy-wait briefly to ensure elapsed time increases (no sleep-as-proof)
    let _sum: u64 = (0..1_000_000).sum();
    let b = c.monotonic();
    assert!(b >= a);
    let t = Timestamp::from_unix_nanos(10);
    assert_eq!(t.checked_sub(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 7);
}

#[test]
fn shutdown_trigger_wakes() {
    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    g.trigger();
    assert!(s.is_triggered());
    s.wait();
}
