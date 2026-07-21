//! natsx 热路径：配置构造（离线）。
use std::hint::black_box;
use std::time::Instant;

use natsx::NatsConfig;

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 10_000 } else { 200_000 }
}

fn main() {
    let n = iters();
    for _ in 0..n.min(100) {
        black_box(NatsConfig::default());
    }
    let start = Instant::now();
    let mut acc = 0usize;
    for _ in 0..n {
        let c = NatsConfig::default();
        acc = acc.wrapping_add(c.url.len());
        black_box(c.validate().is_ok());
    }
    let elapsed = start.elapsed();
    println!(
        "bench_natsx: iters={n} total={elapsed:?} per_iter={:?} acc={}",
        elapsed / n,
        black_box(acc)
    );
}
