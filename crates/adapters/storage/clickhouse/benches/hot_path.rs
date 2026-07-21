//! clickhousex 核心路径：ping + execute SELECT 1（有服务时）。
use std::time::Instant;

use clickhousex::{ClickHouseConfig, ClickHousePool};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 100 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let cfg = ClickHouseConfig::from_env();
    let Ok(pool) = ClickHousePool::connect(cfg).await else {
        println!("bench_clickhousex_select1: skipped (no clickhouse)");
        return;
    };
    pool.ping().await.expect("ping");
    let start = Instant::now();
    for _ in 0..n {
        pool.execute("SELECT 1").await.expect("select");
    }
    let elapsed = start.elapsed();
    println!(
        "bench_clickhousex_select1: iters={n} total={elapsed:?} per_iter={:?}",
        elapsed / n.max(1)
    );
    let _ = pool.close().await;
}
