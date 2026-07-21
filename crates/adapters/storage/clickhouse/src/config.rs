//! ClickHouse 连接配置（环境变量 + 默认值）。

use std::fmt;
use std::time::Duration;

/// ClickHouse HTTP 客户端配置。
///
/// 环境变量前缀：`FOUNDATIONX_CLICKHOUSEX_`
/// - `HOST`（默认 `127.0.0.1`）
/// - `HTTP_PORT`（默认 `8123`）
/// - `USER`（默认 `default`）
/// - `PASSWORD`（默认空；**不**写入仓库）
/// - `DATABASE`（默认 `default`）
/// - `TIMEOUT_MS`（默认 `10000`）
#[derive(Clone)]
pub struct ClickHouseConfig {
    /// 主机名或 IP。
    pub host: String,
    /// HTTP 端口（默认 8123）。
    pub http_port: u16,
    /// 用户名。
    pub user: String,
    /// 密码（`Debug` 脱敏）。
    pub password: String,
    /// 默认数据库。
    pub database: String,
    /// 请求超时。
    pub timeout: Duration,
}

impl fmt::Debug for ClickHouseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClickHouseConfig")
            .field("host", &self.host)
            .field("http_port", &self.http_port)
            .field("user", &self.user)
            .field("password", &"***")
            .field("database", &self.database)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            http_port: 8123,
            user: "default".into(),
            password: String::new(),
            database: "default".into(),
            timeout: Duration::from_secs(10),
        }
    }
}

impl ClickHouseConfig {
    /// 从环境变量加载；未设置项使用 [`Default`]。
    #[must_use]
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_HOST") {
            if !v.is_empty() {
                cfg.host = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT") {
            if let Ok(p) = v.parse() {
                cfg.http_port = p;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_USER") {
            if !v.is_empty() {
                cfg.user = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_PASSWORD") {
            cfg.password = v;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_DATABASE") {
            if !v.is_empty() {
                cfg.database = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS") {
            if let Ok(ms) = v.parse::<u64>() {
                cfg.timeout = Duration::from_millis(ms.max(1));
            }
        }
        cfg
    }

    /// HTTP 基址：`http://host:port`。
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.http_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let c = ClickHouseConfig::default();
        assert_eq!(c.host, "127.0.0.1");
        assert_eq!(c.http_port, 8123);
        assert_eq!(c.user, "default");
        assert!(c.password.is_empty());
        assert_eq!(c.base_url(), "http://127.0.0.1:8123");
    }

    #[test]
    fn debug_redacts_password() {
        let c = ClickHouseConfig { password: "secret-value".into(), ..Default::default() };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("secret-value"));
    }
}
