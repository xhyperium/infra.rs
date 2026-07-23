//! redisx **E2E CRUD**：用 `/home/workspace/data` 的 Binance 合约 K 线做写入/读出/更新/删除。
//!
//! 默认数据：
//!   `REDISX_DATA_ROOT`（默认 `/home/workspace/data`）
//!   `/binance_futures/merged/BTCUSDT/1m.csv`
//!
//! 无数据时 **soft-skip**（exit 0，不 panic）——CI live 无本地 data 卷时不得红。
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

/// 尝试加载 K 线；路径不存在或为空时返回 `None`（供 soft-skip）。
fn try_load_klines(path: &Path, limit: usize) -> Option<Vec<KlineLine>> {
    if !path.is_file() {
        eprintln!("e2e soft-skip: CSV 不存在 {}", path.display());
        return None;
    }
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("e2e soft-skip: 无法打开 {}: {e}", path.display());
            return None;
        }
    };
    let reader = BufReader::new(file);
    let mut out = Vec::with_capacity(limit);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("e2e soft-skip: 读行失败 {}: {e}", path.display());
                return None;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("open_time") || line.starts_with("Open") {
            continue;
        }
        let Some(open_time) = line.split(',').next().map(str::trim) else {
            continue;
        };
        if open_time.is_empty() || !open_time.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push(KlineLine { open_time: open_time.to_owned(), raw: line.as_bytes().to_vec() });
        if out.len() >= limit {
            break;
        }
    }
    if out.is_empty() {
        eprintln!("e2e soft-skip: 无有效行 {}", path.display());
        return None;
    }
    Some(out)
}

fn key_for(run: &str, open_time: &str) -> String {
    format!("redisx-e2e:{run}:kline:{open_time}")
}

#[tokio::test]
#[ignore = "requires live Redis; soft-skips if /home/workspace/data klines missing"]
async fn e2e_klines_single_crud() {
    let csv = default_csv();
    let Some(rows) = try_load_klines(&csv, 1) else {
        return;
    };
    let row = &rows[0];
    let client = match RedisClient::connect_from_env().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("e2e soft-skip: redis connect: {e}");
            return;
        }
    };
    let run = format!("{}", std::process::id());
    let key = key_for(&run, &row.open_time);

    client.set(&key, row.raw.clone(), Some(Duration::from_secs(300))).await.expect("create set");
    let got = client.get(&key).await.expect("read get");
    assert_eq!(got.as_deref(), Some(row.raw.as_slice()));
    let mut updated = row.raw.clone();
    updated.extend_from_slice(b"|upd");
    client.set(&key, updated.clone(), Some(Duration::from_secs(300))).await.expect("update");
    assert_eq!(client.get(&key).await.expect("get2").as_deref(), Some(updated.as_slice()));
    assert!(client.delete(&key).await.expect("del"));
    assert!(client.get(&key).await.expect("get3").is_none());
}

#[tokio::test]
#[ignore = "requires live Redis; soft-skips if /home/workspace/data klines missing"]
async fn e2e_klines_batch_mset_mget_pipeline_delete() {
    let csv = default_csv();
    let limit = row_limit();
    let Some(rows) = try_load_klines(&csv, limit) else {
        return;
    };
    let client = match RedisClient::connect_from_env().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("e2e soft-skip: redis connect: {e}");
            return;
        }
    };
    let run = format!("{}", std::process::id());
    let keys: Vec<String> = rows.iter().map(|r| key_for(&run, &r.open_time)).collect();

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

    let mut got2: Vec<Option<Vec<u8>>> = Vec::with_capacity(rows.len());
    for chunk in key_refs.chunks(CHUNK) {
        got2.extend(client.mget(chunk).await.expect("mget2"));
    }
    for (i, g) in got2.iter().enumerate() {
        assert_eq!(g.as_deref(), Some(updated[i].1.as_slice()));
    }

    let mut deleted = 0usize;
    for k in &keys {
        if client.delete(k).await.expect("del") {
            deleted += 1;
        }
    }
    assert_eq!(deleted, keys.len());

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
#[ignore = "requires live Redis; soft-skips if multi-symbol data missing"]
async fn e2e_klines_multi_symbol_namespace() {
    let root = data_root().join("binance_futures/merged");
    if !root.is_dir() {
        eprintln!("e2e soft-skip: data root missing {}", root.display());
        return;
    }
    let symbols = ["BTCUSDT/1m.csv", "ETHUSDT/1m.csv", "cm/BTCUSD_PERP/1m.csv"];
    let client = match RedisClient::connect_from_env().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("e2e soft-skip: redis connect: {e}");
            return;
        }
    };
    let run = format!("{}", std::process::id());
    let mut all_keys = Vec::new();

    for rel in symbols {
        let path = root.join(rel);
        let Some(rows) = try_load_klines(&path, 50) else {
            continue;
        };
        let ns = rel.replace('/', ":");
        for r in &rows {
            let key = format!("redisx-e2e:{run}:{ns}:{}", r.open_time);
            client.set(&key, r.raw.clone(), Some(Duration::from_secs(300))).await.expect("set");
            all_keys.push((key, r.raw.clone()));
        }
    }
    if all_keys.is_empty() {
        eprintln!("e2e soft-skip: no symbol files under {}", root.display());
        return;
    }

    for (k, v) in &all_keys {
        assert_eq!(client.get(k).await.expect("get").as_deref(), Some(v.as_slice()));
    }
    for (k, _) in &all_keys {
        let _ = client.delete(k).await;
    }
    eprintln!("e2e multi-symbol keys={}", all_keys.len());
}

