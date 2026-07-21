//! 最小消费者路径：ManualClock 墙钟推进与 fault 注入。
//!
//! ```bash
//! cargo run -p testkit --example basic
//! ```

use std::time::Duration;

use kernel::{Clock, Timestamp};
use testkit::{ManualClock, ManualClockFault};

fn main() {
    let clock = ManualClock::new(Timestamp::from_unix_nanos(0));
    assert_eq!(clock.now().expect("now").as_unix_nanos(), 0);

    let advanced = clock.advance_wall(Duration::from_secs(1)).expect("advance");
    assert_eq!(advanced.as_unix_nanos(), 1_000_000_000);
    assert_eq!(clock.now().expect("now").as_unix_nanos(), 1_000_000_000);

    let mono = clock.advance_monotonic(Duration::from_millis(5)).expect("mono");
    assert_eq!(mono.domain(), clock.domain());

    clock.set_wall_fault(ManualClockFault::Unavailable).expect("fault");
    assert!(clock.now().is_err());
    clock.clear_wall_fault().expect("clear");
    assert!(clock.now().is_ok());

    let snap = clock.snapshot().expect("snapshot");
    assert_eq!(snap.wall().as_unix_nanos(), 1_000_000_000);
    assert_eq!(snap.monotonic_elapsed(), Duration::from_millis(5));

    println!(
        "testkit-consumer: ok wall_ns={} mono_ms={}",
        snap.wall().as_unix_nanos(),
        snap.monotonic_elapsed().as_millis()
    );
}
