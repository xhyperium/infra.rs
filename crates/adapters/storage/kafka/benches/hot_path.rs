//! kafkax 核心路径：produce（有 broker 时）或 config+id（离线）。
//! `cargo test --all-targets` 也会编译运行本 bench：connect 必须有界超时。
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use kafkax::{KafkaConfig, KafkaPool, encode_bus_id};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 200 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let start = Instant::now();
    let mut acc = 0usize;
    for i in 0..n.max(1000) {
        acc = acc.wrapping_add(encode_bus_id("bench", (i % 12) as i32, i as i64).len());
    }
    println!("bench_kafkax_encode_id: iters={} total={:?} acc={acc}", n.max(1000), start.elapsed());

    let cfg = KafkaConfig::from_env();
    let connect = tokio::time::timeout(Duration::from_secs(3), KafkaPool::connect(cfg));
    let Ok(Ok(pool)) = connect.await else {
        println!("bench_kafkax_produce: skipped (no broker / timeout)");
        return;
    };
    let topic = format!("infra-bench-kafkax-{}", std::process::id());
    let _ = pool.ensure_topic(&topic, 1, 1).await;
    let start = Instant::now();
    for i in 0..n {
        let payload = Bytes::from(format!("bench-{i}"));
        if pool.producer().publish(&topic, payload).await.is_err() {
            println!("bench_kafkax_produce: publish failed mid-run");
            break;
        }
    }
    let elapsed = start.elapsed();
    println!("bench_kafkax_produce: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n.max(1));
    let _ = pool.close(Duration::from_secs(3)).await;
}
