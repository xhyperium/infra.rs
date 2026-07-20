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
    let origin = MonotonicInstant::from_clock_elapsed(Duration::ZERO);
    assert_eq!(m.checked_duration_since(origin).unwrap(), Duration::from_millis(2));
}
