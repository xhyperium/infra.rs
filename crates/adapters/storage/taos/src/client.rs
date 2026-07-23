//! TDengine REST 生产客户端（默认 6041）+ 批量写入 + 池背压。

use std::fmt::Write as _;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use canonical::Tick;
use chrono::{DateTime, Utc};
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use kernel::{XError, XResult};
use serde::Deserialize;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::debug;

use crate::config::{
    HARD_MAX_BATCH_BYTES, HARD_MAX_BATCH_ROWS, TaosConfig, TransportMode, TsPrecision,
};
use crate::metrics::{OpCounters, TaosMetricsSnapshot};
use crate::native;

const CLOSED_BIT: usize = 1usize << (usize::BITS - 1);
const IN_FLIGHT_MASK: usize = !CLOSED_BIT;
const INSERT_PREFIX: &str = "INSERT INTO ";
const MAX_STABLE_NAME_BYTES: usize = 94;
const MAX_SYMBOL_BYTES: usize = 48;

/// REST 查询结果（精简）。
#[derive(Debug, Clone)]
pub struct TaosExecResult {
    /// 驱动 code（0 = 成功）。
    pub code: i32,
    /// 行数据（字符串化单元格）。
    pub rows: Vec<Vec<String>>,
    /// 列名（若有）。
    pub columns: Vec<String>,
    /// 受影响行数（写路径可能有）。
    pub affected_rows: Option<i64>,
}

/// 池运行时快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaosPoolStats {
    /// 正在执行的请求数。
    pub in_flight: usize,
    /// 是否已关闭。
    pub closed: bool,
}

/// 批量写入结果报告（行数与 chunk 计数）。
///
/// - 全部成功：`failed == 0` 且 `accepted == 请求行数`。
/// - 中途失败：通过 [`BatchWritePartialError`] 带回**已成功提交**的 `accepted`，
///   与仍未提交的 `failed`；**不**自动重试（幂等重试仍为 NO-GO）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BatchWriteReport {
    /// 已成功提交的行数。
    pub accepted: usize,
    /// 未提交行数（含失败 chunk 及其后未尝试行）。
    pub failed: usize,
    /// 成功 chunk 数。
    pub chunks_ok: usize,
    /// 计划 chunk 总数。
    pub chunks_total: usize,
}

impl BatchWriteReport {
    /// 是否整批完成且无失败行。
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.failed == 0 && self.chunks_ok == self.chunks_total
    }
}

/// 批量写入部分成功错误：报告与根因一并返回。
#[derive(Debug)]
pub struct BatchWritePartialError {
    /// 失败瞬间的可定位报告。
    pub report: BatchWriteReport,
    /// 驱动/传输错误。
    pub source: XError,
}

impl std::fmt::Display for BatchWritePartialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "write_batch 部分成功 accepted={} failed={} chunks_ok={}/{}: {}",
            self.report.accepted,
            self.report.failed,
            self.report.chunks_ok,
            self.report.chunks_total,
            self.source
        )
    }
}

impl std::error::Error for BatchWritePartialError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<BatchWritePartialError> for XError {
    fn from(value: BatchWritePartialError) -> Self {
        let kind = value.source.kind();
        let message = value.to_string();
        match kind {
            kernel::ErrorKind::Invalid => XError::invalid(message),
            kernel::ErrorKind::Missing => XError::missing(message),
            kernel::ErrorKind::Conflict => XError::conflict(message),
            kernel::ErrorKind::Transient => XError::transient(message),
            kernel::ErrorKind::Unavailable => XError::unavailable(message),
            kernel::ErrorKind::Cancelled => XError::cancelled(message),
            kernel::ErrorKind::DeadlineExceeded => XError::deadline_exceeded(message),
            kernel::ErrorKind::Invariant => XError::invariant(message),
            _ => XError::internal(message),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawResponse {
    code: i32,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    column_meta: Vec<serde_json::Value>,
    #[serde(default)]
    data: Vec<Vec<serde_json::Value>>,
    #[serde(default)]
    rows: Option<i64>,
}

/// 共享 REST 客户端 + 配置。
#[derive(Clone)]
pub struct TaosPool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    http: reqwest::Client,
    config: TaosConfig,
    precision: RwLock<TsPrecision>,
    sem: Arc<Semaphore>,
    state: AtomicUsize,
    drained: Notify,
    metrics: OpCounters,
}

struct RequestGuard {
    _permit: OwnedSemaphorePermit,
    inner: Arc<PoolInner>,
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        let previous = self.inner.state.fetch_sub(1, Ordering::AcqRel);
        if previous & IN_FLIGHT_MASK == 1 {
            self.inner.drained.notify_waiters();
        }
    }
}

/// 工作句柄别名。
pub type TaosClient = TaosPool;

/// 构建分块 INSERT SQL（纯函数；单测驱动 chunk 尺寸）。
///
/// 每个 chunk 生成一条 `INSERT INTO ...` 多子表语句。
pub fn build_insert_sql_chunks(
    table: &str,
    points: &[Tick],
    prec: TsPrecision,
    max_rows: usize,
) -> XResult<Vec<String>> {
    Ok(build_insert_sql_chunks_with_limits(table, points, prec, max_rows, HARD_MAX_BATCH_BYTES)?
        .into_iter()
        .map(|(sql, _)| sql)
        .collect())
}

/// 单个 SQL chunk 及其行数。
type SqlChunk = (String, usize);

fn build_insert_sql_chunks_with_limits(
    table: &str,
    points: &[Tick],
    prec: TsPrecision,
    max_rows: usize,
    max_bytes: usize,
) -> XResult<Vec<SqlChunk>> {
    validate_stable_ident(table)?;
    if max_rows == 0 || max_rows > HARD_MAX_BATCH_ROWS {
        return Err(XError::invalid(format!("max_rows 必须为 1..={HARD_MAX_BATCH_ROWS}")));
    }
    if max_bytes < INSERT_PREFIX.len() || max_bytes > HARD_MAX_BATCH_BYTES {
        return Err(XError::invalid(format!(
            "max_bytes 必须为 {}..={HARD_MAX_BATCH_BYTES}",
            INSERT_PREFIX.len()
        )));
    }
    if points.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut sql = String::from(INSERT_PREFIX);
    let mut rows = 0usize;
    for tick in points {
        let sub = subtable_name(table, &tick.symbol)?;
        let sym = escape_str(&tick.symbol);
        let ts = prec.from_nanos(tick.ts);
        let bid = tick.bid.as_decimal().to_string();
        let ask = tick.ask.as_decimal().to_string();
        let row = format!("`{sub}` USING `{table}` TAGS ('{sym}') VALUES ({ts},'{bid}','{ask}')");
        let separator = usize::from(rows > 0);
        let next_len = sql
            .len()
            .checked_add(separator)
            .and_then(|length| length.checked_add(row.len()))
            .ok_or_else(|| XError::invalid("批量 SQL 字节数溢出"))?;
        if rows > 0 && (rows >= max_rows || next_len > max_bytes) {
            out.push((sql, rows));
            sql = String::from(INSERT_PREFIX);
            rows = 0;
        }
        let row_len = INSERT_PREFIX
            .len()
            .checked_add(row.len())
            .ok_or_else(|| XError::invalid("单行 SQL 字节数溢出"))?;
        if row_len > max_bytes {
            return Err(XError::invalid(format!("单行 SQL 超过 batch_max_bytes={max_bytes}")));
        }
        if rows > 0 {
            sql.push(' ');
        }
        sql.push_str(&row);
        rows += 1;
    }
    if rows > 0 {
        out.push((sql, rows));
    }
    Ok(out)
}

impl TaosPool {
    /// 连接：构建 HTTP 客户端、可选建库、探测精度、ping。
    ///
    /// `TransportMode::NativeWs` 时先做一次原生 WS 握手探测（失败即返回）。
    pub async fn connect(config: TaosConfig) -> XResult<Self> {
        config.validate()?;

        if config.transport == TransportMode::NativeWs {
            native::connect_native_ws(&config).await?;
        }

        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(8)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| XError::internal(format!("taos http client: {e}")))?;

