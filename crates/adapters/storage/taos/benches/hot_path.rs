//! taosx 核心路径：ping（有服务时）。connect 有界超时。
use std::time::Duration;
use std::time::Instant;

use taosx::{TaosConfig, TaosPool};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 100 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let cfg = TaosConfig::from_env();
    let connect = tokio::time::timeout(Duration::from_secs(3), TaosPool::connect(cfg));
    let Ok(Ok(pool)) = connect.await else {
        println!("bench_taosx_ping: skipped (no taos / timeout)");
        return;
    };
    let start = Instant::now();
    for _ in 0..n {
        if pool.ping().await.is_err() {
            break;
        }
    }
    let elapsed = start.elapsed();
    println!("bench_taosx_ping: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n.max(1));
    let _ = pool.close().await;
}
