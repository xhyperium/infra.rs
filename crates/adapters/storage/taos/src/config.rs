//! TDengine REST / Native WS 连接配置。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

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
        }
    }
}

impl TaosConfig {
    /// 从环境变量加载。
    #[must_use]
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_HOST") {
            if !v.is_empty() {
                cfg.host = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PORT") {
            if let Ok(p) = v.parse() {
                cfg.port = p;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_DATABASE") {
            if !v.is_empty() {
                cfg.database = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_USER") {
            if !v.is_empty() {
                cfg.user = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PASSWORD") {
            cfg.password = v;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TLS") {
            cfg.tls = matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on");
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TIMEOUT_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                cfg.timeout = Duration::from_millis(ms.max(1));
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_PRECISION") {
            cfg.precision = TsPrecision::parse(&v);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_TRANSPORT") {
            if let Some(m) = TransportMode::parse(&v) {
                cfg.transport = m;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_MAX_IN_FLIGHT") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_in_flight = n;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_ACQUIRE_TIMEOUT_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                cfg.acquire_timeout = Duration::from_millis(ms.max(1));
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_TAOSX_BATCH_MAX_ROWS") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.batch_max_rows = n.max(1);
            }
        }
        cfg
    }

    /// 校验约束。
    pub fn validate(&self) -> XResult<()> {
        if self.max_in_flight < 1 {
            return Err(XError::invalid("max_in_flight 必须 ≥ 1"));
        }
        if self.batch_max_rows < 1 {
            return Err(XError::invalid("batch_max_rows 必须 ≥ 1"));
        }
        if self.host.trim().is_empty() {
            return Err(XError::invalid("taos host 不能为空"));
        }
        Ok(())
    }

    /// REST SQL 端点：`http(s)://host:port/rest/sql`。
    #[must_use]
    pub fn rest_sql_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{scheme}://{}:{}/rest/sql", self.host, self.port)
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
        format!("{scheme}://{}:{}/rest/ws", self.host, self.port)
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
}