        let initial_precision = config.precision.unwrap_or(TsPrecision::Ms);
        let max_in_flight = config.max_in_flight;
        let pool = Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                precision: RwLock::new(initial_precision),
                sem: Arc::new(Semaphore::new(max_in_flight)),
                state: AtomicUsize::new(0),
                drained: Notify::new(),
                metrics: OpCounters::new(),
            }),
        };

        // REST 路径：确保 database + 精度探测 + ping
        // NativeWs 探测已完成；仍用 REST 做 SQL（本阶段 WS 仅作连通性 lane）
        if !pool.inner.config.database.is_empty() {
            let db = pool.inner.config.database.clone();
            validate_ident(&db)?;
            pool.exec_sql_raw(&format!("CREATE DATABASE IF NOT EXISTS `{db}` KEEP 3650"), false)
                .await?;
            let detected = pool.detect_precision().await?;
            if let Some(configured) = pool.inner.config.precision {
                if configured != detected {
                    return Err(XError::invalid(format!(
                        "taos 配置精度 {configured:?} 与数据库精度 {detected:?} 不一致"
                    )));
                }
            }
            *pool
                .inner
                .precision
                .write()
                .map_err(|_| XError::invariant("taos 精度状态锁已中毒"))? = detected;
        }

        pool.ping().await?;
        Ok(pool)
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(TaosConfig::from_env()).await
    }

    /// 离线/单测构造：校验配置并构建 HTTP 池，**跳过** ping 与 native 探测。
    ///
    /// 用于驱动 fail-closed 与背压路径，无需真实 TDengine。生产入口请使用 [`Self::connect`]。
    pub fn connect_without_ping(config: TaosConfig) -> XResult<Self> {
        config.validate()?;
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(8)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| XError::internal(format!("taos http client: {e}")))?;
        let initial_precision = config.precision.unwrap_or(TsPrecision::Ms);
        let max_in_flight = config.max_in_flight;
        Ok(Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                precision: RwLock::new(initial_precision),
                sem: Arc::new(Semaphore::new(max_in_flight)),
                state: AtomicUsize::new(0),
                drained: Notify::new(),
                metrics: OpCounters::new(),
            }),
        })
    }

    /// 工作客户端。
    #[must_use]
    pub fn client(&self) -> TaosClient {
        self.clone()
    }

    /// 配置。
    #[must_use]
    pub fn config(&self) -> &TaosConfig {
        &self.inner.config
    }

    /// 当前生效精度。
    #[must_use]
    pub fn precision(&self) -> TsPrecision {
        self.inner.precision.read().map(|g| *g).unwrap_or(TsPrecision::Ms)
    }

    /// 池统计。
    #[must_use]
    pub fn stats(&self) -> TaosPoolStats {
        TaosPoolStats {
            in_flight: self.inner.state.load(Ordering::Acquire) & IN_FLIGHT_MASK,
            closed: self.inner.state.load(Ordering::Acquire) & CLOSED_BIT != 0,
        }
    }

    /// 进程内有界操作计数快照（含进程级 WS 探测累计）。
    #[must_use]
    pub fn metrics(&self) -> TaosMetricsSnapshot {
        self.inner.metrics.snapshot()
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.state.load(Ordering::Acquire) & CLOSED_BIT != 0
    }

    /// `SELECT SERVER_VERSION()`。
    pub async fn ping(&self) -> XResult<()> {
        match self.exec_sql("SELECT SERVER_VERSION()").await {
            Ok(r) if r.code == 0 => {
                self.inner.metrics.inc_ping_ok();
                Ok(())
            }
            Ok(r) => {
                self.inner.metrics.inc_ping_err();
                Err(XError::unavailable(format!("taos ping code={}", r.code)))
            }
            Err(e) => {
                self.inner.metrics.inc_ping_err();
                Err(e)
            }
        }
    }

    /// 在配置 database 上下文执行 SQL。
    pub async fn exec_sql(&self, sql: &str) -> XResult<TaosExecResult> {
        self.exec_sql_raw(sql, true).await
    }

    /// 写入序列前确保超级表存在。
    pub async fn ensure_stable(&self, table: &str) -> XResult<()> {
        validate_stable_ident(table)?;
        let sql = format!(
            "CREATE STABLE IF NOT EXISTS `{table}` (\
               ts TIMESTAMP, bid NCHAR(64), ask NCHAR(64)\
             ) TAGS (symbol NCHAR(128))"
        );
        let r = self.exec_sql(&sql).await?;
        if r.code != 0 {
            return Err(map_taos_code(r.code, "ensure_stable 失败"));
        }
        self.verify_decimal_schema(table).await
    }

    /// 显式批量写入：按 `max_rows` 分块 INSERT。
    ///
    /// 空 `points` → `Ok(())`。任一片失败 → `Err`（可能已有部分行提交）。
    pub async fn write_batch(&self, table: &str, points: &[Tick]) -> XResult<()> {
        self.write_batch_report(table, points).await.map(|_| ())
    }

    /// 批量写入并返回 [`BatchWriteReport`]。
    ///
    /// 中途失败时错误由 [`BatchWritePartialError`] 映射为 `XError`，文案含 accepted/failed。
    pub async fn write_batch_report(
        &self,
        table: &str,
        points: &[Tick],
    ) -> XResult<BatchWriteReport> {
        self.write_batch_chunked_report(table, points, self.inner.config.batch_max_rows).await
    }

    /// 带自定义 chunk 大小的批量写入。
    pub async fn write_batch_chunked(
        &self,
        table: &str,
        points: &[Tick],
        max_rows: usize,
    ) -> XResult<()> {
        self.write_batch_chunked_report(table, points, max_rows).await.map(|_| ())
    }

    /// 带自定义 chunk 大小的批量写入，返回结构化报告。
    ///
    /// 空 `points` → `accepted=0` 的完整报告。不自动重试已提交 chunk。
    pub async fn write_batch_chunked_report(
        &self,
        table: &str,
        points: &[Tick],
        max_rows: usize,
    ) -> XResult<BatchWriteReport> {
        match self.write_batch_chunked_outcome(table, points, max_rows).await {
            Ok(report) => Ok(report),
            Err(partial) => Err(partial.into()),
        }
    }

    /// 与 [`Self::write_batch_chunked_report`] 相同，但部分成功时返回结构化
    /// [`BatchWritePartialError`]（含准确 `accepted`/`failed`），而非仅字符串化 `XError`。
    pub async fn write_batch_chunked_outcome(
        &self,
        table: &str,
        points: &[Tick],
        max_rows: usize,
    ) -> Result<BatchWriteReport, BatchWritePartialError> {
        validate_stable_ident(table).map_err(|source| BatchWritePartialError {
            report: BatchWriteReport {
                accepted: 0,
                failed: points.len(),
                chunks_ok: 0,
                chunks_total: 0,
            },
            source,
        })?;
        if points.is_empty() {
            self.inner.metrics.inc_write_ok();
            return Ok(BatchWriteReport::default());
        }
        if max_rows == 0 || max_rows > self.inner.config.batch_max_rows {
            return Err(BatchWritePartialError {
                report: BatchWriteReport {
                    accepted: 0,
                    failed: points.len(),
                    chunks_ok: 0,
                    chunks_total: 0,
                },
                source: XError::invalid(format!(
                    "max_rows 必须为 1..={}（配置上限）",
                    self.inner.config.batch_max_rows
                )),
            });
        }
        if let Err(source) = self.ensure_stable(table).await {
            return Err(BatchWritePartialError {
                report: BatchWriteReport {
                    accepted: 0,
                    failed: points.len(),
                    chunks_ok: 0,
                    chunks_total: 0,
                },
                source,
            });
        }
        let prec = self.precision();
        let chunks = match build_insert_sql_chunks_with_limits(
            table,
            points,
            prec,
            max_rows,
            self.inner.config.batch_max_bytes,
        ) {
            Ok(c) => c,
            Err(source) => {
                return Err(BatchWritePartialError {
                    report: BatchWriteReport {
                        accepted: 0,
                        failed: points.len(),
                        chunks_ok: 0,
                        chunks_total: 0,
                    },
                    source,
                });
            }
        };
        let chunks_total = chunks.len();
        let mut accepted = 0usize;
        let mut chunks_ok = 0usize;
        for (sql, row_count) in chunks {
            match self.exec_sql(&sql).await {
                Ok(r) if r.code == 0 => {
                    accepted += row_count;
                    chunks_ok += 1;
                }
                Ok(r) => {
                    let failed = points.len().saturating_sub(accepted);
                    self.inner.metrics.inc_write_err();
                    return Err(BatchWritePartialError {
                        report: BatchWriteReport { accepted, failed, chunks_ok, chunks_total },
                        source: map_taos_code(r.code, "write_batch 失败"),
                    });
                }
                Err(source) => {
                    let failed = points.len().saturating_sub(accepted);
                    self.inner.metrics.inc_write_err();
                    return Err(BatchWritePartialError {
                        report: BatchWriteReport { accepted, failed, chunks_ok, chunks_total },
                        source,
                    });
                }
            }
        }
        self.inner.metrics.inc_write_ok();
        Ok(BatchWriteReport { accepted, failed: 0, chunks_ok, chunks_total })
    }

    /// 关闭池。
    pub async fn close(&self) -> XResult<()> {
        self.inner.state.fetch_or(CLOSED_BIT, Ordering::AcqRel);
        self.inner.sem.close();
        let drain = async {
            loop {
                let notified = self.inner.drained.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if self.inner.state.load(Ordering::Acquire) & IN_FLIGHT_MASK == 0 {
                    return;
                }
                notified.await;
            }
        };
        timeout(self.inner.config.close_timeout, drain)
            .await
            .map_err(|_| XError::deadline_exceeded("taos close 等待在途请求排空超时"))?;
        Ok(())
    }

    async fn verify_decimal_schema(&self, table: &str) -> XResult<()> {
        let result = self.exec_sql(&format!("DESCRIBE `{table}`")).await?;
        validate_decimal_schema(&result)
    }

    async fn detect_precision(&self) -> XResult<TsPrecision> {
        let db = self.inner.config.database.clone();
        validate_ident(&db)?;
        let sql =
            format!("SELECT `precision` FROM information_schema.ins_databases WHERE name='{db}'");
        let r = self.exec_sql_raw(&sql, false).await?;
        r.rows
            .first()
            .and_then(|row| row.first())
            .and_then(|value| TsPrecision::parse(value))
            .ok_or_else(|| XError::invariant("taos 无法从 information_schema 探测数据库精度"))
    }

    async fn acquire(&self) -> XResult<RequestGuard> {
        self.ensure_open()?;
        let result =
            timeout(self.inner.config.acquire_timeout, self.inner.sem.clone().acquire_owned())
                .await;
        match result {
            Ok(Ok(permit)) => loop {
                let state = self.inner.state.load(Ordering::Acquire);
                if state & CLOSED_BIT != 0 {
                    drop(permit);
                    return Err(XError::unavailable("taos pool 已关闭"));
                }
                let next = state
                    .checked_add(1)
                    .ok_or_else(|| XError::invariant("taos in-flight 计数溢出"))?;
                if self
                    .inner
                    .state
                    .compare_exchange_weak(state, next, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return Ok(RequestGuard { _permit: permit, inner: Arc::clone(&self.inner) });
                }
            },
            Ok(Err(_)) => Err(XError::unavailable("taos 背压信号量已关闭")),
            Err(_) => Err(XError::deadline_exceeded(format!(
                "taos 获取 in-flight 许可超时（max={}）",
                self.inner.config.max_in_flight
            ))),
        }
    }

    async fn exec_sql_raw(&self, sql: &str, use_db: bool) -> XResult<TaosExecResult> {
        if sql.len() > self.inner.config.batch_max_bytes {
            self.inner.metrics.inc_sql_err();
            return Err(XError::invalid(format!(
                "SQL 请求超过 batch_max_bytes={} 字节",
                self.inner.config.batch_max_bytes
            )));
        }
        let _guard = self.acquire().await?;
        match self.exec_sql_raw_inner(sql, use_db).await {
            Ok(r) => {
                self.inner.metrics.inc_sql_ok();
                Ok(r)
            }
            Err(e) => {
                self.inner.metrics.inc_sql_err();
                Err(e)
            }
        }
    }

    async fn exec_sql_raw_inner(&self, sql: &str, use_db: bool) -> XResult<TaosExecResult> {
        let cfg = &self.inner.config;
        let url = if use_db { cfg.rest_sql_db_url() } else { cfg.rest_sql_url() };

        debug!(target: "taosx", database = %cfg.database, "taos rest sql");

        let resp = self
            .inner
            .http
            .post(&url)
            .basic_auth(&cfg.user, Some(&cfg.password))
            .header(reqwest::header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    XError::deadline_exceeded(format!("taos 超时: {e}"))
                } else {
                    XError::unavailable(format!("taos 请求失败: {e}"))
                }
            })?;

        let status = resp.status();
        let text = read_response_limited(resp, cfg.max_response_bytes).await?;

        if !status.is_success() {
            return Err(XError::unavailable(format!(
                "taos HTTP {status}: {}",
                truncate(&text, 256)
            )));
        }

        let result = parse_taos_json(&text)?;
        if result.rows.len() > cfg.max_query_rows {
            return Err(XError::unavailable(format!(
                "taos SQL 结果超过 max_query_rows={}",
                cfg.max_query_rows
            )));
        }
        Ok(result)
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.is_closed() {
            return Err(XError::unavailable("taos pool 已关闭"));
        }
        Ok(())
    }
}

