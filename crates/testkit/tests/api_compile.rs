//! SPEC-TESTKIT-002 §13.5 / §7.12 / §7.13 — 编译期面。
//!
//! `static_assertions` 证明：
//! - `ManualClock: Send + Sync`
//! - `ManualClock: !Default` / `!Clone`
//! - 相关错误/故障/快照类型的合理边界
//!
//! 退役符号（`xlib_test!` / `mock!` / `FixtureBuilder` / provider macro）
//! 以源码结构守卫（`public_surface`）保证不回流，而非 trybuild。

use std::time::Duration;

use kernel::{Clock, MonotonicInstant, Timestamp};
use static_assertions::{assert_impl_all, assert_not_impl_any};
use testkit::{ManualClock, ManualClockError, ManualClockFault, ManualClockSnapshot};

assert_impl_all!(ManualClock: Clock, Send, Sync);
assert_impl_all!(ManualClockFault: Copy, Send, Sync);
assert_impl_all!(ManualClockError: Copy, Send, Sync);
assert_impl_all!(ManualClockSnapshot: Copy, Send, Sync);

// §7.6 / §7.12 / §13.5：禁止 Default（哨兵 epoch）与 Clone（伪独立时间线）
assert_not_impl_any!(ManualClock: Default);
assert_not_impl_any!(ManualClock: Clone);
assert_not_impl_any!(ManualClock: Copy);

#[test]
fn public_constructors_and_control_surface_compile() {
    let c = ManualClock::new(Timestamp::from_unix_nanos(1));
    let _ =
        ManualClock::with_monotonic_elapsed(Timestamp::from_unix_nanos(2), Duration::from_nanos(3));
    c.set_wall(Timestamp::from_unix_nanos(10)).expect("set_wall");
    let _ = c.advance_wall(Duration::from_nanos(1)).expect("advance_wall");
    let _ = c.rewind_wall(Duration::from_nanos(1)).expect("rewind_wall");
    c.set_monotonic_elapsed(Duration::from_nanos(5)).expect("set_mono");
    let _mono: MonotonicInstant = c.advance_monotonic(Duration::from_nanos(1)).expect("adv_mono");
    c.set_wall_fault(ManualClockFault::Unavailable).expect("fault");
    assert!(c.wall_fault().expect("query").is_some());
    c.clear_wall_fault().expect("clear");
    let snap: ManualClockSnapshot = c.snapshot().expect("snap");
    let _: Timestamp = snap.wall();
    let _: Duration = snap.monotonic_elapsed();
    let _: Option<ManualClockFault> = snap.wall_fault();
    let _: Result<Timestamp, _> = c.now();
    let _: MonotonicInstant = c.monotonic();
}

#[test]
fn manual_clock_error_variants_are_distinct_and_displayable() {
    let variants = [
        ManualClockError::WallOverflow,
        ManualClockError::MonotonicOverflow,
        ManualClockError::MonotonicRegression,
        ManualClockError::Synchronization,
    ];
    for (i, a) in variants.iter().enumerate() {
        assert!(!a.to_string().is_empty());
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}
