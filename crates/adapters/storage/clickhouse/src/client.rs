//! ClickHouse HTTP 生产客户端（8123）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::AnalyticsSink;
use kernel::{XError, XResult};
use reqwest::StatusCode;
use serde_json::Value;
use tracing::debug;

use crate::config::ClickHouseConfig;

/// 默认 analytics sink 表名。
pub const ANALYTICS_TABLE: &str = "analytics_events";

/// 共享 HTTP 连接资源 + 配置。
///
/// 克隆廉价（`Arc`）；`close` 后拒绝新请求。
#[derive(Clone)]
pub struct ClickHousePool {
    inner: Arc<PoolInner>,
}

struct PoolInner {
    http: reqwest::Client,
    config: ClickHouseConfig,
    closed: AtomicBool,
}

/// 池上的工作句柄（与 [`ClickHousePool`] 等价，便于命名区分）。
pub type ClickHouseClient = ClickHousePool;

impl ClickHousePool {
    /// 使用配置建立 HTTP 客户端并 `ping`。
    pub async fn connect(config: ClickHouseConfig) -> XResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(8)
            .build()
            .map_err(|e| XError::internal(format!("clickhouse http client: {e}")))?;
        let pool =
            Self { inner: Arc::new(PoolInner { http, config, closed: AtomicBool::new(false) }) };
        pool.ping().await?;
        Ok(pool)
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(ClickHouseConfig::from_env()).await
    }

    /// 返回工作客户端（当前即 `self` 的克隆）。
    #[must_use]
    pub fn client(&self) -> ClickHouseClient {
        self.clone()
    }

    /// 配置引用。
    #[must_use]
    pub fn config(&self) -> &ClickHouseConfig {
        &self.inner.config
    }

    /// `SELECT 1` 健康检查。
    pub async fn ping(&self) -> XResult<()> {
        let body = self.query_text("SELECT 1").await?;
        if body.trim() != "1" {
            return Err(XError::unavailable(format!("clickhouse ping 异常响应: {}", body.trim())));
        }
        Ok(())
    }

    /// 执行不返回行的 SQL（DDL / INSERT 等）。
    pub async fn execute(&self, sql: &str) -> XResult<()> {
        let _ = self.post_query(sql, None).await?;
        Ok(())
    }

    /// 执行查询，返回响应文本（默认 TabSeparated / 纯文本）。
    pub async fn query_text(&self, sql: &str) -> XResult<String> {
        self.post_query(sql, None).await
    }

    /// 按行拆分查询结果（TabSeparated 默认）。
    pub async fn query_rows(&self, sql: &str) -> XResult<Vec<Vec<String>>> {
        let text = self.query_text(sql).await?;
        let mut rows = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            rows.push(line.split('\t').map(str::to_string).collect());
        }
        Ok(rows)
    }

    /// 以 `JSONEachRow` 批量插入。
    ///
    /// `rows` 中每个 `Value` 必须是 object。
    pub async fn insert_json_each_row(&self, table: &str, rows: &[Value]) -> XResult<()> {
        validate_ident(table)?;
        if rows.is_empty() {
            return Ok(());
        }
        let mut body = String::new();
        for row in rows {
            if !row.is_object() {
                return Err(XError::invalid("insert 行必须为 JSON object"));
            }
            body.push_str(&row.to_string());
            body.push('\n');
        }
        let sql = format!("INSERT INTO {table} FORMAT JSONEachRow");
        let _ = self.post_query(&sql, Some(body)).await?;
        Ok(())
    }

    /// 确保 analytics 表存在（MergeTree）。
    pub async fn ensure_analytics_table(&self) -> XResult<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {ANALYTICS_TABLE} (\
               ts DateTime64(3) DEFAULT now64(3),\
               event String,\
               payload String\
             ) ENGINE = MergeTree ORDER BY (event, ts)"
        );
        self.execute(&sql).await
    }

    /// 关闭池：拒绝后续请求（HTTP 连接由 Drop 回收）。
    pub async fn close(&self) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(XError::unavailable("clickhouse pool 已关闭"));
        }
        Ok(())
    }

    async fn post_query(&self, sql: &str, body_suffix: Option<String>) -> XResult<String> {
        self.ensure_open()?;
        let cfg = &self.inner.config;
        let mut url = url::Url::parse(&cfg.base_url())
            .map_err(|e| XError::invalid(format!("clickhouse base url: {e}")))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("database", &cfg.database);
        }

        let body = match body_suffix {
            Some(extra) => format!("{sql}\n{extra}"),
            None => sql.to_string(),
        };

        debug!(target: "clickhousex", database = %cfg.database, "clickhouse query");

        let resp = self
            .inner
            .http
            .post(url)
            .basic_auth(&cfg.user, Some(&cfg.password))
            .header(reqwest::header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    XError::deadline_exceeded(format!("clickhouse 超时: {e}"))
                } else {
                    XError::unavailable(format!("clickhouse 请求失败: {e}"))
                }
            })?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| XError::unavailable(format!("clickhouse 读响应失败: {e}")))?;

        if !status.is_success() {
            return Err(map_http_error(status, &text));
        }
        // ClickHouse 在 200 时也可能把异常写在 body（少见）；保留原文给调用方。
        Ok(text)
    }
}

#[async_trait]
impl AnalyticsSink for ClickHousePool {
    async fn sink(&self, event: &str, payload: Bytes) -> XResult<()> {
        if event.is_empty() {
            return Err(XError::invalid("analytics event 不能为空"));
        }
        self.ensure_analytics_table().await?;
        let row = serde_json::json!({
            "event": event,
            "payload": String::from_utf8_lossy(&payload),
        });
        self.insert_json_each_row(ANALYTICS_TABLE, &[row]).await
    }
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

fn map_http_error(status: StatusCode, body: &str) -> XError {
    let snippet = truncate(body, 512);
    if status == StatusCode::NOT_FOUND {
        return XError::missing(format!("clickhouse: {snippet}"));
    }
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return XError::unavailable(format!("clickhouse 认证/授权失败: {snippet}"));
    }
    if status.is_server_error() {
        return XError::transient(format!("clickhouse {status}: {snippet}"));
    }
    // 4xx 多半是 SQL/参数问题
    if snippet.contains("UNKNOWN_TABLE") || snippet.contains("doesn't exist") {
        return XError::missing(snippet);
    }
    XError::invalid(format!("clickhouse {status}: {snippet}"))
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
    fn ident_validation() {
        assert!(validate_ident("infra_draft_smoke").is_ok());
        assert!(validate_ident("1bad").is_err());
        assert!(validate_ident("a;drop").is_err());
        assert!(validate_ident("").is_err());
    }

    #[tokio::test]
    async fn connect_refused_fails_on_ping_path() {
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: 1,
            timeout: Duration::from_millis(300),
            ..ClickHouseConfig::default()
        };
        match ClickHousePool::connect(cfg).await {
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