fn parse_taos_json(text: &str) -> XResult<TaosExecResult> {
    let raw: RawResponse = serde_json::from_str(text).map_err(|e| {
        XError::internal(format!("taos JSON 解析失败: {e}; body={}", truncate(text, 256)))
    })?;

    if raw.code != 0 {
        let desc = raw.desc.unwrap_or_default();
        return Err(map_taos_code(raw.code, &desc));
    }

    let columns = raw
        .column_meta
        .iter()
        .filter_map(|c| {
            c.as_array().and_then(|a| a.first()).and_then(|v| v.as_str()).map(str::to_string)
        })
        .collect::<Vec<_>>();

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(raw.data.len());
    for row in raw.data {
        rows.push(row.iter().map(json_cell_to_string).collect());
    }

    let affected_rows = if columns.first().map(String::as_str) == Some("affected_rows") {
        rows.first().and_then(|r| r.first()).and_then(|s| s.parse().ok())
    } else {
        raw.rows
    };

    Ok(TaosExecResult { code: raw.code, rows, columns, affected_rows })
}

async fn read_response_limited(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> XResult<String> {
    if response
        .content_length()
        .is_some_and(|length| length > u64::try_from(max_bytes).unwrap_or(u64::MAX))
    {
        return Err(XError::unavailable(format!("taos 响应超过 max_response_bytes={max_bytes}")));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| XError::unavailable("taos 读响应失败").with_source(error))?
    {
        let next_len = body
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| XError::unavailable("taos 响应字节数溢出"))?;
        if next_len > max_bytes {
            return Err(XError::unavailable(format!(
                "taos 响应超过 max_response_bytes={max_bytes}"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    String::from_utf8(body)
        .map_err(|error| XError::invalid("taos 响应不是 UTF-8").with_source(error))
}

fn validate_decimal_schema(result: &TaosExecResult) -> XResult<()> {
    let mut bid_ok = false;
    let mut ask_ok = false;
    for row in &result.rows {
        if row.len() < 3 {
            continue;
        }
        let field = row[0].trim();
        let data_type = row[1].trim();
        let length_ok = row[2].trim().parse::<usize>().is_ok_and(|length| length >= 64);
        let exact_text = data_type.eq_ignore_ascii_case("NCHAR") && length_ok;
        if field.eq_ignore_ascii_case("bid") {
            bid_ok = exact_text;
        } else if field.eq_ignore_ascii_case("ask") {
            ask_ok = exact_text;
        }
    }
    if !bid_ok || !ask_ok {
        return Err(XError::conflict(
            "TDengine schema 不兼容：bid/ask 必须为 NCHAR(64+)；拒绝 DOUBLE 精度降级",
        ));
    }
    Ok(())
}

#[async_trait]
impl TimeSeriesStore for TaosPool {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()> {
        // 委托显式批量 API
        self.write_batch(table, &points).await
    }

    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        let result = self.query_series_inner(table, start, end).await;
        match &result {
            Ok(_) => self.inner.metrics.inc_query_ok(),
            Err(_) => self.inner.metrics.inc_query_err(),
        }
        result
    }
}

impl TaosPool {
    async fn query_series_inner(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        validate_stable_ident(table)?;
        if start > end {
            return Err(XError::invalid("query_series: start > end"));
        }
        if let Err(error) = self.verify_decimal_schema(table).await {
            if error.kind() == kernel::ErrorKind::Missing {
                return Ok(Vec::new());
            }
            return Err(error);
        }
        let prec = self.precision();
        let start_db = prec.from_nanos(start);
        let end_db = prec.from_nanos(end);
        let limit = self
            .inner
            .config
            .max_query_rows
            .checked_add(1)
            .ok_or_else(|| XError::invariant("max_query_rows 溢出"))?;
        let sql = format!(
            "SELECT ts, bid, ask, symbol FROM `{table}` WHERE ts >= {start_db} AND ts <= {end_db} ORDER BY ts ASC LIMIT {limit}"
        );
        let r = match self.exec_sql(&sql).await {
            Ok(r) => r,
            Err(e) => {
                // 表不存在时返回空集，便于首次查询；依赖 map_taos_code 的类型化
                // ErrorKind::Missing，而非对驱动错误文案做脆弱字符串匹配。
                if e.kind() == kernel::ErrorKind::Missing {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };

        if r.rows.len() > self.inner.config.max_query_rows {
            return Err(XError::unavailable(format!(
                "taos 查询结果超过 max_query_rows={}",
                self.inner.config.max_query_rows
            )));
        }

        let mut out = Vec::with_capacity(r.rows.len());
        for row in r.rows {
            if row.len() < 4 {
                continue;
            }
            let ts = parse_ts_cell(&row[0], prec)?;
            let bid = parse_decimal_cell(&row[1])?;
            let ask = parse_decimal_cell(&row[2])?;
            let symbol = row[3].clone();
            out.push(Tick { symbol, bid: Price::new(bid), ask: Price::new(ask), ts });
        }
        Ok(out)
    }
}

fn subtable_name(stable: &str, symbol: &str) -> XResult<String> {
    validate_stable_ident(stable)?;
    if symbol.len() > MAX_SYMBOL_BYTES {
        return Err(XError::invalid(format!("symbol 超过 {MAX_SYMBOL_BYTES} UTF-8 字节")));
    }
    let mut encoded = String::with_capacity(symbol.len().saturating_mul(2));
    for byte in symbol.as_bytes() {
        write!(&mut encoded, "{byte:02x}").map_err(|_| XError::invariant("symbol 子表编码失败"))?;
    }
    let name = format!("{stable}_s{encoded}");
    validate_ident(&name)?;
    Ok(name)
}

fn validate_stable_ident(name: &str) -> XResult<()> {
    validate_ident(name)?;
    if name.len() > MAX_STABLE_NAME_BYTES {
        return Err(XError::invalid(format!("stable 名称超过 {MAX_STABLE_NAME_BYTES} 字节")));
    }
    Ok(())
}

fn validate_ident(name: &str) -> XResult<()> {
    if name.is_empty() || name.len() > 192 {
        return Err(XError::invalid(format!("非法标识符长度: {name}")));
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(XError::invalid("空标识符"));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(XError::invalid(format!("标识符须以字母或下划线开头: {name}")));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(XError::invalid(format!("标识符含非法字符: {name}")));
    }
    Ok(())
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn json_cell_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn parse_ts_cell(raw: &str, prec: TsPrecision) -> XResult<i64> {
    if let Ok(n) = raw.parse::<i64>() {
        return Ok(prec.to_nanos(n));
    }
    // RFC3339（TDengine REST 常返回此格式）
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Ok(dt.timestamp_nanos_opt().unwrap_or_else(|| dt.timestamp() * 1_000_000_000));
    }
    // 兜底：当作 UTC naive
    if let Ok(dt) = DateTime::<Utc>::from_str(raw) {
        return Ok(dt.timestamp_nanos_opt().unwrap_or_else(|| dt.timestamp() * 1_000_000_000));
    }
    Err(XError::invalid(format!("无法解析时间戳: {raw}")))
}

fn parse_decimal_cell(raw: &str) -> XResult<Decimal> {
    Decimal::from_str(raw).map_err(|e| XError::invalid(format!("小数解析失败 '{raw}': {e}")))
}

fn map_taos_code(code: i32, ctx: &str) -> XError {
    let msg = if ctx.is_empty() {
        format!("taos code={code}")
    } else {
        format!("taos code={code}: {ctx}")
    };
    // TDengine 错误码在 REST 里多为正整数；大范围 code 按语义粗分
    match code {
        896 => XError::unavailable(msg),
        0x2603 | 9826 => XError::missing(msg),
        c if c > 0 => XError::invalid(msg),
        _ => XError::internal(msg),
    }
}

fn truncate(s: &str, max: usize) -> String {
    let mut t = s.trim().replace('\n', " ");
    if t.len() > max {
        let mut boundary = max;
        while !t.is_char_boundary(boundary) {
            boundary -= 1;
        }
        t.truncate(boundary);
        t.push('…');
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use kernel::ErrorKind;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn sample_tick(symbol: &str, ts: i64) -> Tick {
        Tick {
            symbol: symbol.into(),
            bid: Price::new(Decimal::from_str("1.0").unwrap()),
            ask: Price::new(Decimal::from_str("1.1").unwrap()),
            ts,
        }
    }

    #[test]
    fn subtable_sanitizes() {
        let n = subtable_name("ticks", "BTC/USDT").unwrap();
        assert!(n.starts_with("ticks_"));
        assert!(!n.contains('/'));
    }

    #[test]
    fn reject_bad_ident() {
        assert!(validate_ident("a b").is_err());
    }

    #[test]
    fn parse_rfc3339_ts() {
        let ns = parse_ts_cell("2026-07-21T17:12:39.582758368Z", TsPrecision::Ns).unwrap();
        assert!(ns > 0);
    }

    #[test]
    fn insert_sql_chunks() {
        let points: Vec<Tick> = (0..5).map(|i| sample_tick("BTC", i * 1_000_000)).collect();
        let chunks = build_insert_sql_chunks("ticks", &points, TsPrecision::Ms, 2).unwrap();
        assert_eq!(chunks.len(), 3); // 2+2+1
        assert!(chunks[0].starts_with("INSERT INTO "));
        assert!(chunks[0].contains("VALUES"));
        assert!(chunks[0].contains("'1'"));
        // 空
        assert!(build_insert_sql_chunks("ticks", &[], TsPrecision::Ms, 10).unwrap().is_empty());
    }

    #[test]
    fn decimal_text_path_preserves_i128_scale_18() {
        let bid =
            Decimal::try_new(123_456_789_012_345_678_901_234_567_890_123_456, 18).expect("bid");
        let ask_mantissa = -123_456_789_012_345_678_901_234_567_890_123_455;
        let ask = Decimal::try_new(ask_mantissa, 18).expect("ask");
        let tick =
            Tick { symbol: "BTC/USDT".into(), bid: Price::new(bid), ask: Price::new(ask), ts: 1 };
        let sql =
            build_insert_sql_chunks("ticks", &[tick], TsPrecision::Ns, 1).expect("build").remove(0);
        assert!(sql.contains(&format!("'{}'", bid)));
        assert!(sql.contains(&format!("'{}'", ask)));
        assert_eq!(parse_decimal_cell(&bid.to_string()).expect("parse bid"), bid);
        assert_eq!(parse_decimal_cell(&ask.to_string()).expect("parse ask"), ask);
    }

    #[test]
    fn schema_rejects_double_and_accepts_nchar_64() {
        let double = TaosExecResult {
            code: 0,
            rows: vec![
                vec!["bid".into(), "DOUBLE".into(), "8".into()],
                vec!["ask".into(), "DOUBLE".into(), "8".into()],
            ],
            columns: Vec::new(),
            affected_rows: None,
        };
        assert_eq!(
            validate_decimal_schema(&double).expect_err("double must fail").kind(),
            ErrorKind::Conflict
        );

        let text = TaosExecResult {
            code: 0,
            rows: vec![
                vec!["bid".into(), "NCHAR".into(), "64".into()],
                vec!["ask".into(), "NCHAR".into(), "64".into()],
            ],
            columns: Vec::new(),
            affected_rows: None,
        };
        validate_decimal_schema(&text).expect("nchar schema");
    }

    #[test]
    fn batch_bytes_and_symbol_mapping_are_bounded() {
        let first = subtable_name("ticks", "BTC/USDT").expect("first");
        let second = subtable_name("ticks", "BTC_USDT").expect("second");
        assert_ne!(first, second);

        let tick = sample_tick(&"X".repeat(MAX_SYMBOL_BYTES), 1);
        let error = build_insert_sql_chunks_with_limits("ticks", &[tick], TsPrecision::Ns, 1, 20)
            .expect_err("single row exceeds byte cap");
        assert_eq!(error.kind(), ErrorKind::Invalid);

        assert!(subtable_name("ticks", &"X".repeat(MAX_SYMBOL_BYTES + 1)).is_err());
        assert!(
            build_insert_sql_chunks("ticks", &[], TsPrecision::Ns, HARD_MAX_BATCH_ROWS + 1)
                .is_err()
        );
    }

    #[test]
    fn truncate_is_utf8_boundary_safe() {
        assert_eq!(truncate("中文响应", 2), "…");
        assert_eq!(truncate("中文响应", 4), "中…");
    }

    #[tokio::test]
    async fn connect_refused_fails() {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1,
            timeout: Duration::from_millis(300),
            acquire_timeout: Duration::from_millis(300),
            ..TaosConfig::default()
        };
        match TaosPool::connect(cfg).await {
            Ok(p) => {
                let err = p.ping().await.expect_err("ping must fail");
                assert!(
                    matches!(
                        err.kind(),
                        ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
                    ),
                    "kind={:?}",
                    err.kind()
                );
            }
            Err(e) => {
                assert!(
                    matches!(
                        e.kind(),
                        ErrorKind::Unavailable | ErrorKind::DeadlineExceeded | ErrorKind::Transient
                    ),
                    "kind={:?}",
                    e.kind()
                );
            }
        }
    }

    #[tokio::test]
    async fn closed_pool_rejects() {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1,
            max_in_flight: 2,
            acquire_timeout: Duration::from_millis(100),
            timeout: Duration::from_millis(100),
            ..TaosConfig::default()
        };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        assert_eq!(pool.stats().in_flight, 0);
        pool.close().await.expect("close");
        assert!(pool.stats().closed);
        let err = pool.exec_sql("SELECT 1").await.expect_err("closed");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn acquire_timeout_when_saturated() {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1,
            max_in_flight: 1,
            acquire_timeout: Duration::from_millis(50),
            timeout: Duration::from_millis(200),
            ..TaosConfig::default()
        };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        // 占住唯一许可
        let permit = pool.acquire().await.expect("first permit");
        let err = match pool.acquire().await {
            Ok(_) => panic!("second must timeout"),
            Err(error) => error,
        };
        assert_eq!(err.kind(), ErrorKind::DeadlineExceeded);
        drop(permit);
    }

    #[tokio::test]
    async fn close_deadline_waits_for_in_flight_and_is_repeatable() {
        let cfg = TaosConfig { close_timeout: Duration::from_millis(20), ..TaosConfig::default() };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let guard = pool.acquire().await.expect("guard");
        assert_eq!(pool.stats().in_flight, 1);
        let error = pool.close().await.expect_err("close must time out while busy");
        assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
        assert!(pool.is_closed());
        drop(guard);
        pool.close().await.expect("repeat close drains");
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[tokio::test]
    async fn cancelled_task_releases_in_flight_guard() {
        let pool = TaosPool::connect_without_ping(TaosConfig::default()).expect("build");
        let worker = {
            let pool = pool.clone();
            tokio::spawn(async move {
                let _guard = pool.acquire().await.expect("guard");
                std::future::pending::<()>().await;
            })
        };
        tokio::time::timeout(Duration::from_secs(1), async {
            while pool.stats().in_flight == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("worker acquired");
        worker.abort();
        let _ = worker.await;
        assert_eq!(pool.stats().in_flight, 0);
        pool.close().await.expect("close after cancellation");
    }

    async fn serve_response(status: &'static str, body: &'static str, chunked: bool) -> u16 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut request = [0u8; 4096];
            let _ = stream.read(&mut request).await.expect("read request");
            let response = if chunked {
                format!(
                    "HTTP/1.1 {status}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",
                    body.len()
                )
            } else {
                format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
            };
            stream.write_all(response.as_bytes()).await.expect("write response");
        });
        port
    }

    /// 依序为多个连续请求（各自独立连接，`Connection: close`）返回预设 JSON body。
    ///
    /// 用于驱动 `connect()` 内部 `CREATE DATABASE` → 精度探测 → `ping` 的多步 REST 序列。
    async fn serve_sequence(bodies: Vec<&'static str>) -> u16 {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        tokio::spawn(async move {
            for body in bodies {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request).await.expect("read request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.expect("write response");
            }
        });
        port
    }

    #[tokio::test]
    async fn response_limit_applies_to_success_and_error_bodies() {
        for (status, chunked) in [("200 OK", true), ("500 Internal Server Error", false)] {
            let port = serve_response(status, "中文响应体超过上限", chunked).await;
            let cfg = TaosConfig {
                port,
                max_response_bytes: 8,
                timeout: Duration::from_secs(1),
                ..TaosConfig::default()
            };
            let pool = TaosPool::connect_without_ping(cfg).expect("build");
            let error = pool.exec_sql("SELECT 1").await.expect_err("body too large");
            assert_eq!(error.kind(), ErrorKind::Unavailable);
        }
    }

    #[tokio::test]
    async fn public_exec_sql_cannot_bypass_query_row_limit() {
        let body = r#"{"code":0,"column_meta":[["value","INT",4]],"data":[[1],[2]],"rows":2}"#;
        let port = serve_response("200 OK", body, false).await;
        let cfg = TaosConfig { port, max_query_rows: 1, ..TaosConfig::default() };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let error = pool.exec_sql("SELECT value").await.expect_err("row cap");
        assert_eq!(error.kind(), ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn rejects_zero_max_in_flight() {
        let cfg = TaosConfig { max_in_flight: 0, ..TaosConfig::default() };
        let err = match TaosPool::connect(cfg).await {
            Ok(_) => panic!("must reject zero max_in_flight"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn public_sql_and_custom_chunk_cannot_bypass_config_limits() {
        let cfg = TaosConfig {
            port: 1,
            batch_max_rows: 1,
            batch_max_bytes: INSERT_PREFIX.len(),
            ..TaosConfig::default()
        };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let sql_error = pool.exec_sql("SELECT 123456789").await.expect_err("sql byte cap");
        assert_eq!(sql_error.kind(), ErrorKind::Invalid);
        let batch_error = pool
            .write_batch_chunked("ticks", &[sample_tick("BTC", 1)], 2)
            .await
            .expect_err("custom row cap");
        assert_eq!(batch_error.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn write_batch_report_partial_failure_exposes_accepted_rows() {
        // 序列：ensure_stable CREATE + DESCRIBE + chunk0 OK + chunk1 FAIL
        let create_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#;
        let describe_ok = concat!(
            r#"{"code":0,"column_meta":[["field","VARCHAR",16],["type","VARCHAR",16],["length","VARCHAR",8]],"#,
            r#""data":[["ts","TIMESTAMP","8"],["bid","NCHAR","64"],["ask","NCHAR","64"]],"rows":3}"#
        );
        let insert_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0,"affected_rows":1}"#;
        let insert_fail = r#"{"code":-1,"desc":"injected write failure"}"#;
        let port = serve_sequence(vec![create_ok, describe_ok, insert_ok, insert_fail]).await;
        let cfg = TaosConfig {
            port,
            database: String::new(),
            batch_max_rows: 1,
            timeout: Duration::from_secs(2),
            ..TaosConfig::default()
        };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let points = vec![sample_tick("BTC", 1_000_000), sample_tick("ETH", 2_000_000)];
        match pool.write_batch_chunked_outcome("ticks", &points, 1).await {
            Ok(_) => panic!("second chunk must fail"),
            Err(partial) => {
                assert_eq!(partial.report.accepted, 1);
                assert_eq!(partial.report.failed, 1);
                assert_eq!(partial.report.chunks_ok, 1);
                assert_eq!(partial.report.chunks_total, 2);
                assert!(!partial.report.is_complete());
                let as_xerror: XError = partial.into();
                let text = as_xerror.to_string();
                assert!(text.contains("accepted=1"), "{text}");
                assert!(text.contains("failed=1"), "{text}");
            }
        }
    }

    #[tokio::test]
    async fn write_batch_report_full_success() {
        let create_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#;
        let describe_ok = concat!(
            r#"{"code":0,"column_meta":[["field","VARCHAR",16],["type","VARCHAR",16],["length","VARCHAR",8]],"#,
            r#""data":[["ts","TIMESTAMP","8"],["bid","NCHAR","64"],["ask","NCHAR","64"]],"rows":3}"#
        );
        let insert_ok = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#;
        let port = serve_sequence(vec![create_ok, describe_ok, insert_ok]).await;
        let cfg = TaosConfig {
            port,
            database: String::new(),
            batch_max_rows: 10,
            timeout: Duration::from_secs(2),
            ..TaosConfig::default()
        };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let report =
            pool.write_batch_report("ticks", &[sample_tick("BTC", 1)]).await.expect("full write");
        assert_eq!(report.accepted, 1);
        assert_eq!(report.failed, 0);
        assert!(report.is_complete());
        let m = pool.metrics();
        assert!(m.write_ok >= 1, "write_ok={m:?}");
        assert!(m.sql_ok >= 1, "sql_ok={m:?}");
    }

    #[tokio::test]
    async fn connect_rejects_configured_precision_mismatch() {
        // 序列：CREATE DATABASE（无 use_db） → 精度探测（information_schema）。
        // 数据库实际精度为 "us"，但配置显式声明 Ms，必须 fail-closed 而不是静默采用探测值。
        let create_db_body = r#"{"code":0,"column_meta":[],"data":[],"rows":0}"#;
        let precision_body =
            r#"{"code":0,"column_meta":[["precision","VARCHAR",8]],"data":[["us"]],"rows":1}"#;
        let port = serve_sequence(vec![create_db_body, precision_body]).await;
        let cfg = TaosConfig {
            port,
            database: "infra_draft".into(),
            precision: Some(TsPrecision::Ms),
            timeout: Duration::from_secs(2),
            ..TaosConfig::default()
        };
        // TaosPool 未 derive Debug，match 而非 expect_err/unwrap_err 取错误分支。
        match TaosPool::connect(cfg).await {
            Ok(_) => panic!("precision mismatch must fail-closed"),
            Err(error) => assert_eq!(error.kind(), ErrorKind::Invalid),
        }
    }

    #[tokio::test]
    async fn query_series_missing_table_relies_on_typed_error_kind_not_message_text() {
        // DESCRIBE 与 SELECT 均返回 TDengine「表不存在」错误码（0x2603 = 9731），
        // map_taos_code 必须把它类型化为 ErrorKind::Missing，
        // query_series 据此返回空集，而不是依赖对错误文案做子串匹配。
        let describe_error = r#"{"code":9731,"desc":"Table does not exist"}"#;
        let port = serve_response("200 OK", describe_error, false).await;
        let cfg = TaosConfig { port, timeout: Duration::from_secs(2), ..TaosConfig::default() };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let rows = pool.query_series("missing_table", 0, 1).await.expect("missing table => empty");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn query_series_propagates_non_missing_describe_errors() {
        // DESCRIBE 失败但错误码不是「表不存在」语义时，必须原样传播错误，
        // 不能被误判为空表而静默吞掉。
        let internal_error = r#"{"code":-1,"desc":"internal driver failure"}"#;
        let port = serve_response("200 OK", internal_error, false).await;
        let cfg = TaosConfig { port, timeout: Duration::from_secs(2), ..TaosConfig::default() };
        let pool = TaosPool::connect_without_ping(cfg).expect("build");
        let error = pool.query_series("some_table", 0, 1).await.expect_err("must propagate");
        assert_eq!(error.kind(), ErrorKind::Internal);
    }
}
