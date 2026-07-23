//! ClickHouse HTTP 生产客户端（8123）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::AnalyticsSink;
use kernel::{XError, XResult};
use reqwest::StatusCode;
use serde_json::Value;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::debug;

use crate::config::ClickHouseConfig;

/// 默认 analytics sink 表名。
pub const ANALYTICS_TABLE: &str = "analytics_events";

const ERROR_RESPONSE_CAPTURE_LIMIT: usize = 4096;

/// 批量插入选项。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchInsertOptions {
    /// 每个 HTTP 请求最大行数（≥1；0 会被抬升为 1）。
    pub max_rows_per_chunk: usize,
}

impl Default for BatchInsertOptions {
    fn default() -> Self {
        Self { max_rows_per_chunk: 1000 }
    }
}

/// 池运行时快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickHousePoolStats {
    /// 正在执行的请求数。
    pub in_flight: usize,
    /// 是否已关闭。
    pub closed: bool,
}

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
    sem: Arc<Semaphore>,
    in_flight: AtomicUsize,
    closed: AtomicBool,
}

/// 池上的工作句柄（与 [`ClickHousePool`] 等价，便于命名区分）。
pub type ClickHouseClient = ClickHousePool;

/// 解析 ClickHouse 默认 `TabSeparated` 查询文本为行列。
///
/// 跳过空行；按 tab 分列。纯函数，供 `query_rows` 与单测共用，避免测试侧重实现。
#[must_use]
pub fn parse_tab_separated_rows(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        rows.push(line.split('\t').map(str::to_string).collect());
    }
    rows
}

/// 计算分块范围：`(start, end)` 半开区间。
///
/// 纯函数，供单测驱动具体 chunk 尺寸。
#[must_use]
pub fn chunk_ranges(total: usize, max_per_chunk: usize) -> Vec<(usize, usize)> {
    if total == 0 {
        return Vec::new();
    }
    let size = max_per_chunk.max(1);
    let mut out = Vec::new();
    let mut start = 0;
    while start < total {
        let end = (start + size).min(total);
        out.push((start, end));
        start = end;
    }
    out
}

