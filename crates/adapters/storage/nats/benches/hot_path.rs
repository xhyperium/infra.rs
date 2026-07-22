//! natsx 核心路径：publish（有 NATS 时）。connect 有界超时。
use std::time::Duration;
use std::time::Instant;

use bytes::Bytes;
use natsx::{NatsConfig, NatsPool};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 50 } else { 500 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let cfg = match NatsConfig::from_env() {
        Ok(cfg) => cfg,
        Err(error) => {
            eprintln!("bench_natsx_publish: skipped (invalid config: {error})");
            return;
        }
    };
    let connect = tokio::time::timeout(Duration::from_secs(3), NatsPool::connect(cfg));
    let Ok(Ok(pool)) = connect.await else {
        println!("bench_natsx_publish: skipped (no nats / timeout)");
        return;
    };
    let subject = format!("infra.bench.natsx.{}", std::process::id());
    let start = Instant::now();
    for i in 0..n {
        if pool.publish(&subject, Bytes::from(format!("b{i}"))).await.is_err() {
            println!("bench_natsx_publish: publish failed mid-run");
            break;
        }
    }
    let elapsed = start.elapsed();
    println!("bench_natsx_publish: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n.max(1));
    let _ = pool.close().await;
}
