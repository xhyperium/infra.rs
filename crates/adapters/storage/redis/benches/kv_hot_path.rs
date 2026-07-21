//! GET/SET 热路径微基准（`harness = false`）。
//!
//! ```bash
//! cargo bench -p redisx --bench kv_hot_path
//! # 需要可达 Redis：FOUNDATIONX_REDISX_* 或 REDIS_URL
//! ```

use std::time::{Duration, Instant};

use redisx::{RedisClient, RedisConfig, RedisPool};

fn env_or_skip() -> Option<RedisConfig> {
    match RedisConfig::from_env() {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("skip bench: config error: {e}");
            None
        }
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("runtime");

    rt.block_on(async {
        let Some(cfg) = env_or_skip() else {
            return;
        };
        let pool = match RedisPool::connect(cfg).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skip bench: connect failed: {e}");
                return;
            }
        };
        if let Err(e) = pool.ping().await {
            eprintln!("skip bench: ping failed (need auth/env?): {e}");
            let _ = pool.close(Duration::from_secs(1)).await;
            return;
        }
        let client = pool.client();
        run_set_get(&client, 1_000).await;
        let _ = pool.close(Duration::from_secs(2)).await;
    });
}

async fn run_set_get(client: &RedisClient, iters: usize) {
    let key = format!("redisx-bench:{}:k", std::process::id());
    let payload = vec![0u8; 64];

    // warmup
    for _ in 0..50 {
        let _ = client.set(&key, payload.clone(), None).await;
        let _ = client.get(&key).await;
    }

    let start = Instant::now();
    for i in 0..iters {
        client.set(&key, payload.clone(), None).await.unwrap_or_else(|e| panic!("set {i}: {e}"));
        let v = client.get(&key).await.unwrap_or_else(|e| panic!("get {i}: {e}"));
        assert_eq!(v.as_deref(), Some(payload.as_slice()));
    }
    let elapsed = start.elapsed();
    let ops = (iters * 2) as f64 / elapsed.as_secs_f64();
    println!(
        "kv_hot_path set+get iters={iters} elapsed={elapsed:?} throughput≈{ops:.0} ops/s payload=64B"
    );

    let _ = client.delete(&key).await;
}
