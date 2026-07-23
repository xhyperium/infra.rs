//! 有界异步写批处理器：`push` → 按行/字节/时间刷写 → 显式 `flush` / `close`。
//!
//! drop **不**保证远端写入；关闭须 `close().await`。

use std::sync::Arc;
use std::time::{Duration, Instant};

use canonical::Tick;
use kernel::{XError, XResult};
use tokio::sync::Mutex;

use crate::client::{BatchWriteReport, TaosPool};

/// 批处理器配置。
#[derive(Debug, Clone)]
pub struct WriteBatcherConfig {
    pub max_rows: usize,
    pub max_bytes_hint: usize,
    pub flush_interval: Duration,
}

impl Default for WriteBatcherConfig {
    fn default() -> Self {
        Self {
            max_rows: 500,
            max_bytes_hint: 256 * 1024,
            flush_interval: Duration::from_millis(200),
        }
    }
}

struct Inner {
    table: String,
    buf: Vec<Tick>,
    closed: bool,
    last_flush: Instant,
    cfg: WriteBatcherConfig,
    total_accepted: usize,
    total_failed: usize,
}

/// 异步写批处理器。
pub struct WriteBatcher {
    pool: TaosPool,
    inner: Arc<Mutex<Inner>>,
}

impl WriteBatcher {
    /// 绑定池与目标超级表。
    #[must_use]
    pub fn new(pool: TaosPool, table: impl Into<String>, cfg: WriteBatcherConfig) -> Self {
        Self {
            pool,
            inner: Arc::new(Mutex::new(Inner {
                table: table.into(),
                buf: Vec::with_capacity(cfg.max_rows.min(1024)),
                closed: false,
                last_flush: Instant::now(),
                cfg,
                total_accepted: 0,
                total_failed: 0,
            })),
        }
    }

    /// 推入点；达阈值时自动 flush。
    pub async fn push(&self, tick: Tick) -> XResult<()> {
        let mut g = self.inner.lock().await;
        if g.closed {
            return Err(XError::unavailable("WriteBatcher 已关闭"));
        }
        g.buf.push(tick);
        let should =
            g.buf.len() >= g.cfg.max_rows || g.last_flush.elapsed() >= g.cfg.flush_interval;
        if should {
            let table = g.table.clone();
            let batch = std::mem::take(&mut g.buf);
            drop(g);
            self.flush_batch(&table, batch).await?;
            let mut g = self.inner.lock().await;
            g.last_flush = Instant::now();
        }
        Ok(())
    }

    /// 刷空缓冲。
    pub async fn flush(&self) -> XResult<BatchWriteReport> {
        let mut g = self.inner.lock().await;
        if g.closed {
            return Err(XError::unavailable("WriteBatcher 已关闭"));
        }
        let table = g.table.clone();
        let batch = std::mem::take(&mut g.buf);
        g.last_flush = Instant::now();
        drop(g);
        self.flush_batch(&table, batch).await
    }

    /// 刷写并关闭；之后 `push`/`flush` 失败。
    pub async fn close(&self) -> XResult<BatchWriteReport> {
        let report = self.flush().await?;
        let mut g = self.inner.lock().await;
        g.closed = true;
        Ok(report)
    }

    /// 累计 accepted/failed。
    pub async fn totals(&self) -> (usize, usize) {
        let g = self.inner.lock().await;
        (g.total_accepted, g.total_failed)
    }

    async fn flush_batch(&self, table: &str, batch: Vec<Tick>) -> XResult<BatchWriteReport> {
        if batch.is_empty() {
            return Ok(BatchWriteReport::default());
        }
        let report = self.pool.write_batch_report(table, &batch).await?;
        let mut g = self.inner.lock().await;
        g.total_accepted = g.total_accepted.saturating_add(report.accepted);
        g.total_failed = g.total_failed.saturating_add(report.failed);
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TaosConfig;
    use decimalx::{Decimal, Price};

    fn tick(ts: i64) -> Tick {
        Tick {
            symbol: "T".into(),
            bid: Price::new(Decimal::try_new(1, 2).expect("d")),
            ask: Price::new(Decimal::try_new(2, 2).expect("d")),
            ts,
        }
    }

    #[tokio::test]
    async fn closed_rejects_push() {
        let pool = TaosPool::connect_without_ping(TaosConfig::default()).expect("pool");
        let b = WriteBatcher::new(
            pool,
            "sc_batcher",
            WriteBatcherConfig { max_rows: 100, ..Default::default() },
        );
        let _ = b.close().await; // 无服务时可能 err；仍标记 closed
        let mut g = b.inner.lock().await;
        g.closed = true;
        drop(g);
        let err = b.push(tick(1)).await.expect_err("closed");
        assert_eq!(err.kind(), kernel::ErrorKind::Unavailable);
    }
}