/// 纯结构测：无数据路径必须 soft-skip 而非 panic（CI 无 data 卷）。
#[test]
fn e2e_missing_csv_is_soft_skip_not_panic() {
    let missing = PathBuf::from("/nonexistent/redisx-e2e-no-data/1m.csv");
    assert!(try_load_klines(&missing, 10).is_none());
}

/// 用真实 K 线数据驱动 Hash / Streams / MULTI 公共 API 面。
#[tokio::test]
#[ignore = "requires live Redis; soft-skips if /home/workspace/data klines missing"]
async fn e2e_klines_structures_streams_multi() {
    use redisx::TxCmd;

    let csv = default_csv();
    let Some(rows) = try_load_klines(&csv, 32) else {
        return;
    };
    let client = match RedisClient::connect_from_env().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("e2e soft-skip: redis connect: {e}");
            return;
        }
    };
    let run = format!("{}", std::process::id());
    let hash = format!("redisx-e2e:{run}:meta");
    let stream = format!("redisx-e2e:{run}:stream");
    let list = format!("redisx-e2e:{run}:list");

    // Hash：open_time → raw
    for r in rows.iter().take(16) {
        client.hset(&hash, &r.open_time, r.raw.clone()).await.expect("hset");
    }
    let first = &rows[0];
    assert_eq!(
        client.hget(&hash, &first.open_time).await.expect("hget").as_deref(),
        Some(first.raw.as_slice())
    );

    // Streams：每根 K 线一条（自动 id）+ 一条显式 id（xadd_with_id 公共面）
    let mut last_id = "0-0".to_owned();
    for r in rows.iter().take(16) {
        let id = client
            .xadd(&stream, &[("ot", r.open_time.as_bytes()), ("row", r.raw.as_slice())])
            .await
            .expect("xadd");
        last_id = id;
    }
    // 基于最后自动 id 构造更大显式 id，保证单调
    let ms: u128 = last_id
        .split('-')
        .next()
        .and_then(|s| s.parse::<u128>().ok())
        .unwrap_or(1u128)
        .saturating_add(1);
    let fixed = format!("{ms}-0");
    let got = client
        .xadd_with_id(
            &stream,
            &fixed,
            &[("ot", rows[0].open_time.as_bytes()), ("row", rows[0].raw.as_slice())],
        )
        .await
        .expect("xadd_with_id");
    assert_eq!(got, fixed);
    last_id = got;
    assert!(client.xlen(&stream).await.expect("xlen") >= 17);
    let entries = client.xrange(&stream, "-", "+", Some(20)).await.expect("xrange");
    assert!(entries.len() >= 16);
    let more = client.xread(&stream, "0-0", Some(20)).await.expect("xread");
    assert!(!more.is_empty());
    assert!(!last_id.is_empty());

    // List + MULTI：批量标记
    for r in rows.iter().take(8) {
        client.lpush(&list, r.open_time.as_bytes().to_vec()).await.expect("lpush");
    }
    let cmds: Vec<TxCmd> = rows
        .iter()
        .take(4)
        .map(|r| TxCmd::Set {
            key: format!("redisx-e2e:{run}:tx:{}", r.open_time),
            value: r.raw.clone(),
        })
        .collect();
    let vals = client.multi_exec(&cmds).await.expect("multi_exec");
    assert_eq!(vals.len(), 4);

    // cleanup
    let _ = client.delete(&hash).await;
    let _ = client.delete(&stream).await;
    let _ = client.delete(&list).await;
    for r in rows.iter().take(4) {
        let _ = client.delete(&format!("redisx-e2e:{run}:tx:{}", r.open_time)).await;
    }
    eprintln!(
        "e2e_structures_streams_multi ok rows_used={} csv={}",
        rows.len().min(32),
        csv.display()
    );
}
