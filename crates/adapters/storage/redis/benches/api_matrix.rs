//! redisx 多 API 基准：ping / set+get / mset+mget / pipeline / delete。
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   cargo bench -p redisx --bench api_matrix
//! ```

use std::time::{Duration, Instant};

use redisx::{RedisClient, RedisConfig, RedisPool};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("rt");
    rt.block_on(async {
        let cfg = match RedisConfig::from_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("skip bench: config: {e}");
                return;
            }
        };
        let pool = match RedisPool::connect(cfg).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skip bench: connect: {e}");
                return;
            }
        };
        if let Err(e) = pool.ping().await {
            eprintln!("skip bench: ping: {e}");
            let _ = pool.close(Duration::from_secs(1)).await;
            return;
        }
        let client = pool.client();
        let pid = std::process::id();
        let prefix = format!("redisx-bench-matrix:{pid}:");

        bench_ping(&pool, 200).await;
        bench_set_get(&client, &prefix, 1_000, 64).await;
        bench_mset_mget(&client, &prefix, 200, 16).await;
        bench_pipeline(&client, &prefix, 200, 16).await;

        let _ = pool.close(Duration::from_secs(2)).await;
    });
}

async fn bench_ping(pool: &RedisPool, iters: usize) {
    for _ in 0..20 {
        let _ = pool.ping().await;
    }
    let t0 = Instant::now();
    for _ in 0..iters {
        pool.ping().await.expect("ping");
    }
    let elapsed = t0.elapsed();
    println!(
        "ping iters={iters} elapsed={elapsed:?} rtt_avg≈{:.3}ms",
        elapsed.as_secs_f64() * 1000.0 / iters as f64
    );
}

async fn bench_set_get(client: &RedisClient, prefix: &str, iters: usize, payload: usize) {
    let key = format!("{prefix}sg");
    let body = vec![0xAB; payload];
    for _ in 0..50 {
        let _ = client.set(&key, body.clone(), None).await;
        let _ = client.get(&key).await;
    }
    let t0 = Instant::now();
    for i in 0..iters {
        client.set(&key, body.clone(), None).await.unwrap_or_else(|e| panic!("set {i}: {e}"));
        let v = client.get(&key).await.unwrap_or_else(|e| panic!("get {i}: {e}"));
        assert_eq!(v.as_deref(), Some(body.as_slice()));
    }
    let elapsed = t0.elapsed();
    let ops = (iters * 2) as f64 / elapsed.as_secs_f64();
    println!(
        "set+get iters={iters} payload={payload}B elapsed={elapsed:?} throughput≈{ops:.0} ops/s"
    );
    let _ = client.delete(&key).await;
}

async fn bench_mset_mget(client: &RedisClient, prefix: &str, rounds: usize, batch: usize) {
    let keys: Vec<String> = (0..batch).map(|i| format!("{prefix}m{i}")).collect();
    let vals: Vec<Vec<u8>> = (0..batch).map(|i| format!("v{i}").into_bytes()).collect();
    let pairs: Vec<(&str, &[u8])> =
        keys.iter().zip(vals.iter()).map(|(k, v)| (k.as_str(), v.as_slice())).collect();
    let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();

    for _ in 0..10 {
        let _ = client.mset(&pairs).await;
        let _ = client.mget(&key_refs).await;
    }
    let t0 = Instant::now();
    for _ in 0..rounds {
        client.mset(&pairs).await.expect("mset");
        let got = client.mget(&key_refs).await.expect("mget");
        assert_eq!(got.len(), batch);
    }
    let elapsed = t0.elapsed();
    let keys_ops = (rounds * batch * 2) as f64 / elapsed.as_secs_f64();
    println!("mset+mget rounds={rounds} batch={batch} elapsed={elapsed:?} key_ops≈{keys_ops:.0}/s");
    for k in &keys {
        let _ = client.delete(k).await;
    }
}

async fn bench_pipeline(client: &RedisClient, prefix: &str, rounds: usize, batch: usize) {
    let keys: Vec<String> = (0..batch).map(|i| format!("{prefix}p{i}")).collect();
    let items: Vec<(&str, Vec<u8>)> =
        keys.iter().enumerate().map(|(i, k)| (k.as_str(), format!("p{i}").into_bytes())).collect();

    for _ in 0..10 {
        let _ = client.pipeline_set(&items, Some(Duration::from_secs(60))).await;
    }
    let t0 = Instant::now();
    for _ in 0..rounds {
        client.pipeline_set(&items, Some(Duration::from_secs(60))).await.expect("pipe");
    }
    let elapsed = t0.elapsed();
    let keys_ops = (rounds * batch) as f64 / elapsed.as_secs_f64();
    println!(
        "pipeline_set rounds={rounds} batch={batch} elapsed={elapsed:?} key_ops≈{keys_ops:.0}/s"
    );
    for k in &keys {
        let _ = client.delete(k).await;
    }
}
