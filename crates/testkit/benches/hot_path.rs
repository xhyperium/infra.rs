//! testkit 热路径：ManualClock 推进。
use std::hint::black_box;
use std::time::{Duration, Instant};

use kernel::{Clock, Timestamp};
use testkit::ManualClock;

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 100_000 }
}

fn main() {
    let n = iters();
    let clock = ManualClock::new(Timestamp::from_unix_nanos(1_000_000_000));
    for _ in 0..n.min(20) {
        let _ = clock.advance_wall(Duration::from_nanos(1));
    }
    let start = Instant::now();
    for _ in 0..n {
        black_box(clock.advance_wall(Duration::from_nanos(1)).expect("wall"));
        black_box(clock.advance_monotonic(Duration::from_nanos(1)).expect("mono"));
        black_box(clock.now().expect("now"));
    }
    let elapsed = start.elapsed();
    println!("bench_testkit_manual_clock: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n);
}