impl ClickHousePool {
    /// 使用配置建立 HTTP 客户端并 `ping`。
    pub async fn connect(config: ClickHouseConfig) -> XResult<Self> {
        config.validate()?;
        let build_config = config.clone();
        let http =
            tokio::task::spawn_blocking(move || build_http_client(&build_config)).await.map_err(
                |error| XError::internal("clickhouse HTTP 客户端构建任务失败").with_source(error),
            )??;
        let max_in_flight = config.max_in_flight;
        let pool = Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                sem: Arc::new(Semaphore::new(max_in_flight)),
                in_flight: AtomicUsize::new(0),
                closed: AtomicBool::new(false),
            }),
        };
        pool.ping().await?;
        Ok(pool)
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(ClickHouseConfig::from_env()?).await
    }

    /// 供集成测试使用：跳过 ping，便于离线验证 close / stats / acquire。
    pub fn connect_without_ping(config: ClickHouseConfig) -> XResult<Self> {
        config.validate()?;
        let http = build_http_client(&config)?;
        let max_in_flight = config.max_in_flight;
        Ok(Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                sem: Arc::new(Semaphore::new(max_in_flight)),
                in_flight: AtomicUsize::new(0),
                closed: AtomicBool::new(false),
            }),
        })
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

    /// 当前统计。
    #[must_use]
    pub fn stats(&self) -> ClickHousePoolStats {
        ClickHousePoolStats {
            in_flight: self.inner.in_flight.load(Ordering::Relaxed),
            closed: self.inner.closed.load(Ordering::Relaxed),
        }
    }

    /// 是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    /// `SELECT 1` 健康检查。
    pub async fn ping(&self) -> XResult<()> {
        let body = self.query_text("SELECT 1").await?;
        if body.trim() != "1" {
            return Err(XError::unavailable("clickhouse ping 响应不符合协议（响应正文已省略）"));
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
        Ok(parse_tab_separated_rows(&text))
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

    /// 分块批量插入：按 `max_rows_per_chunk` 切分后调用 `insert_json_each_row`。
    ///
    /// 空 `rows` → `Ok(())`。
    pub async fn insert_batch(
        &self,
        table: &str,
        rows: &[Value],
        options: BatchInsertOptions,
    ) -> XResult<()> {
        validate_ident(table)?;
        if rows.is_empty() {
            return Ok(());
        }
        let ranges = chunk_ranges(rows.len(), options.max_rows_per_chunk);
        for (start, end) in ranges {
            self.insert_json_each_row(table, &rows[start..end]).await?;
        }
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
        // 关闭信号量，使后续 acquire 失败
        self.inner.sem.close();
        Ok(())
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(XError::unavailable("clickhouse pool 已关闭"));
        }
        Ok(())
    }

    async fn acquire(&self) -> XResult<OwnedSemaphorePermit> {
        self.ensure_open()?;
        let result =
            timeout(self.inner.config.acquire_timeout, self.inner.sem.clone().acquire_owned())
                .await;
        match result {
            Ok(Ok(permit)) => {
                if self.is_closed() {
                    drop(permit);
                    return Err(XError::unavailable("clickhouse pool 已关闭"));
                }
                Ok(permit)
            }
            Ok(Err(_)) => Err(XError::unavailable("clickhouse 背压信号量已关闭")),
            Err(_) => Err(XError::deadline_exceeded(format!(
                "clickhouse 获取 in-flight 许可超时（max={}）",
                self.inner.config.max_in_flight
            ))),
        }
    }

    async fn post_query(&self, sql: &str, body_suffix: Option<String>) -> XResult<String> {
        let _permit = self.acquire().await?;
        self.inner.in_flight.fetch_add(1, Ordering::SeqCst);
        let result = self.post_query_inner(sql, body_suffix).await;
        self.inner.in_flight.fetch_sub(1, Ordering::SeqCst);
        result
    }

    async fn post_query_inner(&self, sql: &str, body_suffix: Option<String>) -> XResult<String> {
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
                    XError::deadline_exceeded("clickhouse 请求超时").with_source(e)
                } else {
                    XError::unavailable("clickhouse 请求失败").with_source(e)
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let error_prefix = read_error_prefix(resp).await?;
            return Err(map_http_error(status, &error_prefix));
        }
        let text = resp
            .text()
            .await
            .map_err(|error| XError::unavailable("clickhouse 读响应失败").with_source(error))?;
        // ClickHouse 在 200 时也可能把异常写在 body（少见）；保留原文给调用方。
        Ok(text)
    }
}

fn build_http_client(config: &ClickHouseConfig) -> XResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(config.timeout)
        .pool_max_idle_per_host(config.max_idle_per_host);
    if let Some(path) = &config.tls_ca_file {
        let pem = std::fs::read(path).map_err(|error| {
            XError::invalid(format!("clickhouse 无法读取 TLS CA `{}`", path.display()))
                .with_source(error)
        })?;
        let certificate = reqwest::Certificate::from_pem(&pem).map_err(|error| {
            XError::invalid("clickhouse TLS CA 不是合法 PEM").with_source(error)
        })?;
        builder = builder.add_root_certificate(certificate);
    }
    builder
        .build()
        .map_err(|error| XError::internal("clickhouse HTTP 客户端构建失败").with_source(error))
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

pub fn validate_ident(name: &str) -> XResult<()> {
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

fn map_http_error(status: StatusCode, body: &[u8]) -> XError {
    let server_code = clickhouse_server_code(body);
    let context = safe_http_error_context(status, server_code);
    if status == StatusCode::NOT_FOUND {
        return XError::missing(context);
    }
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return XError::unavailable(format!("clickhouse 认证/授权失败；{context}"));
    }
    if status.is_server_error() {
        return XError::transient(context);
    }
    match server_code {
        // UNKNOWN_TABLE / UNKNOWN_DATABASE
        Some(60 | 81) => XError::missing(context),
        // TABLE_ALREADY_EXISTS
        Some(57) => XError::conflict(context),
        // 其余 4xx 多半是 SQL 或参数问题；响应正文始终不进入错误。
        _ => XError::invalid(context),
    }
}

