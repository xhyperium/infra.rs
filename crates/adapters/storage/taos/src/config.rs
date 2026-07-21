//! TDengine REST 连接配置。

use std::fmt;
use std::time::Duration;

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

/// TDengine REST 客户端配置。
///
/// 环境变量前缀：`FOUNDATIONX_TAOSX_`
/// - `HOST`（默认 `127.0.0.1`）
/// - `PORT`（默认 `6041` REST）
/// - `DATABASE`（默认 `infra_draft`）
/// - `USER`（默认 `root`）
/// - `PASSWORD`（默认空；**不**写入仓库）
/// - `TLS`（`1`/`true`/`yes` 启用 https）
/// - `TIMEOUT_MS`（默认 `10000`）
/// - `PRECISION`（可选 `ms`/`us`/`ns`；未设则连接后探测）
#[derive(Clone)]
pub struct TaosConfig {
    /// 主机。
    pub host: String,
    /// REST 端口。
    pub port: u16,
    /// 数据库名。
    pub database: String,
    /// 用户。
    pub user: String,
    /// 密码（`Debug` 脱敏）。
    pub password: String,
    /// 是否 HTTPS。
    pub tls: bool,
    /// 请求超时。
    pub timeout: Duration,
    /// 可选显式精度；`None` 时在 `connect` 后探测。
    pub precision: Option<TsPrecision>,
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
        cfg
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
}
