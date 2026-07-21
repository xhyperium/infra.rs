//! TDengine REST 生产客户端（默认 6041）。

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use canonical::Tick;
use chrono::{DateTime, Utc};
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use kernel::{XError, XResult};
use serde::Deserialize;
use tracing::debug;

use crate::config::{TaosConfig, TsPrecision};

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
    closed: AtomicBool,
}

/// 工作句柄别名。
pub type TaosClient = TaosPool;

impl TaosPool {
    /// 连接：构建 HTTP 客户端、可选建库、探测精度、ping。
    pub async fn connect(config: TaosConfig) -> XResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(8)
            .build()
            .map_err(|e| XError::internal(format!("taos http client: {e}")))?;

        let initial_precision = config.precision.unwrap_or(TsPrecision::Ms);
        let pool = Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                precision: RwLock::new(initial_precision),
                closed: AtomicBool::new(false),
            }),
        };

        // 确保 database 存在（失败时留给后续 SQL 暴露权限问题）
        if !pool.inner.config.database.is_empty() {
            let db = pool.inner.config.database.clone();
            validate_ident(&db)?;
            let _ = pool
                .exec_sql_raw(&format!("CREATE DATABASE IF NOT EXISTS `{db}` KEEP 3650"), false)
                .await;
            if let Ok(p) = pool.detect_precision().await {
                if let Ok(mut g) = pool.inner.precision.write() {
                    *g = p;
                }
            }
        }

        pool.ping().await?;
        Ok(pool)
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(TaosConfig::from_env()).await
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

    /// `SELECT SERVER_VERSION()`。
    pub async fn ping(&self) -> XResult<()> {
        let r = self.exec_sql("SELECT SERVER_VERSION()").await?;
        if r.code != 0 {
            return Err(XError::unavailable(format!("taos ping code={}", r.code)));
        }
        Ok(())
    }

    /// 在配置 database 上下文执行 SQL。
    pub async fn exec_sql(&self, sql: &str) -> XResult<TaosExecResult> {
        self.exec_sql_raw(sql, true).await
    }

    /// 写入序列前确保超级表存在。
    pub async fn ensure_stable(&self, table: &str) -> XResult<()> {
        validate_ident(table)?;
        let sql = format!(
            "CREATE STABLE IF NOT EXISTS `{table}` (\
               ts TIMESTAMP, bid DOUBLE, ask DOUBLE\
             ) TAGS (symbol NCHAR(128))"
        );
        let r = self.exec_sql(&sql).await?;
        if r.code != 0 {
            return Err(map_taos_code(r.code, "ensure_stable 失败"));
        }
        Ok(())
    }

    /// 关闭池。
    pub async fn close(&self) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn detect_precision(&self) -> XResult<TsPrecision> {
        let db = self.inner.config.database.clone();
        validate_ident(&db)?;
        let sql =
            format!("SELECT `precision` FROM information_schema.ins_databases WHERE name='{db}'");
        let r = self.exec_sql_raw(&sql, false).await?;
        if let Some(row) = r.rows.first() {
            if let Some(p) = row.first().and_then(|s| TsPrecision::parse(s)) {
                return Ok(p);
            }
        }
        Ok(self.precision())
    }

    async fn exec_sql_raw(&self, sql: &str, use_db: bool) -> XResult<TaosExecResult> {
        self.ensure_open()?;
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
        let text =
            resp.text().await.map_err(|e| XError::unavailable(format!("taos 读响应失败: {e}")))?;

        if !status.is_success() {
            return Err(XError::unavailable(format!(
                "taos HTTP {status}: {}",
                truncate(&text, 256)
            )));
        }

        parse_taos_json(&text)
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::SeqCst) {
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

#[async_trait]
impl TimeSeriesStore for TaosPool {
    async fn write_series(&self, table: &str, points: Vec<Tick>) -> XResult<()> {
        validate_ident(table)?;
        if points.is_empty() {
            return Ok(());
        }
        self.ensure_stable(table).await?;
        let prec = self.precision();

        // 多表批量 INSERT
        let mut sql = String::from("INSERT INTO ");
        for (i, tick) in points.iter().enumerate() {
            if i > 0 {
                sql.push(' ');
            }
            let sub = subtable_name(table, &tick.symbol)?;
            let sym = escape_str(&tick.symbol);
            let ts = prec.from_nanos(tick.ts);
            let bid = tick.bid.as_decimal().to_string();
            let ask = tick.ask.as_decimal().to_string();
            sql.push_str(&format!(
                "`{sub}` USING `{table}` TAGS ('{sym}') VALUES ({ts},{bid},{ask})"
            ));
        }
        let _ = self.exec_sql(&sql).await?;
        Ok(())
    }

    async fn query_series(&self, table: &str, start: i64, end: i64) -> XResult<Vec<Tick>> {
        validate_ident(table)?;
        if start > end {
            return Err(XError::invalid("query_series: start > end"));
        }
        let prec = self.precision();
        let start_db = prec.from_nanos(start);
        let end_db = prec.from_nanos(end);
        let sql = format!(
            "SELECT ts, bid, ask, symbol FROM `{table}` WHERE ts >= {start_db} AND ts <= {end_db} ORDER BY ts ASC"
        );
        let r = match self.exec_sql(&sql).await {
            Ok(r) => r,
            Err(e) => {
                // 表不存在时返回空集，便于首次查询
                let msg = format!("{e}");
                if msg.contains("Table does not exist")
                    || msg.contains("not exist")
                    || msg.contains("0x2603")
                {
                    return Ok(Vec::new());
                }
                return Err(e);
            }
        };

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
    let mut slug: String = symbol
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    if slug.is_empty() {
        slug = "sym".into();
    }
    if slug.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        slug.insert(0, 't');
    }
    let max = 180usize.saturating_sub(stable.len());
    if slug.len() > max {
        slug.truncate(max.max(8));
    }
    let name = format!("{stable}_{slug}");
    validate_ident(&name)?;
    Ok(name)
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
        t.truncate(max);
        t.push('…');
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use kernel::ErrorKind;

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

    #[tokio::test]
    async fn connect_refused_fails() {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1,
            timeout: Duration::from_millis(300),
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
}