async fn read_error_prefix(mut response: reqwest::Response) -> XResult<Vec<u8>> {
    let mut prefix = Vec::with_capacity(ERROR_RESPONSE_CAPTURE_LIMIT);
    while prefix.len() < ERROR_RESPONSE_CAPTURE_LIMIT {
        let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| XError::unavailable("clickhouse 读错误响应失败").with_source(error))?
        else {
            break;
        };
        let remaining = ERROR_RESPONSE_CAPTURE_LIMIT - prefix.len();
        prefix.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        if chunk.len() >= remaining {
            break;
        }
    }
    Ok(prefix)
}

fn clickhouse_server_code(body: &[u8]) -> Option<u32> {
    let body = std::str::from_utf8(body).ok()?.trim_start();
    let rest = body.strip_prefix("Code:")?.trim_start();
    let digits = rest.chars().take_while(char::is_ascii_digit).collect::<String>();
    if digits.is_empty() { None } else { digits.parse().ok() }
}

fn safe_http_error_context(status: StatusCode, server_code: Option<u32>) -> String {
    match server_code {
        Some(code) => {
            format!("clickhouse HTTP {status}（server_code={code}，响应正文已省略）")
        }
        None => format!("clickhouse HTTP {status}（响应正文已省略）"),
    }
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

    #[test]
    fn chunk_ranges_concrete_sizes() {
        assert!(chunk_ranges(0, 10).is_empty());
        assert_eq!(chunk_ranges(5, 10), vec![(0, 5)]);
        assert_eq!(chunk_ranges(5, 2), vec![(0, 2), (2, 4), (4, 5)]);
        assert_eq!(chunk_ranges(3, 1), vec![(0, 1), (1, 2), (2, 3)]);
        // max_per_chunk=0 → 抬升为 1
        assert_eq!(chunk_ranges(2, 0), vec![(0, 1), (1, 2)]);
        assert_eq!(chunk_ranges(7, 3), vec![(0, 3), (3, 6), (6, 7)]);
    }

    #[test]
    fn chunk_ranges_property_coverage_and_continuity() {
        let cases: &[(usize, usize)] = &[
            (0, 1),
            (1, 1),
            (1, 5),
            (5, 2),
            (6, 2),
            (10, 3),
            (100, 7),
            (100, 1),
            (5, 100),
            (7, 0),
            (1000, 197),
        ];
        for &(total, max_per_chunk) in cases {
            let chunks = chunk_ranges(total, max_per_chunk);
            let effective_max = max_per_chunk.max(1);

            if total == 0 {
                assert!(chunks.is_empty(), "total=0: expected empty, got {chunks:?}");
                continue;
            }

            assert!(!chunks.is_empty(), "total={total} max={max_per_chunk}: expected non-empty");

            // 覆盖性：最后一个 chunk 的 end == total
            assert_eq!(
                chunks.last().unwrap().1,
                total,
                "total={total} max={max_per_chunk}: last chunk end should be {total}, chunks={chunks:?}"
            );

            // 连续性：前一个 chunk 的 end == 下一个 chunk 的 start
            for pair in chunks.windows(2) {
                assert_eq!(
                    pair[0].1, pair[1].0,
                    "total={total} max={max_per_chunk}: gap between {:?} and {:?}",
                    pair[0], pair[1]
                );
            }

            // 单调性：每个 chunk 的 start < end
            for (start, end) in &chunks {
                assert!(
                    start < end,
                    "total={total} max={max_per_chunk}: invalid chunk ({start},{end})"
                );
            }

            // 上界约束：每个 chunk 大小 <= effective_max
            for (start, end) in &chunks {
                let size = end - start;
                assert!(
                    size <= effective_max,
                    "total={total} max={max_per_chunk}: chunk ({start},{end}) size {size} exceeds limit {effective_max}"
                );
            }

            // 覆盖性（前向）：第一个 chunk 的 start == 0
            assert_eq!(
                chunks[0].0, 0,
                "total={total} max={max_per_chunk}: first chunk should start at 0"
            );
        }
    }

    #[test]
    fn batch_options_default() {
        let o = BatchInsertOptions::default();
        assert_eq!(o.max_rows_per_chunk, 1000);
    }

    #[test]
    fn http_error_mapping_uses_code_without_echoing_response() {
        let secret = "SELECT private_column; payload=secret-value";
        let body = format!("Code: 60. DB::Exception: UNKNOWN_TABLE; {secret}");
        let error = map_http_error(StatusCode::BAD_REQUEST, body.as_bytes());
        assert_eq!(error.kind(), ErrorKind::Missing);
        assert!(error.context().contains("server_code=60"));
        assert!(!error.to_string().contains(secret));

        let auth = map_http_error(StatusCode::UNAUTHORIZED, secret.as_bytes());
        assert_eq!(auth.kind(), ErrorKind::Unavailable);
        assert!(!auth.to_string().contains(secret));
    }

    #[test]
    fn server_code_parser_is_bounded_to_the_prefix() {
        assert_eq!(clickhouse_server_code(b"Code: 81. DB::Exception"), Some(81));
        assert_eq!(clickhouse_server_code(b"not a ClickHouse exception"), None);
        assert_eq!(clickhouse_server_code(&[0xff, 0xfe]), None);
    }

    #[tokio::test]
    async fn connect_refused_fails_on_ping_path() {
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: 1,
            timeout: Duration::from_millis(300),
            acquire_timeout: Duration::from_millis(300),
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

    #[tokio::test]
    async fn connect_rejects_zero_max_in_flight() {
        let cfg = ClickHouseConfig {
            max_in_flight: 0,
            host: "127.0.0.1".into(),
            http_port: 1,
            timeout: Duration::from_millis(100),
            ..ClickHouseConfig::default()
        };
        let err = match ClickHousePool::connect(cfg).await {
            Ok(_) => panic!("must reject zero max_in_flight"),
            Err(e) => e,
        };
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn closed_pool_rejects_and_stats() {
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: 1,
            timeout: Duration::from_millis(200),
            acquire_timeout: Duration::from_millis(200),
            max_in_flight: 2,
            ..ClickHouseConfig::default()
        };
        let pool = ClickHousePool::connect_without_ping(cfg).expect("build");
        assert_eq!(pool.stats().in_flight, 0);
        assert!(!pool.stats().closed);

        pool.close().await.expect("close");
        assert!(pool.stats().closed);
        assert!(pool.is_closed());

        let err = pool.execute("SELECT 1").await.expect_err("closed");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
    }

    #[tokio::test]
    async fn idle_per_host_applied_on_builder() {
        let cfg = ClickHouseConfig {
            max_idle_per_host: 3,
            max_in_flight: 4,
            host: "127.0.0.1".into(),
            http_port: 1,
            ..ClickHouseConfig::default()
        };
        let pool = ClickHousePool::connect_without_ping(cfg).expect("build");
        assert_eq!(pool.config().max_idle_per_host, 3);
        assert_eq!(pool.config().max_in_flight, 4);
    }

    fn local_pool(max_in_flight: usize) -> ClickHousePool {
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: 1,
            timeout: Duration::from_millis(200),
            acquire_timeout: Duration::from_millis(200),
            max_in_flight,
            ..ClickHouseConfig::default()
        };
        ClickHousePool::connect_without_ping(cfg).expect("build")
    }

    // R1：insert_json_each_row / insert_batch 的标识符校验必须在发出任何 HTTP
    // 请求之前 fail-closed；用未 ping 的 pool（http_port=1 必然连接失败）证明
    // 校验错误优先于网络错误返回。
    #[tokio::test]
    async fn insert_json_each_row_rejects_invalid_table_before_network() {
        let pool = local_pool(1);
        let err = pool
            .insert_json_each_row("1bad", &[serde_json::json!({"a": 1})])
            .await
            .expect_err("非法表名必须拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn insert_json_each_row_rejects_non_object_row() {
        let pool = local_pool(1);
        let err = pool
            .insert_json_each_row("valid_table", &[serde_json::json!(["not", "an", "object"])])
            .await
            .expect_err("非 object 行必须拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
        assert!(err.context().contains("object"));
    }

    #[tokio::test]
    async fn insert_json_each_row_empty_rows_short_circuits_without_network() {
        // http_port=1 若真的发起网络请求会失败；空 rows 必须在此之前返回 Ok。
        let pool = local_pool(1);
        pool.insert_json_each_row("valid_table", &[]).await.expect("空 rows 必须直接成功");
    }

    #[tokio::test]
    async fn insert_batch_rejects_invalid_table_before_chunking() {
        let pool = local_pool(1);
        let err = pool
            .insert_batch("a;drop", &[serde_json::json!({"a": 1})], BatchInsertOptions::default())
            .await
            .expect_err("非法表名必须在分块前拒绝");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn insert_batch_empty_rows_short_circuits_without_network() {
        let pool = local_pool(1);
        pool.insert_batch("valid_table", &[], BatchInsertOptions::default())
            .await
            .expect("空 rows 必须直接成功");
    }

    // 驱动真实 `parse_tab_separated_rows`（query_rows 生产路径共用），禁止测试侧重实现。
    #[test]
    fn parse_tab_separated_rows_skips_blank_lines_and_splits_tabs() {
        let rows = parse_tab_separated_rows("a\tb\n\nc\td\te\n");
        assert_eq!(
            rows,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string(), "e".to_string()]
            ]
        );
        assert!(parse_tab_separated_rows("").is_empty());
        assert!(parse_tab_separated_rows("\n\n\n").is_empty());
    }

    #[test]
    fn map_http_error_covers_not_found_conflict_and_server_error_branches() {
        // 404 → Missing（无 server_code）
        let not_found = map_http_error(StatusCode::NOT_FOUND, b"");
        assert_eq!(not_found.kind(), ErrorKind::Missing);

        // TABLE_ALREADY_EXISTS（57）→ Conflict
        let conflict =
            map_http_error(StatusCode::BAD_REQUEST, b"Code: 57. DB::Exception: already exists");
        assert_eq!(conflict.kind(), ErrorKind::Conflict);

        // UNKNOWN_DATABASE（81）→ Missing
        let unknown_db =
            map_http_error(StatusCode::BAD_REQUEST, b"Code: 81. DB::Exception: unknown db");
        assert_eq!(unknown_db.kind(), ErrorKind::Missing);

        // 5xx → Transient（服务端错误应可重试）
        let server_error = map_http_error(StatusCode::INTERNAL_SERVER_ERROR, b"boom");
        assert_eq!(server_error.kind(), ErrorKind::Transient);

        let unavailable = map_http_error(StatusCode::SERVICE_UNAVAILABLE, b"");
        assert_eq!(unavailable.kind(), ErrorKind::Transient);

        // FORBIDDEN 与 UNAUTHORIZED 同归 Unavailable（认证/授权失败）
        let forbidden = map_http_error(StatusCode::FORBIDDEN, b"denied");
        assert_eq!(forbidden.kind(), ErrorKind::Unavailable);

        // 未知 4xx 且无匹配 server_code → Invalid（SQL/参数问题的默认归类）
        let other_client_error = map_http_error(StatusCode::BAD_REQUEST, b"Code: 999. unknown");
        assert_eq!(other_client_error.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn read_error_prefix_is_bounded_to_capture_limit() {
        // 构造超过 ERROR_RESPONSE_CAPTURE_LIMIT（4096）的响应体，证明
        // read_error_prefix 截断在边界处而不会无界读取或丢失边界内数据。
        let oversized_marker = "Code: 60. DB::Exception: ";
        let filler = "x".repeat(ERROR_RESPONSE_CAPTURE_LIMIT * 2);
        let body = format!("{oversized_marker}{filler}");
        assert!(body.len() > ERROR_RESPONSE_CAPTURE_LIMIT);

        let (port, server) = spawn_one_response("400 Bad Request", body.clone()).await;
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: port,
            timeout: Duration::from_secs(2),
            acquire_timeout: Duration::from_secs(2),
            ..ClickHouseConfig::default()
        };
        let error = match ClickHousePool::connect(cfg).await {
            Err(error) => error,
            Ok(_) => panic!("超限响应必须仍被拒绝"),
        };
        // 错误上下文只保留状态码与 server_code，超限正文永不进入错误。
        assert!(!error.to_string().contains(&filler));
        assert!(error.context().contains("server_code=60"));
        server.await.expect("HTTP server task");
    }

    async fn spawn_one_response(status: &str, body: String) -> (u16, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
        let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
        let status = status.to_owned();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("接受 HTTP 连接");
            let mut request = vec![0u8; 16 * 1024];
            let _ = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut request))
                .await
                .expect("读取请求不得超时")
                .expect("读取 HTTP 请求");
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.expect("写 HTTP 响应");
            stream.shutdown().await.expect("关闭 HTTP 流");
        });
        (port, server)
    }

    // ── R2：对抗验证 / 边界回归 ──────────────────────────────────

    /// 起一个会挂起 `hold` 时长才回应的一次性 HTTP 服务：用于占住唯一的
    /// in-flight 许可，逼迫第二个并发请求走 `acquire_timeout` 分支。
    async fn spawn_slow_response(hold: Duration) -> (u16, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
        let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("接受 HTTP 连接");
            let mut request = vec![0u8; 16 * 1024];
            let _ = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut request))
                .await
                .expect("读取请求不得超时")
                .expect("读取 HTTP 请求");
            tokio::time::sleep(hold).await;
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\n1\n",
                )
                .await
                .expect("写 HTTP 响应");
            stream.shutdown().await.expect("关闭 HTTP 流");
        });
        (port, server)
    }

    /// 背压边界：`max_in_flight=1` 时，第一个请求占住唯一许可并长时间挂起，
    /// 第二个并发请求必须在 `acquire_timeout` 后收到 `DeadlineExceeded`，
    /// 而不是无限等待或被静默丢弃。
    #[tokio::test]
    async fn second_request_times_out_waiting_for_the_only_permit() {
        let hold = Duration::from_secs(3);
        let (port, server) = spawn_slow_response(hold).await;
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: port,
            timeout: Duration::from_secs(10),
            acquire_timeout: Duration::from_millis(200),
            max_in_flight: 1,
            ..ClickHouseConfig::default()
        };
        let pool = ClickHousePool::connect_without_ping(cfg).expect("build");

        let first = {
            let pool = pool.clone();
            tokio::spawn(async move { pool.query_text("SELECT 1").await })
        };
        // 给第一个请求足够时间先拿到唯一许可，再发第二个请求。
        tokio::time::sleep(Duration::from_millis(50)).await;

        let second_err = pool.query_text("SELECT 1").await.expect_err("第二个请求必须超时");
        assert_eq!(second_err.kind(), ErrorKind::DeadlineExceeded);
        assert!(second_err.context().contains("max=1"));

        let first_result = first.await.expect("first task join");
        assert!(first_result.is_ok(), "第一个请求应最终成功: {first_result:?}");
        server.await.expect("HTTP server task");
    }

    /// 起一个记录累计收到多少个独立 HTTP 请求的服务端；每个请求均回应
    /// `200 OK` 的最小 body，用于验证 `insert_batch` 真正发出多次独立
    /// POST（而非把所有 chunk 拼进一次请求）。
    async fn spawn_counting_server(
        expected_requests: usize,
    ) -> (u16, tokio::task::JoinHandle<usize>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
        let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
        let server = tokio::spawn(async move {
            let mut count = 0usize;
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().await.expect("接受 HTTP 连接");
                let mut request = Vec::with_capacity(4096);
                loop {
                    let mut chunk = [0_u8; 1024];
                    let read =
                        tokio::time::timeout(Duration::from_secs(5), stream.read(&mut chunk))
                            .await
                            .expect("读取请求不得超时")
                            .expect("读取 HTTP 请求");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                    let headers = String::from_utf8_lossy(&request);
                    let Some(header_end) = headers.find("\r\n\r\n") else { continue };
                    let content_length = headers[..header_end]
                        .lines()
                        .find_map(|line| {
                            line.strip_prefix("content-length:")
                                .or_else(|| line.strip_prefix("Content-Length:"))
                        })
                        .and_then(|value| value.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    if request.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
                count += 1;
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 1\r\nConnection: close\r\n\r\n\n",
                    )
                    .await
                    .expect("写 HTTP 响应");
                stream.shutdown().await.expect("关闭 HTTP 流");
            }
            count
        });
        (port, server)
    }

    /// `insert_batch` 按 `max_rows_per_chunk` 分块后必须对每个 chunk 发出
    /// 一次独立的 HTTP POST；5 行、每 chunk 2 行应产生 3 次请求（2+2+1）。
    #[tokio::test]
    async fn insert_batch_sends_one_http_request_per_chunk() {
        let (port, server) = spawn_counting_server(3).await;
        let cfg = ClickHouseConfig {
            host: "127.0.0.1".into(),
            http_port: port,
            timeout: Duration::from_secs(5),
            acquire_timeout: Duration::from_secs(5),
            ..ClickHouseConfig::default()
        };
        let pool = ClickHousePool::connect_without_ping(cfg).expect("build");

        let rows: Vec<Value> = (0..5).map(|i| serde_json::json!({"n": i})).collect();
        pool.insert_batch("valid_table", &rows, BatchInsertOptions { max_rows_per_chunk: 2 })
            .await
            .expect("分块插入必须成功");

        let observed_requests = server.await.expect("counting server task");
        assert_eq!(observed_requests, 3, "5 行按每块 2 行应产生 3 次独立请求");
    }

    /// 属性验证：对于多组 (total, max_per_chunk) 参数，`insert_batch` 的 HTTP
    /// 请求次数与 `chunk_ranges` 返回的 chunk 数量一致，且等于 ceil(total/max)。
    #[tokio::test]
    async fn insert_batch_request_count_matches_chunk_ranges() {
        // 使用不限流的 server：所有 chunk 依次处理直到 timeout 关闭。
        // 每个 case 独立创建一个 pool+server，避免状态污染。
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        async fn serve_until_idle(listener: TcpListener, expected: usize) -> usize {
            let mut count = 0usize;
            for _ in 0..expected {
                match tokio::time::timeout(Duration::from_secs(3), listener.accept()).await {
                    Ok(Ok((mut stream, _))) => {
                        let mut buf = [0u8; 4096];
                        let _ = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf))
                            .await;
                        stream
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 1\r\nConnection: close\r\n\r\n\n")
                            .await
                            .ok();
                        stream.shutdown().await.ok();
                        count += 1;
                    }
                    _ => break,
                }
            }
            count
        }

        let cases: &[(usize, usize)] =
            &[(3, 1), (5, 2), (10, 3), (1, 1), (10, 10), (10, 100), (7, 0)];

        for &(total, max_per_chunk) in cases {
            let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
            let port = listener.local_addr().expect("addr").port();
            let expected = chunk_ranges(total, max_per_chunk).len();
            let server = tokio::spawn(serve_until_idle(listener, expected));

            let cfg = ClickHouseConfig {
                host: "127.0.0.1".into(),
                http_port: port,
                timeout: Duration::from_secs(10),
                acquire_timeout: Duration::from_secs(5),
                max_in_flight: 64,
                ..ClickHouseConfig::default()
            };
            let pool = ClickHousePool::connect_without_ping(cfg).expect("build");

            let rows: Vec<Value> = (0..total).map(|i| serde_json::json!({"n": i})).collect();
            pool.insert_batch(
                "valid_table",
                &rows,
                BatchInsertOptions { max_rows_per_chunk: max_per_chunk },
            )
            .await
            .expect("insert_batch should succeed");

            let observed = server.await.expect("server join");
            assert_eq!(
                observed, expected,
                "total={total} max_per_chunk={max_per_chunk}: expected {expected} HTTP requests, got {observed}"
            );
        }
    }
}
