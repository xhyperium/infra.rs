//! ManualClock 公开面全量：每个公开方法断言返回值。

use std::time::Duration;

use kernel::{Clock, Timestamp};
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
    assert!(clock.now().is_err());
    clock.clear_wall_fault().unwrap();
    assert_eq!(clock.wall_fault().unwrap(), None);
    assert!(clock.now().is_ok());

    let snap: ManualClockSnapshot = clock.snapshot().unwrap();
    assert_eq!(snap.wall().as_unix_nanos(), 2_000);
    assert_eq!(snap.monotonic_elapsed(), Duration::from_millis(5));
    assert_eq!(snap.wall_fault(), None);

    let c2 =
        ManualClock::with_monotonic_elapsed(Timestamp::from_unix_nanos(10), Duration::from_secs(1));
    assert_eq!(c2.now().unwrap().as_unix_nanos(), 10);
    // 不同实例 domain 可不同
    assert_ne!(c2.domain(), domain);

    let _ = format!("{:?}", ManualClockError::Synchronization);
    let _ = format!("{:?}", ManualClockFault::Unavailable);
    let _ = format!("{:?}", ManualClockFault::Overflow);
    let _ = format!("{:?}", ManualClockFault::BeforeUnixEpoch);
}
