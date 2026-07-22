//! ClickHouse 连接配置（环境变量 + 默认值）。

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use kernel::{XError, XResult};

/// ClickHouse HTTP 客户端配置。
///
/// 环境变量前缀：`FOUNDATIONX_CLICKHOUSEX_`
/// - `HOST`（默认 `127.0.0.1`）
/// - `HTTP_PORT`（默认 `8123`；兼容别名 `PORT`）
/// - `USER`（默认 `default`）
/// - `PASSWORD`（默认空；**不**写入仓库）
/// - `DATABASE`（默认 `default`）
/// - `TIMEOUT_MS`（默认 `10000`）
/// - `MAX_IDLE_PER_HOST`（默认 `8`）
/// - `MAX_IN_FLIGHT`（默认 `64`）
/// - `ACQUIRE_TIMEOUT_MS`（默认 `5000`）
#[derive(Clone)]
pub struct ClickHouseConfig {
    /// 主机名或 IP。
    pub host: String,
    /// HTTP 端口（默认 8123）。
    pub http_port: u16,
    /// 是否使用 HTTPS（远程地址必须为 `true`）。
    pub tls: bool,
    /// 可选 PEM CA 文件；未设置时使用 reqwest/rustls 的公开可信根。
    pub tls_ca_file: Option<PathBuf>,
    /// 用户名。
    pub user: String,
    /// 密码（`Debug` 脱敏）。
    pub password: String,
    /// 默认数据库。
    pub database: String,
    /// 请求超时。
    pub timeout: Duration,
    /// reqwest 每主机最大空闲连接数。
    pub max_idle_per_host: usize,
    /// 全局 in-flight 上限（Semaphore 许可数，≥1）。
    pub max_in_flight: usize,
    /// 获取 in-flight 许可超时。
    pub acquire_timeout: Duration,
}

impl fmt::Debug for ClickHouseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClickHouseConfig")
            .field("host", &self.host)
            .field("http_port", &self.http_port)
            .field("tls", &self.tls)
            .field("tls_ca_file", &self.tls_ca_file)
            .field("user", &self.user)
            .field("password", &"***")
            .field("database", &self.database)
            .field("timeout", &self.timeout)
            .field("max_idle_per_host", &self.max_idle_per_host)
            .field("max_in_flight", &self.max_in_flight)
            .field("acquire_timeout", &self.acquire_timeout)
            .finish()
    }
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            http_port: 8123,
            tls: false,
            tls_ca_file: None,
            user: "default".into(),
            password: String::new(),
            database: "default".into(),
            timeout: Duration::from_secs(10),
            max_idle_per_host: 8,
            max_in_flight: 64,
            acquire_timeout: Duration::from_secs(5),
        }
    }
}

impl ClickHouseConfig {
    /// 从环境变量加载；未设置项使用 [`Default`]。
    pub fn from_env() -> XResult<Self> {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_HOST") {
            if !v.is_empty() {
                cfg.host = v;
            }
        }
        cfg.http_port = resolve_http_port(
            std::env::var("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT").ok().as_deref(),
            std::env::var("FOUNDATIONX_CLICKHOUSEX_PORT").ok().as_deref(),
            cfg.http_port,
        )?;
        if let Ok(value) = std::env::var("FOUNDATIONX_CLICKHOUSEX_TLS") {
            cfg.tls = parse_bool(&value)?;
        }
        if let Ok(value) = std::env::var("FOUNDATIONX_CLICKHOUSEX_TLS_CA_FILE") {
            if !value.trim().is_empty() {
                cfg.tls_ca_file = Some(PathBuf::from(value));
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
            cfg.timeout = Duration::from_millis(v.parse::<u64>().map_err(|error| {
                XError::invalid("FOUNDATIONX_CLICKHOUSEX_TIMEOUT_MS 非法").with_source(error)
            })?);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST") {
            cfg.max_idle_per_host = v.parse::<usize>().map_err(|error| {
                XError::invalid("FOUNDATIONX_CLICKHOUSEX_MAX_IDLE_PER_HOST 非法").with_source(error)
            })?;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT") {
            cfg.max_in_flight = v.parse::<usize>().map_err(|error| {
                XError::invalid("FOUNDATIONX_CLICKHOUSEX_MAX_IN_FLIGHT 非法").with_source(error)
            })?;
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS") {
            cfg.acquire_timeout = Duration::from_millis(v.parse::<u64>().map_err(|error| {
                XError::invalid("FOUNDATIONX_CLICKHOUSEX_ACQUIRE_TIMEOUT_MS 非法")
                    .with_source(error)
            })?);
        }
        cfg.validate()?;
        Ok(cfg)
    }

