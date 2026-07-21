//! taosx 核心路径：ping + SELECT SERVER_STATUS（有服务时）。
use std::time::Instant;

use taosx::{TaosConfig, TaosPool};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 20 } else { 100 }
}

#[tokio::main]
async fn main() {
    let n = iters();
    let cfg = TaosConfig::from_env();
    let Ok(pool) = TaosPool::connect(cfg).await else {
        println!("bench_taosx_ping: skipped (no taos)");
        return;
    };
    let start = Instant::now();
    for _ in 0..n {
        pool.ping().await.expect("ping");
    }
    let elapsed = start.elapsed();
    println!("bench_taosx_ping: iters={n} total={elapsed:?} per_iter={:?}", elapsed / n.max(1));
    let _ = pool.close().await;
}
