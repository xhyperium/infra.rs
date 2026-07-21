//! bootstrap 热路径：build_app + instrumentation。
use std::hint::black_box;
use std::time::Instant;

use bootstrap::Bootstrap;

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 500 } else { 20_000 }
}

fn main() {
    let n = iters();
    for _ in 0..n.min(10) {
        let app = Bootstrap::new().build_app();
        app.context().instrumentation().record_retry("b", 1);
        black_box(());
    }
    let start = Instant::now();
    for _ in 0..n {
        let app = Bootstrap::new().build_app();
        app.context().instrumentation().record_retry("bench", 1);
        black_box(());
        let (ctx, sc) = app.into_parts();
        sc.trigger();
        black_box(ctx.shutdown_signal().is_triggered());
    }
    let elapsed = start.elapsed();
    println!("bench_bootstrap_build: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n);
}
