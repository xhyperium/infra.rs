//! clickhousex 核心路径：SELECT 1（有服务时）。connect/ping 有界超时。
use std::time::Duration;
use std::time::Instant;

use clickhousex::{ClickHouseConfig, ClickHousePool};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 100 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let cfg = ClickHouseConfig::from_env();
    let connect = tokio::time::timeout(Duration::from_secs(3), ClickHousePool::connect(cfg));
    let Ok(Ok(pool)) = connect.await else {
        println!("bench_clickhousex_select1: skipped (no clickhouse / timeout)");
        return;
    };
    if tokio::time::timeout(Duration::from_secs(2), pool.ping()).await.is_err() {
        println!("bench_clickhousex_select1: skipped (ping timeout)");
        return;
    }
    let start = Instant::now();
    for _ in 0..n {
        if pool.execute("SELECT 1").await.is_err() {
            break;
        }
    }
    let elapsed = start.elapsed();
    println!(
        "bench_clickhousex_select1: iters={n} total={elapsed:?} per_iter={:?}",
        elapsed / n.max(1)
    );
    let _ = pool.close().await;
}
