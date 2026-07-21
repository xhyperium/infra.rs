//! kafkax 热路径：配置解析 + bus id 编码（离线，无 broker）。
use std::hint::black_box;
use std::time::Instant;

use kafkax::{KafkaConfig, encode_bus_id};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 10_000 } else { 200_000 }
}

fn main() {
    let n = iters();
    // 预热
    for i in 0..n.min(100) {
        let _ = encode_bus_id("bench", (i % 12) as i32, i as i64);
        let _ = KafkaConfig::default();
    }

    let start = Instant::now();
    let mut acc = 0usize;
    for i in 0..n {
        let id = encode_bus_id("bench-topic", (i % 12) as i32, i as i64);
        acc = acc.wrapping_add(id.len());
        if i % 1024 == 0 {
            let c = KafkaConfig::default();
            acc = acc.wrapping_add(c.brokers.len());
            black_box(c.security_protocol());
        }
    }
    let elapsed = start.elapsed();
    println!(
        "bench_kafkax: iters={n} total={elapsed:?} per_iter={:?} acc={}",
        elapsed / n,
        black_box(acc)
    );
}
