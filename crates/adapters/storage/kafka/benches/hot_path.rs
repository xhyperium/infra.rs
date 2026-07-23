//! kafkax 生产基准：encode_id + 多 payload produce + 延迟分位摘要。
//! `cargo test --all-targets` 会编译运行：connect 必须有界超时；无 broker 时 skip produce。
//!
//! ```text
//! cargo bench -p kafkax --bench hot_path -- --quick
//! cargo bench -p kafkax --bench hot_path
//! ```

use std::time::{Duration, Instant};

use bytes::Bytes;
use kafkax::{KafkaConfig, KafkaPool, encode_bus_id};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 200 }
}

fn percentile(sorted_ns: &[u64], p: f64) -> Duration {
    if sorted_ns.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted_ns.len() as f64 - 1.0) * p).round() as usize;
    Duration::from_nanos(sorted_ns[idx.min(sorted_ns.len() - 1)])
}

fn print_latency(label: &str, samples: &mut [u64]) {
    samples.sort_unstable();
    let n = samples.len() as u64;
    let sum: u64 = samples.iter().sum();
    let avg = sum.checked_div(n).unwrap_or(0);
    println!(
        "{label}: n={n} avg={:?} p50={:?} p95={:?} p99={:?}",
        Duration::from_nanos(avg),
        percentile(samples, 0.50),
        percentile(samples, 0.95),
        percentile(samples, 0.99)
    );
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

    let cfg = match KafkaConfig::from_env() {
        Ok(cfg) => cfg,
        Err(error) => {
            eprintln!("bench_kafkax_produce: skipped (invalid config: {error})");
            return;
        }
    };
    let connect = tokio::time::timeout(Duration::from_secs(3), KafkaPool::connect(cfg));
    let Ok(Ok(pool)) = connect.await else {
        println!("bench_kafkax_produce: skipped (no broker / timeout)");
        return;
    };
    let topic = format!("infra-bench-kafkax-{}", std::process::id());
    let _ = pool.ensure_topic(&topic, 1, 1).await;

    for (label, size) in [("100B", 100usize), ("1KiB", 1024), ("1MiB", 1024 * 1024)] {
        let payload_template = vec![0xAB; size];
        let count = if size >= 1024 * 1024 { n.min(5) } else { n };
        let mut samples = Vec::with_capacity(count as usize);
        let wall = Instant::now();
        for i in 0..count {
            let mut body = payload_template.clone();
            body[0] = (i % 255) as u8;
            let t0 = Instant::now();
            match pool.producer().publish(&topic, Bytes::from(body)).await {
                Ok(_) => samples.push(t0.elapsed().as_nanos() as u64),
                Err(e) => {
                    println!("bench_kafkax_produce_{label}: publish failed: {e}");
                    break;
                }
            }
        }
        let elapsed = wall.elapsed();
        println!(
            "bench_kafkax_produce_{label}: iters={} wall={elapsed:?} throughput_approx={:.1}/s",
            samples.len(),
            samples.len() as f64 / elapsed.as_secs_f64().max(1e-9)
        );
        print_latency(&format!("bench_kafkax_latency_{label}"), &mut samples);
    }

    let _ = pool.close(Duration::from_secs(3)).await;
}
