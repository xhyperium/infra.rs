//! TDengine REST / Native WS 连接配置。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 单进程允许的最大并发请求数。
pub const HARD_MAX_IN_FLIGHT: usize = 1_024;
/// 单条 SQL 请求允许的最大 UTF-8 字节数。
pub const HARD_MAX_BATCH_BYTES: usize = 8 * 1024 * 1024;
/// 单个批次允许的最大行数。
pub const HARD_MAX_BATCH_ROWS: usize = 10_000;
/// 单个 REST 响应允许的最大字节数。
pub const HARD_MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
/// 单次查询允许返回的最大行数。
pub const HARD_MAX_QUERY_ROWS: usize = 100_000;
/// 关闭排空允许配置的最长时间。
pub const HARD_MAX_CLOSE_TIMEOUT: Duration = Duration::from_secs(30);

const INVALID_ENV_HOST: &str = "__invalid_taos_env__";

/// 时间戳精度（库级；`Tick.ts` 始终为纳秒，写入前按精度换算）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TsPrecision {
    /// 毫秒（TDengine 默认）。
    #[default]
    Ms,
    /// 微秒。
    Us,
    /// 纳秒。
    Ns,
}

impl TsPrecision {
    /// 从 TDengine 返回值解析。
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ms" => Some(Self::Ms),
            "us" => Some(Self::Us),
            "ns" => Some(Self::Ns),
            _ => None,
        }
    }

    /// 纳秒 → 库时间戳数值。
    #[must_use]
    pub fn from_nanos(self, ts_ns: i64) -> i64 {
        match self {
            Self::Ns => ts_ns,
            Self::Us => ts_ns / 1_000,
            Self::Ms => ts_ns / 1_000_000,
        }
    }

    /// 库时间戳数值 → 纳秒。
    #[must_use]
    pub fn to_nanos(self, ts: i64) -> i64 {
        match self {
            Self::Ns => ts,
            Self::Us => ts.saturating_mul(1_000),
            Self::Ms => ts.saturating_mul(1_000_000),
        }
    }
}

/// 传输模式：REST（默认）或原生 WebSocket。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportMode {
    /// HTTP REST（`/rest/sql`）。
    #[default]
    Rest,
    /// 原生 WebSocket（`/rest/ws`）。
    NativeWs,
}

impl TransportMode {
    /// 从字符串解析（`rest` / `native` / `ws` / `native_ws`）。
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "rest" | "http" => Some(Self::Rest),
            "native" | "ws" | "native_ws" | "native-ws" => Some(Self::NativeWs),
            _ => None,
        }
    }
}

/// TDengine 客户端配置。
///
/// 环境变量前缀：`FOUNDATIONX_TAOSX_`
/// - `HOST`（默认 `127.0.0.1`）
/// - `PORT`（默认 `6041` REST）
/// - `DATABASE`（默认 `infra_draft`）
/// - `USER`（默认 `root`）
/// - `PASSWORD`（默认空；**不**写入仓库）
/// - `TLS`（`1`/`true`/`yes` 启用 https/wss）
/// - `TIMEOUT_MS`（默认 `10000`）
/// - `PRECISION`（可选 `ms`/`us`/`ns`；未设则连接后探测）
/// - `TRANSPORT`（`rest` / `native`；默认 `rest`）
/// - `MAX_IN_FLIGHT`（默认 `64`）
/// - `ACQUIRE_TIMEOUT_MS`（默认 `5000`）
/// - `BATCH_MAX_ROWS`（默认 `500`）
/// - `BATCH_MAX_BYTES`（默认 `1048576`）
/// - `MAX_RESPONSE_BYTES`（默认 `8388608`）
/// - `MAX_QUERY_ROWS`（默认 `10000`）
/// - `CLOSE_TIMEOUT_MS`（默认 `5000`）
#[derive(Clone)]
pub struct TaosConfig {
    /// 主机。
    pub host: String,
    /// REST / WS 端口。
    pub port: u16,
    /// 数据库名。
    pub database: String,
    /// 用户。
    pub user: String,
    /// 密码（`Debug` 脱敏）。
    pub password: String,
    /// 是否 HTTPS / WSS。
    pub tls: bool,
    /// 请求超时。
    pub timeout: Duration,
    /// 可选显式精度；`None` 时在 `connect` 后探测。
    pub precision: Option<TsPrecision>,
    /// 传输模式。
    pub transport: TransportMode,
    /// 全局 in-flight 上限（≥1）。
    pub max_in_flight: usize,
    /// 获取 in-flight 许可超时。
    pub acquire_timeout: Duration,
    /// `write_batch` 默认每批最大行数。
    pub batch_max_rows: usize,
    /// 单条 SQL 请求最大 UTF-8 字节数。
    pub batch_max_bytes: usize,
    /// REST 响应体最大字节数。
    pub max_response_bytes: usize,
    /// 单次查询最大结果行数。
    pub max_query_rows: usize,
    /// 关闭时等待在途请求排空的 deadline。
    pub close_timeout: Duration,
}

