//! ManualClock 公开面全量：每个公开方法断言返回值。

use std::time::Duration;

use kernel::{Clock, ClockError, Timestamp};
use testkit::{ManualClock, ManualClockError, ManualClockFault, ManualClockSnapshot};

#[test]
fn manual_clock_full_surface() {
    let clock = ManualClock::new(Timestamp::from_unix_nanos(1_000));
    let domain = clock.domain();
    assert_eq!(clock.now().unwrap().as_unix_nanos(), 1_000);

    clock.set_wall(Timestamp::from_unix_nanos(2_000)).unwrap();
    assert_eq!(clock.now().unwrap().as_unix_nanos(), 2_000);

    let advanced = clock.advance_wall(Duration::from_nanos(5)).unwrap();
    assert_eq!(advanced.as_unix_nanos(), 2_005);

    let rewound = clock.rewind_wall(Duration::from_nanos(5)).unwrap();
    assert_eq!(rewound.as_unix_nanos(), 2_000);

    clock.set_monotonic_elapsed(Duration::from_millis(3)).unwrap();
    let mono = clock.advance_monotonic(Duration::from_millis(2)).unwrap();
    assert_eq!(mono.domain(), domain);

    clock.set_wall_fault(ManualClockFault::Unavailable).unwrap();
    assert_eq!(clock.wall_fault().unwrap(), Some(ManualClockFault::Unavailable));
    assert!(matches!(clock.now().unwrap_err(), ClockError::Unavailable));
    clock.clear_wall_fault().unwrap();
    assert_eq!(clock.wall_fault().unwrap(), None);
    assert!(clock.now().is_ok());

    // 全部 fault 变体 → ClockError 映射（经 now）
    for fault in [
        ManualClockFault::BeforeUnixEpoch,
        ManualClockFault::Overflow,
        ManualClockFault::Unavailable,
    ] {
        clock.set_wall_fault(fault).unwrap();
        let err = clock.now().unwrap_err();
        match fault {
            ManualClockFault::BeforeUnixEpoch => assert_eq!(err, ClockError::BeforeUnixEpoch),
            ManualClockFault::Overflow => assert_eq!(err, ClockError::Overflow),
            ManualClockFault::Unavailable => assert_eq!(err, ClockError::Unavailable),
            other => panic!("unexpected ManualClockFault variant: {other:?}"),
        }
        clock.clear_wall_fault().unwrap();
    }

    let snap: ManualClockSnapshot = clock.snapshot().unwrap();
    assert_eq!(snap.wall().as_unix_nanos(), 2_000);
    assert_eq!(snap.monotonic_elapsed(), Duration::from_millis(5));
    assert_eq!(snap.wall_fault(), None);

    let c2 =
        ManualClock::with_monotonic_elapsed(Timestamp::from_unix_nanos(10), Duration::from_secs(1));
    assert_eq!(c2.now().unwrap().as_unix_nanos(), 10);
    // 不同实例 domain 可不同
    assert_ne!(c2.domain(), domain);

    // Clock trait
    let as_clock: &dyn Clock = &clock;
    assert_eq!(as_clock.now().unwrap().as_unix_nanos(), 2_000);
    assert_eq!(as_clock.monotonic().domain(), domain);

    // 控制路径错误：墙钟溢出 / 单调回退
    let max_wall = ManualClock::new(Timestamp::from_unix_nanos(i64::MAX));
    assert_eq!(
        max_wall.advance_wall(Duration::from_nanos(1)).unwrap_err(),
        ManualClockError::WallOverflow
    );
    assert_eq!(
        max_wall.rewind_wall(Duration::from_nanos(1)).map(|t| t.as_unix_nanos()).unwrap(),
        i64::MAX - 1
    );
    // rewind 再越过 i64::MIN 边界
    let min_wall = ManualClock::new(Timestamp::from_unix_nanos(i64::MIN));
    assert_eq!(
        min_wall.rewind_wall(Duration::from_nanos(1)).unwrap_err(),
        ManualClockError::WallOverflow
    );

    let mono_reg = ManualClock::with_monotonic_elapsed(
        Timestamp::from_unix_nanos(0),
        Duration::from_millis(10),
    );
    assert_eq!(
        mono_reg.set_monotonic_elapsed(Duration::from_millis(5)).unwrap_err(),
        ManualClockError::MonotonicRegression
    );

    // Display 中文错误
    assert!(ManualClockError::WallOverflow.to_string().contains("墙钟"));
    assert!(ManualClockError::MonotonicOverflow.to_string().contains("单调"));
    assert!(ManualClockError::MonotonicRegression.to_string().contains("回退"));
    assert!(ManualClockError::Synchronization.to_string().contains("锁"));
    let _ = format!("{:?}", ManualClockError::Synchronization);
    let _ = format!("{:?}", ManualClockFault::Unavailable);
    let _ = format!("{:?}", ManualClockFault::Overflow);
    let _ = format!("{:?}", ManualClockFault::BeforeUnixEpoch);
}
