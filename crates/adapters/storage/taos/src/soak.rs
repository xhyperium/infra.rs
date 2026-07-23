//! 有界 soak 框架：固定时长内循环 ping/写读，产物写入 data 目录。
//!
//! 默认短时长（测试/CI）；`TAOSX_SOAK_SECS` 可升至 86400 做 24h 墙钟（进程外调度）。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use kernel::XResult;
use serde::Serialize;

use crate::client::TaosPool;

/// soak 运行参数。
#[derive(Debug, Clone)]
pub struct SoakConfig {
    pub duration: Duration,
    pub interval: Duration,
    pub table: String,
    pub artifact_dir: PathBuf,
}

impl Default for SoakConfig {
    fn default() -> Self {
        let secs: u64 =
            std::env::var("TAOSX_SOAK_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(3);
        Self {
            duration: Duration::from_secs(secs.clamp(1, 86_400)),
            interval: Duration::from_millis(200),
            table: format!("_sc_soak_{}", std::process::id()),
            artifact_dir: PathBuf::from(
                std::env::var("TAOSX_DATA_ROOT")
                    .unwrap_or_else(|_| "/home/workspace/data/taosx".into()),
            )
            .join("soak"),
        }
    }
}

/// soak 结果摘要。
#[derive(Debug, Clone, Serialize)]
pub struct SoakReport {
    pub duration_ms: u128,
    pub iterations: u64,
    pub ping_ok: u64,
    pub write_ok: u64,
    pub query_ok: u64,
    pub errors: u64,
    pub artifact: String,
}

/// 运行有界 soak，写 JSON 产物。
pub async fn run_soak(pool: &TaosPool, cfg: SoakConfig) -> XResult<SoakReport> {
    std::fs::create_dir_all(&cfg.artifact_dir)
        .map_err(|e| kernel::XError::internal(format!("创建 soak 目录失败: {e}")))?;
    let start = Instant::now();
    let mut iterations = 0u64;
    let mut ping_ok = 0u64;
    let mut write_ok = 0u64;
    let mut query_ok = 0u64;
    let mut errors = 0u64;
    let symbol = "SOAK";

    while start.elapsed() < cfg.duration {
        iterations += 1;
        match pool.ping().await {
            Ok(()) => ping_ok += 1,
            Err(_) => errors += 1,
        }
        let ts =
            SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as i64).unwrap_or(0);
        let prec = pool.precision();
        let ts = prec.to_nanos(prec.from_nanos(ts));
        let tick = Tick {
            symbol: symbol.into(),
            bid: Price::new(decimal_scale2(100, iterations)),
            ask: Price::new(decimal_scale2(101, iterations)),
            ts,
        };
        match pool.write_series(&cfg.table, vec![tick]).await {
            Ok(()) => write_ok += 1,
            Err(_) => errors += 1,
        }
        match pool.query_series(&cfg.table, ts.saturating_sub(1_000_000_000), ts).await {
            Ok(_) => query_ok += 1,
            Err(_) => errors += 1,
        }
        tokio::time::sleep(cfg.interval).await;
    }

    let report = SoakReport {
        duration_ms: start.elapsed().as_millis(),
        iterations,
        ping_ok,
        write_ok,
        query_ok,
        errors,
        artifact: String::new(),
    };
    let path = write_report(&cfg.artifact_dir, &report)?;
    Ok(SoakReport { artifact: path.display().to_string(), ..report })
}

fn decimal_scale2(base: i128, iterations: u64) -> Decimal {
    let offset = (iterations % 50) as i128;
    let mantissa = base.saturating_add(offset);
    Decimal::try_new(mantissa, 2).unwrap_or_else(|_| Decimal::try_new(0, 2).expect("zero"))
}

fn write_report(dir: &Path, report: &SoakReport) -> XResult<PathBuf> {
    let name = format!(
        "soak_{}_{}.json",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
    );
    let path = dir.join(name);
    let body = serde_json::to_vec_pretty(report)
        .map_err(|e| kernel::XError::internal(format!("soak json: {e}")))?;
    std::fs::write(&path, body)
        .map_err(|e| kernel::XError::internal(format!("写 soak 产物失败: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_duration_clamped() {
        let c = SoakConfig::default();
        assert!(c.duration >= Duration::from_secs(1));
        assert!(c.duration <= Duration::from_secs(86_400));
    }
}
