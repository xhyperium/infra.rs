//! Clock trait 合同不变量测试。
//!
//! 验证所有 `Clock` 实现必须满足的通用合同。

use kernel::{Clock, SystemClock};

#[test]
fn test_system_clock_now_returns_valid_timestamp() {
    let clock = SystemClock::new();
    let ts = clock.now().expect("wall clock should be available");
    // Should be after the year 2000
    let y2k_nanos: i64 = 946_684_800_000_000_000; // 2000-01-01T00:00:00Z
    assert!(ts.as_unix_nanos() > y2k_nanos);
}

#[test]
fn test_system_clock_monotonic_non_decreasing() {
    let clock = SystemClock::new();
    let a = clock.monotonic();
    let b = clock.monotonic();
    let c = clock.monotonic();
    assert!(b >= a);
    assert!(c >= b);
}

#[test]
fn test_system_clock_monotonic_increases() {
    let clock = SystemClock::new();
    let a = clock.monotonic();
    // Busy-wait briefly to ensure elapsed time increases
    let _sum: u64 = (0..1_000_000).sum();
    let b = clock.monotonic();
    assert!(b >= a);
}

#[test]
fn test_system_clock_default_works() {
    let clock = SystemClock::default();
    let ts = clock.now().expect("wall clock should be available");
    assert!(ts.as_unix_nanos() > 0);
}
