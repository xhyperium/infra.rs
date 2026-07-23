//! 进程内有界操作计数（非 OTLP/Prometheus 导出）。
//!
//! 标签仅 operation/outcome 语义，无高基数 symbol/table。
//! 完整 RED 指标面与远程导出仍为 NO-GO；本模块提供可测的本地快照。

use std::sync::atomic::{AtomicU64, Ordering};

/// 池/传输操作计数快照。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TaosMetricsSnapshot {
    /// REST SQL 成功次数。
    pub sql_ok: u64,
    /// REST SQL 失败次数。
    pub sql_err: u64,
    /// 批量写入整批成功次数。
    pub write_ok: u64,
    /// 批量写入失败次数（含部分成功映射为错误）。
    pub write_err: u64,
    /// `query_series` 成功次数。
    pub query_ok: u64,
    /// `query_series` 失败次数。
    pub query_err: u64,
    /// `ping` 成功次数。
    pub ping_ok: u64,
    /// `ping` 失败次数。
    pub ping_err: u64,
    /// `health` 判定就绪次数。
    pub health_ready: u64,
    /// `health` 判定未就绪次数。
    pub health_not_ready: u64,
    /// Native WS 握手探测成功（进程级累计）。
    pub ws_probe_ok: u64,
    /// Native WS 握手探测失败（进程级累计）。
    pub ws_probe_err: u64,
}

impl TaosMetricsSnapshot {
    /// 全部计数之和（粗粒度负载指示）。
    #[must_use]
    pub fn total_events(&self) -> u64 {
        self.sql_ok
            + self.sql_err
            + self.write_ok
            + self.write_err
            + self.query_ok
            + self.query_err
            + self.ping_ok
            + self.ping_err
            + self.health_ready
            + self.health_not_ready
            + self.ws_probe_ok
            + self.ws_probe_err
    }
}

/// 池级计数器（随 `TaosPool` 生命周期）。
pub(crate) struct OpCounters {
    sql_ok: AtomicU64,
    sql_err: AtomicU64,
    write_ok: AtomicU64,
    write_err: AtomicU64,
    query_ok: AtomicU64,
    query_err: AtomicU64,
    ping_ok: AtomicU64,
    ping_err: AtomicU64,
    health_ready: AtomicU64,
    health_not_ready: AtomicU64,
}

impl OpCounters {
    pub(crate) fn new() -> Self {
        Self {
            sql_ok: AtomicU64::new(0),
            sql_err: AtomicU64::new(0),
            write_ok: AtomicU64::new(0),
            write_err: AtomicU64::new(0),
            query_ok: AtomicU64::new(0),
            query_err: AtomicU64::new(0),
            ping_ok: AtomicU64::new(0),
            ping_err: AtomicU64::new(0),
            health_ready: AtomicU64::new(0),
            health_not_ready: AtomicU64::new(0),
        }
    }

    pub(crate) fn snapshot(&self) -> TaosMetricsSnapshot {
        let (ws_ok, ws_err) = ws_probe_totals();
        TaosMetricsSnapshot {
            sql_ok: self.sql_ok.load(Ordering::Relaxed),
            sql_err: self.sql_err.load(Ordering::Relaxed),
            write_ok: self.write_ok.load(Ordering::Relaxed),
            write_err: self.write_err.load(Ordering::Relaxed),
            query_ok: self.query_ok.load(Ordering::Relaxed),
            query_err: self.query_err.load(Ordering::Relaxed),
            ping_ok: self.ping_ok.load(Ordering::Relaxed),
            ping_err: self.ping_err.load(Ordering::Relaxed),
            health_ready: self.health_ready.load(Ordering::Relaxed),
            health_not_ready: self.health_not_ready.load(Ordering::Relaxed),
            ws_probe_ok: ws_ok,
            ws_probe_err: ws_err,
        }
    }

    pub(crate) fn inc_sql_ok(&self) {
        self.sql_ok.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_sql_err(&self) {
        self.sql_err.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_write_ok(&self) {
        self.write_ok.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_write_err(&self) {
        self.write_err.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_query_ok(&self) {
        self.query_ok.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_query_err(&self) {
        self.query_err.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_ping_ok(&self) {
        self.ping_ok.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_ping_err(&self) {
        self.ping_err.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_health_ready(&self) {
        self.health_ready.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn inc_health_not_ready(&self) {
        self.health_not_ready.fetch_add(1, Ordering::Relaxed);
    }
}

static WS_PROBE_OK: AtomicU64 = AtomicU64::new(0);
static WS_PROBE_ERR: AtomicU64 = AtomicU64::new(0);

/// 进程级 WS 探测计数（`connect_native_ws` 自由函数路径）。
#[must_use]
pub fn ws_probe_totals() -> (u64, u64) {
    (WS_PROBE_OK.load(Ordering::Relaxed), WS_PROBE_ERR.load(Ordering::Relaxed))
}

pub(crate) fn record_ws_probe(ok: bool) {
    if ok {
        WS_PROBE_OK.fetch_add(1, Ordering::Relaxed);
    } else {
        WS_PROBE_ERR.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate() {
        let c = OpCounters::new();
        c.inc_sql_ok();
        c.inc_sql_err();
        c.inc_write_ok();
        let snap = c.snapshot();
        assert_eq!(snap.sql_ok, 1);
        assert_eq!(snap.sql_err, 1);
        assert_eq!(snap.write_ok, 1);
        assert!(snap.total_events() >= 3);
    }
}
