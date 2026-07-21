//! ManualClock 合同测试（SPEC-TESTKIT-002 §13.1 / §13.2 ManualClockDeterminism）。

use std::time::Duration;

use kernel::{Clock, ClockError, MonotonicInstant, Timestamp};
use testkit::{ManualClock, ManualClockError, ManualClockFault};

fn ts(n: i64) -> Timestamp {
    Timestamp::from_unix_nanos(n)
}

#[test]
fn no_real_time_drift_between_reads() {
    let c = ManualClock::new(ts(1_700_000_000_000_000_000));
    let a = c.now().unwrap();
    let b = c.now().unwrap();
    assert_eq!(a, b);
    assert_eq!(c.monotonic(), c.monotonic());
}

#[test]
fn common_clock_now_or_explicit_error() {
    let c = ManualClock::new(ts(10));
    assert_eq!(c.now().unwrap().as_unix_nanos(), 10);
    c.set_wall_fault(ManualClockFault::Unavailable).unwrap();
    assert!(matches!(c.now(), Err(ClockError::Unavailable)));
}

#[test]
fn wall_may_rewind() {
    let c = ManualClock::new(ts(100));
    c.rewind_wall(Duration::from_nanos(40)).unwrap();
    assert_eq!(c.now().unwrap().as_unix_nanos(), 60);
}

#[test]
fn monotonic_non_decreasing() {
    let c = ManualClock::new(ts(0));
    let m0 = c.monotonic();
    c.advance_monotonic(Duration::from_nanos(1)).unwrap();
    let m1 = c.monotonic();
    assert!(m1.checked_duration_since(m0).is_some());
    assert!(matches!(
        c.set_monotonic_elapsed(Duration::from_nanos(0)),
        Err(ManualClockError::MonotonicRegression)
    ));
}

#[test]
fn mono_from_elapsed_roundtrip() {
    let c = ManualClock::with_monotonic_elapsed(ts(1), Duration::from_millis(2));
    let m = c.monotonic();
    // 必须同 domain：from_clock_elapsed 默认 PROCESS，ManualClock 为独立 domain
    let origin = MonotonicInstant::from_clock_elapsed_in(Duration::ZERO, c.domain());
    assert_eq!(m.checked_duration_since(origin).unwrap(), Duration::from_millis(2));
    assert!(
        m.checked_duration_since(MonotonicInstant::from_clock_elapsed(Duration::ZERO)).is_none()
    );
}

/// §13.2：wall 与 monotonic 独立（ManualClockDeterminismContract）。
#[test]
fn wall_and_monotonic_are_independent() {
    let c = ManualClock::new(ts(100));
    c.advance_monotonic(Duration::from_nanos(1_000)).unwrap();
    c.rewind_wall(Duration::from_nanos(50)).unwrap();
    assert_eq!(c.now().unwrap().as_unix_nanos(), 50);
    assert_eq!(c.snapshot().unwrap().monotonic_elapsed(), Duration::from_nanos(1_000));
}

/// §13.2：两次读取之间无 control call 时 wall/mono 值不变。
#[test]
fn two_reads_stable_without_control_call() {
    let c = ManualClock::new(ts(42));
    c.advance_monotonic(Duration::from_nanos(9)).unwrap();
    let w1 = c.now().unwrap();
    let m1 = c.monotonic();
    let w2 = c.now().unwrap();
    let m2 = c.monotonic();
    assert_eq!(w1, w2);
    assert_eq!(m1, m2);
}