impl fmt::Debug for TaosConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaosConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"***")
            .field("tls", &self.tls)
            .field("timeout", &self.timeout)
            .field("precision", &self.precision)
            .field("transport", &self.transport)
            .field("max_in_flight", &self.max_in_flight)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("batch_max_rows", &self.batch_max_rows)
            .field("batch_max_bytes", &self.batch_max_bytes)
            .field("max_response_bytes", &self.max_response_bytes)
            .field("max_query_rows", &self.max_query_rows)
            .field("close_timeout", &self.close_timeout)
            .finish()
    }
}

impl Default for TaosConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6041,
            database: "infra_draft".into(),
            user: "root".into(),
            password: String::new(),
            tls: false,
            timeout: Duration::from_secs(10),
            precision: None,
            transport: TransportMode::Rest,
            max_in_flight: 64,
            acquire_timeout: Duration::from_secs(5),
            batch_max_rows: 500,
            batch_max_bytes: 1024 * 1024,
            max_response_bytes: 8 * 1024 * 1024,
            max_query_rows: 10_000,
            close_timeout: Duration::from_secs(5),
        }
    }
}

impl TaosConfig {
    /// 从环境变量加载。
    #[must_use]
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_HOST") {
            cfg.host = if v.trim().is_empty() { INVALID_ENV_HOST.into() } else { v };
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PORT") {
            cfg.port = v.parse().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_DATABASE") {
            if !v.is_empty() {
                cfg.database = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_USER") {
            cfg.user = v;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PASSWORD") {
            cfg.password = v;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TLS") {
            match parse_bool(&v) {
                Some(value) => cfg.tls = value,
                None => cfg.host = INVALID_ENV_HOST.into(),
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TIMEOUT_MS") {
            cfg.timeout = Duration::from_millis(v.parse::<u64>().unwrap_or(0));
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PRECISION") {
            cfg.precision = TsPrecision::parse(&v);
            if cfg.precision.is_none() {
                cfg.host = INVALID_ENV_HOST.into();
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TRANSPORT") {
            if !v.trim().is_empty() {
                match TransportMode::parse(&v) {
                    Some(m) => cfg.transport = m,
                    None => cfg.host = INVALID_ENV_HOST.into(),
                }
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_MAX_IN_FLIGHT") {
            cfg.max_in_flight = v.parse::<usize>().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_ACQUIRE_TIMEOUT_MS") {
            cfg.acquire_timeout = Duration::from_millis(v.parse::<u64>().unwrap_or(0));
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_BATCH_MAX_ROWS") {
            cfg.batch_max_rows = v.parse::<usize>().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_BATCH_MAX_BYTES") {
            cfg.batch_max_bytes = v.parse::<usize>().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_MAX_RESPONSE_BYTES") {
            cfg.max_response_bytes = v.parse::<usize>().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_MAX_QUERY_ROWS") {
            cfg.max_query_rows = v.parse::<usize>().unwrap_or(0);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_CLOSE_TIMEOUT_MS") {
            cfg.close_timeout = Duration::from_millis(v.parse::<u64>().unwrap_or(0));
        }
        cfg
    }

    /// 校验约束。
    pub fn validate(&self) -> XResult<()> {
        if self.host == INVALID_ENV_HOST {
            return Err(XError::invalid("FOUNDATIONX_TAOSX_* 含空白或非法值"));
        }
        if self.max_in_flight < 1 || self.max_in_flight > HARD_MAX_IN_FLIGHT {
            return Err(XError::invalid(format!("max_in_flight 必须为 1..={HARD_MAX_IN_FLIGHT}")));
        }
        if self.batch_max_rows < 1 || self.batch_max_rows > HARD_MAX_BATCH_ROWS {
            return Err(XError::invalid(format!(
                "batch_max_rows 必须为 1..={HARD_MAX_BATCH_ROWS}"
            )));
        }
        if self.batch_max_bytes < 1 || self.batch_max_bytes > HARD_MAX_BATCH_BYTES {
            return Err(XError::invalid(format!(
                "batch_max_bytes 必须为 1..={HARD_MAX_BATCH_BYTES}"
            )));
        }
        if self.max_response_bytes < 1 || self.max_response_bytes > HARD_MAX_RESPONSE_BYTES {
            return Err(XError::invalid(format!(
                "max_response_bytes 必须为 1..={HARD_MAX_RESPONSE_BYTES}"
            )));
        }
        if self.max_query_rows < 1 || self.max_query_rows > HARD_MAX_QUERY_ROWS {
            return Err(XError::invalid(format!(
                "max_query_rows 必须为 1..={HARD_MAX_QUERY_ROWS}"
            )));
        }
        if self.timeout.is_zero()
            || self.acquire_timeout.is_zero()
            || self.close_timeout.is_zero()
            || self.close_timeout > HARD_MAX_CLOSE_TIMEOUT
        {
            return Err(XError::invalid("taos timeout 必须大于零且 close_timeout 不超过 30 秒"));
        }
        if !valid_host(&self.host) || self.port == 0 {
            return Err(XError::invalid("taos host/port 非法"));
        }
        if !self.database.is_empty() && !valid_ident(&self.database) {
            return Err(XError::invalid("taos database 标识符非法"));
        }
        if self.user.trim().is_empty() {
            return Err(XError::invalid("taos user 不能为空"));
        }
        if !host_is_loopback(&self.host) {
            if !self.tls {
                return Err(XError::invalid("远程 TDengine 必须使用 TLS"));
            }
            if self.password.trim().is_empty() {
                return Err(XError::invalid("远程 TDengine 必须配置认证密码"));
            }
        }
        Ok(())
    }

    /// REST SQL 端点：`http(s)://host:port/rest/sql`。
    #[must_use]
    pub fn rest_sql_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{scheme}://{}:{}/rest/sql", url_host(&self.host), self.port)
    }

    /// 带 database 路径的 REST URL。
    #[must_use]
    pub fn rest_sql_db_url(&self) -> String {
        let base = self.rest_sql_url();
        if self.database.is_empty() { base } else { format!("{base}/{}", self.database) }
    }

    /// 原生 WebSocket SQL 端点：`ws(s)://host:port/rest/ws`。
    #[must_use]
    pub fn native_ws_url(&self) -> String {
        let scheme = if self.tls { "wss" } else { "ws" };
        format!("{scheme}://{}:{}/rest/ws", url_host(&self.host), self.port)
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.eq_ignore_ascii_case("localhost.")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn valid_host(host: &str) -> bool {
    let trimmed = host.trim();
    if trimmed.is_empty()
        || trimmed != host
        || trimmed.contains("//")
        || trimmed.contains('@')
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('?')
        || trimmed.contains('#')
        || trimmed.chars().any(char::is_whitespace)
    {
        return false;
    }
    let unbracketed =
        trimmed.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(trimmed);
    if unbracketed.contains(':') {
        return unbracketed.parse::<std::net::IpAddr>().is_ok();
    }
    unbracketed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}

fn valid_ident(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn url_host(host: &str) -> String {
    if host.starts_with('[') || !host.contains(':') {
        host.to_string()
    } else {
        format!("[{host}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precision_convert() {
        assert_eq!(TsPrecision::Ms.from_nanos(1_500_000_000), 1500);
        assert_eq!(TsPrecision::Ms.to_nanos(1500), 1_500_000_000);
        assert_eq!(TsPrecision::Ns.from_nanos(42), 42);
    }

    #[test]
    fn debug_redacts_password() {
        let c = TaosConfig { password: "s3cret".into(), ..Default::default() };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("s3cret"));
    }

    #[test]
    fn transport_mode_parse() {
        assert_eq!(TransportMode::parse("rest"), Some(TransportMode::Rest));
        assert_eq!(TransportMode::parse("native"), Some(TransportMode::NativeWs));
        assert_eq!(TransportMode::parse("ws"), Some(TransportMode::NativeWs));
        assert!(TransportMode::parse("bogus").is_none());
    }

    #[test]
    fn native_ws_url_builder() {
        let c = TaosConfig::default();
        assert_eq!(c.native_ws_url(), "ws://127.0.0.1:6041/rest/ws");
        let tls =
            TaosConfig { tls: true, host: "td.example".into(), port: 6041, ..Default::default() };
        assert_eq!(tls.native_ws_url(), "wss://td.example:6041/rest/ws");
    }

    #[test]
    fn validate_max_in_flight() {
        let c = TaosConfig { max_in_flight: 0, ..Default::default() };
        assert!(c.validate().is_err());
        TaosConfig::default().validate().expect("default ok");
    }

    #[test]
    fn remote_plaintext_and_auth_fail_closed() {
        let plain = TaosConfig { host: "td.example".into(), ..Default::default() };
        assert_eq!(plain.validate().expect_err("remote http").kind(), kernel::ErrorKind::Invalid);

        let no_password = TaosConfig { host: "td.example".into(), tls: true, ..Default::default() };
        assert_eq!(
            no_password.validate().expect_err("remote auth").kind(),
            kernel::ErrorKind::Invalid
        );
        let blank_password = TaosConfig {
            host: "td.example".into(),
            tls: true,
            password: "   ".into(),
            ..Default::default()
        };
        assert!(blank_password.validate().is_err());

        let secure = TaosConfig {
            host: "td.example".into(),
            tls: true,
            password: "configured".into(),
            ..Default::default()
        };
        secure.validate().expect("remote tls and auth");
    }

    #[test]
    fn host_classification_and_ipv6_are_strict() {
        for bad in ["localhost.evil", "127.0.0.1.evil", "user@localhost", "http://localhost"] {
            let cfg = TaosConfig { host: bad.into(), ..Default::default() };
            assert!(cfg.validate().is_err(), "bad host {bad}");
        }
        let ipv6 = TaosConfig { host: "::1".into(), ..Default::default() };
        ipv6.validate().expect("ipv6 loopback");
        assert_eq!(ipv6.rest_sql_url(), "http://[::1]:6041/rest/sql");
    }

    #[test]
    fn hard_resource_limits_fail_closed() {
        let too_many = TaosConfig { max_in_flight: HARD_MAX_IN_FLIGHT + 1, ..Default::default() };
        assert!(too_many.validate().is_err());
        let too_large =
            TaosConfig { max_response_bytes: HARD_MAX_RESPONSE_BYTES + 1, ..Default::default() };
        assert!(too_large.validate().is_err());
        let too_slow = TaosConfig {
            close_timeout: HARD_MAX_CLOSE_TIMEOUT + Duration::from_millis(1),
            ..Default::default()
        };
        assert!(too_slow.validate().is_err());
    }
}
