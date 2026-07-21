//! kernel 热路径基准：墙钟读取 + Timestamp 运算。
use std::hint::black_box;
use std::time::{Duration, Instant};

use kernel::{Clock, SystemClock, Timestamp};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 200_000 }
}

fn main() {
    let n = iters();
    let clock = SystemClock::new();
    for _ in 0..n.min(50) {
        let _ = black_box(clock.now());
    }
    let start = Instant::now();
    let mut acc = 0i64;
    for i in 0..n {
        let ts = clock.now().expect("now");
        let t = Timestamp::from_unix_nanos(ts.as_unix_nanos().wrapping_add(i as i64));
        acc = acc.wrapping_add(
            t.checked_add(Duration::from_nanos(1)).map(|x| x.as_unix_nanos()).unwrap_or(0),
        );
        black_box(clock.monotonic());
    }
    let elapsed = start.elapsed();
    println!(
        "bench_kernel_clock: iters={n} total={elapsed:?} per_iter={:?} acc={}",
        elapsed / n,
        black_box(acc)
    );
}
