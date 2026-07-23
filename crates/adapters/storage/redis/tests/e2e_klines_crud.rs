//! redisx **E2E CRUD**：用 `/home/workspace/data` 的 Binance 合约 K 线做写入/读出/更新/删除。
//!
//! 默认数据：
//!   `REDISX_DATA_ROOT`（默认 `/home/workspace/data`）
//!   `/binance_futures/merged/BTCUSDT/1m.csv`
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   env REDISX_E2E_ROWS=200 \
//!   cargo test -p redisx --test e2e_klines_crud -- --ignored --nocapture
//! ```

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use redisx::RedisClient;

/// 单行 K 线（保留原始 CSV 字节作为 value，便于 round-trip 比对）。
#[derive(Debug, Clone)]
struct KlineLine {
    open_time: String,
    raw: Vec<u8>,
}

fn data_root() -> PathBuf {
    PathBuf::from(
        std::env::var("REDISX_DATA_ROOT").unwrap_or_else(|_| "/home/workspace/data".into()),
    )
}

fn default_csv() -> PathBuf {
    if let Ok(p) = std::env::var("REDISX_E2E_CSV") {
        return PathBuf::from(p);
    }
    data_root().join("binance_futures/merged/BTCUSDT/1m.csv")
}

fn row_limit() -> usize {
    std::env::var("REDISX_E2E_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
        .clamp(1, 5_000)
}

fn load_klines(path: &Path, limit: usize) -> Vec<KlineLine> {
    let file = File::open(path).unwrap_or_else(|e| panic!("open {}: {e}", path.display()));
    let reader = BufReader::new(file);
    let mut out = Vec::with_capacity(limit);
    for line in reader.lines() {
        let line = line.expect("read line");
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 跳过可能的表头
        if line.starts_with("open_time") || line.starts_with("Open") {
            continue;
        }
        let open_time = line.split(',').next().expect("open_time field").trim().to_owned();
        if open_time.is_empty() || !open_time.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push(KlineLine { open_time, raw: line.as_bytes().to_vec() });
        if out.len() >= limit {
            break;
        }
    }
    assert!(
        !out.is_empty(),
        "no klines loaded from {} (check REDISX_DATA_ROOT / REDISX_E2E_CSV)",
        path.display()
    );
    out
}

fn key_for(run: &str, open_time: &str) -> String {
    format!("redisx-e2e:{run}:kline:{open_time}")
}

#[tokio::test]
#[ignore = "requires live Redis + /home/workspace/data klines"]
async fn e2e_klines_single_crud() {
    let csv = default_csv();
    let rows = load_klines(&csv, 1);
    let row = &rows[0];
    let client = RedisClient::connect_from_env().await.expect("connect");
    let run = format!("{}", std::process::id());
    let key = key_for(&run, &row.open_time);

    // Create
    client.set(&key, row.raw.clone(), Some(Duration::from_secs(300))).await.expect("create set");
    // Read
    let got = client.get(&key).await.expect("read get");
    assert_eq!(got.as_deref(), Some(row.raw.as_slice()));
    // Update
    let mut updated = row.raw.clone();
    updated.extend_from_slice(b"|upd");
    client.set(&key, updated.clone(), Some(Duration::from_secs(300))).await.expect("update");
    assert_eq!(client.get(&key).await.expect("get2").as_deref(), Some(updated.as_slice()));
    // Delete
    assert!(client.delete(&key).await.expect("del"));
    assert!(client.get(&key).await.expect("get3").is_none());
}

#[tokio::test]
#[ignore = "requires live Redis + /home/workspace/data klines"]
async fn e2e_klines_batch_mset_mget_pipeline_delete() {
    let csv = default_csv();
    let limit = row_limit();
    let rows = load_klines(&csv, limit);
    let client = RedisClient::connect_from_env().await.expect("connect");
    let run = format!("{}", std::process::id());
    let keys: Vec<String> = rows.iter().map(|r| key_for(&run, &r.open_time)).collect();

    // Batch create：分块 pipeline（避免单事务过大）
    const CHUNK: usize = 32;
    for chunk in keys.chunks(CHUNK).zip(rows.chunks(CHUNK)) {
        let (ks, rs) = chunk;
        let items: Vec<(&str, Vec<u8>)> =
            ks.iter().zip(rs.iter()).map(|(k, r)| (k.as_str(), r.raw.clone())).collect();
        client
            .pipeline_set(&items, Some(Duration::from_secs(300)))
            .await
            .expect("pipeline_set batch create");
    }

    // 抽样确认首尾已写入（pipeline 失败时尽早暴露）
    assert_eq!(
        client.get(&keys[0]).await.expect("probe0").as_deref(),
        Some(rows[0].raw.as_slice()),
        "first key missing after pipeline"
    );
    assert_eq!(
        client.get(keys.last().unwrap()).await.expect("probeN").as_deref(),
        Some(rows.last().unwrap().raw.as_slice()),
        "last key missing after pipeline"
    );

    // Batch read mget（同样分块，兼容服务端参数上限）
    let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
    let mut got: Vec<Option<Vec<u8>>> = Vec::with_capacity(rows.len());
    for chunk in key_refs.chunks(CHUNK) {
        got.extend(client.mget(chunk).await.expect("mget chunk"));
    }
    assert_eq!(got.len(), rows.len());
    for (i, g) in got.iter().enumerate() {
        assert_eq!(
            g.as_deref(),
            Some(rows[i].raw.as_slice()),
            "mismatch at index {i} open_time={}",
            rows[i].open_time
        );
    }

    // Update all with mset (append marker)
    let updated: Vec<(String, Vec<u8>)> = keys
        .iter()
        .zip(rows.iter())
        .map(|(k, r)| {
            let mut v = r.raw.clone();
            v.extend_from_slice(b"|m");
            (k.clone(), v)
        })
        .collect();
    let mset_items: Vec<(&str, &[u8])> =
        updated.iter().map(|(k, v)| (k.as_str(), v.as_slice())).collect();
    client.mset(&mset_items).await.expect("mset update");

    let got2 = client.mget(&key_refs).await.expect("mget2");
    for (i, g) in got2.iter().enumerate() {
        assert_eq!(g.as_deref(), Some(updated[i].1.as_slice()));
    }

    // Delete all
    let mut deleted = 0usize;
    for k in &keys {
        if client.delete(k).await.expect("del") {
            deleted += 1;
        }
    }
    assert_eq!(deleted, keys.len());

    // Confirm gone (sample first 10)
    for k in keys.iter().take(10) {
        assert!(client.get(k).await.expect("get gone").is_none());
    }

    eprintln!(
        "e2e_klines_batch ok rows={} csv={} data_root={}",
        rows.len(),
        csv.display(),
        data_root().display()
    );
}

#[tokio::test]
#[ignore = "requires live Redis + multi-symbol data"]
async fn e2e_klines_multi_symbol_namespace() {
    let root = data_root().join("binance_futures/merged");
    let symbols = ["BTCUSDT/1m.csv", "ETHUSDT/1m.csv", "cm/BTCUSD_PERP/1m.csv"];
    let client = RedisClient::connect_from_env().await.expect("connect");
    let run = format!("{}", std::process::id());
    let mut all_keys = Vec::new();

    for rel in symbols {
        let path = root.join(rel);
        if !path.is_file() {
            eprintln!("skip missing {}", path.display());
            continue;
        }
        let rows = load_klines(&path, 50);
        let ns = rel.replace('/', ":");
        for r in &rows {
            let key = format!("redisx-e2e:{run}:{ns}:{}", r.open_time);
            client.set(&key, r.raw.clone(), Some(Duration::from_secs(300))).await.expect("set");
            all_keys.push((key, r.raw.clone()));
        }
    }
    assert!(!all_keys.is_empty(), "no symbol files under {}", root.display());

    for (k, v) in &all_keys {
        assert_eq!(client.get(k).await.expect("get").as_deref(), Some(v.as_slice()));
    }
    for (k, _) in &all_keys {
        let _ = client.delete(k).await;
    }
    eprintln!("e2e multi-symbol keys={}", all_keys.len());
}
