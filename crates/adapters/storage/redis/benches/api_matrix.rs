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
        bench_hash_list(&client, &prefix, 400).await;
        bench_streams(&client, &prefix, 200).await;
        bench_multi_exec(&client, &prefix, 100).await;
        bench_readiness(&pool, 100).await;

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

async fn bench_hash_list(client: &RedisClient, prefix: &str, iters: usize) {
    let h = format!("{prefix}h");
    let l = format!("{prefix}l");
    for _ in 0..20 {
        let _ = client.hset(&h, "f", b"v".to_vec()).await;
        let _ = client.hget(&h, "f").await;
        let _ = client.lpush(&l, b"x".to_vec()).await;
    }
    let t0 = Instant::now();
    for i in 0..iters {
        client.hset(&h, "f", format!("v{i}").into_bytes()).await.expect("hset");
        let _ = client.hget(&h, "f").await.expect("hget");
        client.lpush(&l, b"x".to_vec()).await.expect("lpush");
    }
    let elapsed = t0.elapsed();
    println!(
        "hash+list iters={iters} elapsed={elapsed:?} ops≈{:.0}/s",
        (iters * 3) as f64 / elapsed.as_secs_f64()
    );
    let _ = client.delete(&h).await;
    let _ = client.delete(&l).await;
}

async fn bench_streams(client: &RedisClient, prefix: &str, iters: usize) {
    let s = format!("{prefix}s");
    for _ in 0..10 {
        let _ = client.xadd(&s, &[("f", b"v")]).await;
    }
    // 预热 + 一次显式 id 公共面（xadd_with_id）
    let base_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(1);
    let _ = client.xadd_with_id(&s, &format!("{}-0", base_ms), &[("f", b"fixed")]).await;
    let t0 = Instant::now();
    for i in 0..iters {
        client.xadd(&s, &[("i", format!("{i}").as_bytes())]).await.expect("xadd");
    }
    let _ = client.xlen(&s).await.expect("xlen");
    let _ = client.xrange(&s, "-", "+", Some(10)).await.expect("xrange");
    let elapsed = t0.elapsed();
    println!(
        "streams xadd(+xadd_with_id) iters={iters} elapsed={elapsed:?} ops≈{:.0}/s",
        iters as f64 / elapsed.as_secs_f64()
    );
    let _ = client.delete(&s).await;
}

async fn bench_multi_exec(client: &RedisClient, prefix: &str, rounds: usize) {
    use redisx::TxCmd;
    let k = format!("{prefix}tx");
    let cmds =
        [TxCmd::Set { key: k.clone(), value: b"1".to_vec() }, TxCmd::Incr { key: k.clone() }];
    for _ in 0..10 {
        let _ = client.multi_exec(&cmds).await;
    }
    let t0 = Instant::now();
    for _ in 0..rounds {
        client.multi_exec(&cmds).await.expect("multi");
    }
    let elapsed = t0.elapsed();
    println!(
        "multi_exec rounds={rounds} elapsed={elapsed:?} ops≈{:.0}/s",
        rounds as f64 / elapsed.as_secs_f64()
    );
    let _ = client.delete(&k).await;
}

async fn bench_readiness(pool: &RedisPool, iters: usize) {
    for _ in 0..10 {
        let _ = pool.readiness().await;
    }
    let t0 = Instant::now();
    for _ in 0..iters {
        pool.readiness().await.expect("ready");
    }
    let elapsed = t0.elapsed();
    println!(
        "readiness iters={iters} elapsed={elapsed:?} rtt_avg≈{:.3}ms",
        elapsed.as_secs_f64() * 1000.0 / iters as f64
    );
}