    /// 校验约束（`max_in_flight ≥ 1`）。
    pub fn validate(&self) -> XResult<()> {
        if self.max_in_flight < 1 {
            return Err(XError::invalid("max_in_flight 必须 ≥ 1"));
        }
        if self.timeout.is_zero() || self.acquire_timeout.is_zero() {
            return Err(XError::invalid("clickhouse timeout 必须大于零"));
        }
        if self.host.trim().is_empty() || self.http_port == 0 {
            return Err(XError::invalid("clickhouse host/port 非法"));
        }
        if !self.tls && !host_is_loopback(&self.host) {
            return Err(XError::invalid("远程 ClickHouse 必须使用 HTTPS"));
        }
        if self.tls_ca_file.is_some() && !self.tls {
            return Err(XError::invalid("配置 TLS_CA_FILE 时必须启用 TLS"));
        }
        Ok(())
    }

    /// HTTP(S) 基址。
    #[must_use]
    pub fn base_url(&self) -> String {
        let scheme = if self.tls { "https" } else { "http" };
        format!("{scheme}://{}:{}", self.host, self.http_port)
    }
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn resolve_http_port(
    http_port: Option<&str>,
    port_alias: Option<&str>,
    default: u16,
) -> XResult<u16> {
    let http_port = parse_optional_port("FOUNDATIONX_CLICKHOUSEX_HTTP_PORT", http_port)?;
    let port_alias = parse_optional_port("FOUNDATIONX_CLICKHOUSEX_PORT", port_alias)?;
    match (http_port, port_alias) {
        (Some(primary), Some(alias)) if primary != alias => Err(XError::invalid(
            "FOUNDATIONX_CLICKHOUSEX_HTTP_PORT 与 FOUNDATIONX_CLICKHOUSEX_PORT 冲突",
        )),
        (Some(primary), _) => Ok(primary),
        (None, Some(alias)) => Ok(alias),
        (None, None) => Ok(default),
    }
}

fn parse_optional_port(name: &'static str, value: Option<&str>) -> XResult<Option<u16>> {
    value
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|error| XError::invalid(format!("{name} 非法")).with_source(error))
        })
        .transpose()
}

fn parse_bool(value: &str) -> XResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(XError::invalid(format!("FOUNDATIONX_CLICKHOUSEX_TLS 非法: {value}"))),
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
        assert_eq!(c.max_idle_per_host, 8);
        assert_eq!(c.max_in_flight, 64);
        assert_eq!(c.base_url(), "http://127.0.0.1:8123");
        c.validate().expect("default valid");
    }

    #[test]
    fn debug_redacts_password() {
        let c = ClickHouseConfig { password: "secret-value".into(), ..Default::default() };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("secret-value"));
    }

    #[test]
    fn rejects_zero_max_in_flight() {
        let c = ClickHouseConfig { max_in_flight: 0, ..Default::default() };
        let err = c.validate().expect_err("must fail");
        assert!(format!("{err}").contains("max_in_flight"));
    }

    #[test]
    fn remote_http_fails_closed_and_https_is_selected() {
        let plain =
            ClickHouseConfig { host: "clickhouse.example.com".into(), ..Default::default() };
        assert_eq!(
            plain.validate().expect_err("远程 HTTP 必须失败").kind(),
            kernel::ErrorKind::Invalid
        );

        let tls = ClickHouseConfig {
            host: "clickhouse.example.com".into(),
            http_port: 8443,
            tls: true,
            ..Default::default()
        };
        tls.validate().expect("远程 HTTPS 应通过");
        assert_eq!(tls.base_url(), "https://clickhouse.example.com:8443");
    }

    #[test]
    fn ca_file_requires_tls_and_zero_deadlines_fail() {
        let ca_without_tls =
            ClickHouseConfig { tls_ca_file: Some("/tmp/ca.pem".into()), ..Default::default() };
        assert_eq!(
            ca_without_tls.validate().expect_err("CA 不得用于明文").kind(),
            kernel::ErrorKind::Invalid
        );
        assert_eq!(
            ClickHouseConfig { timeout: Duration::ZERO, ..Default::default() }
                .validate()
                .expect_err("零 timeout 必须失败")
                .kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn port_alias_is_supported_and_conflicts_fail_closed() {
        assert_eq!(resolve_http_port(None, None, 8123).expect("使用默认端口"), 8123);
        assert_eq!(resolve_http_port(None, Some("8443"), 8123).expect("兼容 PORT"), 8443);
        assert_eq!(resolve_http_port(Some("9440"), Some("9440"), 8123).expect("相同别名值"), 9440);

        let conflict =
            resolve_http_port(Some("8123"), Some("8443"), 8123).expect_err("冲突必须拒绝");
        assert_eq!(conflict.kind(), kernel::ErrorKind::Invalid);
        assert!(conflict.context().contains("冲突"));
    }

    #[test]
    fn invalid_port_alias_preserves_source_without_echoing_value() {
        let error = resolve_http_port(None, Some("secret-not-a-port"), 8123)
            .expect_err("非法 PORT 必须拒绝");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        assert!(std::error::Error::source(&error).is_some());
        assert!(!error.to_string().contains("secret-not-a-port"));
    }
}
