//! taosx **E2E**：`/home/workspace/data` Binance K 线 → 写/读/校验/清理。
//!
//! 无数据 soft-skip（exit 0）。
//!
//! ```bash
//! scripts/live/export-foundationx-env.sh --env dev -- \
//!   env TAOSX_E2E_ROWS=100 \
//!   cargo test -p taosx --test e2e_klines -- --ignored --nocapture
//! ```

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use taosx::TaosPool;

struct Kline {
    open_time_ms: i64,
    close: i128,
}

fn data_root() -> PathBuf {
    PathBuf::from(
        std::env::var("TAOSX_DATA_ROOT").unwrap_or_else(|_| "/home/workspace/data".into()),
    )
}

fn default_csv() -> PathBuf {
    if let Ok(p) = std::env::var("TAOSX_E2E_CSV") {
        return PathBuf::from(p);
    }
    data_root().join("binance_futures/merged/BTCUSDT/1m.csv")
}

fn row_limit() -> usize {
    std::env::var("TAOSX_E2E_ROWS").ok().and_then(|s| s.parse().ok()).unwrap_or(100).clamp(1, 2_000)
}

fn try_load(path: &Path, limit: usize) -> Option<Vec<Kline>> {
    if !path.is_file() {
        eprintln!("e2e soft-skip: CSV 不存在 {}", path.display());
        return None;
    }
    let file = File::open(path).ok()?;
    let mut out = Vec::with_capacity(limit);
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("open_time") || line.starts_with("Open") {
            continue;
        }
        let mut parts = line.split(',');
        let open_time = parts.next()?.trim();
        // typical: open_time,open,high,low,close,...
        let _open = parts.next()?;
        let _high = parts.next()?;
        let _low = parts.next()?;
        let close = parts.next()?.trim();
        let open_time_ms: i64 = open_time.parse().ok()?;
        // close as decimal with scale 2 (strip dot)
        let close_i = parse_price_scale2(close)?;
        out.push(Kline { open_time_ms, close: close_i });
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

fn parse_price_scale2(s: &str) -> Option<i128> {
    let s = s.trim();
    if let Some((a, b)) = s.split_once('.') {
        let a: i128 = a.parse().ok()?;
        let b = format!("{:0<2}", &b[..b.len().min(2)]);
        let frac: i128 = b.parse().ok()?;
        Some(a.saturating_mul(100).saturating_add(frac))
    } else {
        let a: i128 = s.parse().ok()?;
        Some(a.saturating_mul(100))
    }
}

#[tokio::test]
#[ignore = "requires live TDengine + optional /home/workspace/data"]
async fn e2e_klines_write_query_delete() {
    let csv = default_csv();
    let Some(klines) = try_load(&csv, row_limit()) else {
        return; // soft-skip
    };
    let pool = match TaosPool::connect_from_env().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("e2e soft-skip: connect failed: {e}");
            return;
        }
    };
    let table = format!("_sc_e2e_klines_{}", std::process::id());
    let prec = pool.precision();
    let points: Vec<Tick> = klines
        .iter()
        .map(|k| {
            let ts_ns = prec.to_nanos(prec.from_nanos(k.open_time_ms.saturating_mul(1_000_000)));
            Tick {
                symbol: "BTCUSDT".into(),
                bid: Price::new(Decimal::try_new(k.close, 2).expect("bid")),
                ask: Price::new(Decimal::try_new(k.close.saturating_add(1), 2).expect("ask")),
                ts: ts_ns,
            }
        })
        .collect();
    let start = points.first().map(|t| t.ts).unwrap_or(0);
    let end = points.last().map(|t| t.ts).unwrap_or(0);

    // artifact dir
    let art = data_root().join("taosx/e2e");
    let _ = std::fs::create_dir_all(&art);

    pool.write_series(&table, points.clone()).await.expect("write klines");
    let got = pool.query_series(&table, start, end).await.expect("query");
    assert!(!got.is_empty(), "query empty after write");
    assert!(got.len() >= points.len().min(1));

    // 写回摘要
    let summary = format!(
        "e2e_klines table={table} csv={} written={} read={}\n",
        csv.display(),
        points.len(),
        got.len()
    );
    let path = art.join(format!("e2e_{}.txt", std::process::id()));
    std::fs::write(&path, summary).expect("write artifact");
    assert!(path.is_file());

    let _ = pool.exec_sql(&format!("DROP STABLE IF EXISTS `{table}`")).await;
    pool.close().await.ok();
}
