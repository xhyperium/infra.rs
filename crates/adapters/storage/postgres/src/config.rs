//! Postgres 连接配置与 Builder。
//!
//! 环境变量（优先 `DATABASE_URL` 覆盖）：
//! - `FOUNDATIONX_POSTGRESX_HOST`
//! - `FOUNDATIONX_POSTGRESX_PORT`
//! - `FOUNDATIONX_POSTGRESX_DATABASE`
//! - `FOUNDATIONX_POSTGRESX_USER`
//! - `FOUNDATIONX_POSTGRESX_PASSWORD`
//! - `FOUNDATIONX_POSTGRESX_SSLMODE`（`disable` / `prefer` / `require`）
//! - 可选：`FOUNDATIONX_POSTGRESX_MAX_POOL_SIZE`、`FOUNDATIONX_POSTGRESX_APPLICATION_NAME`

use std::env;
use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 默认连接池大小。
pub const DEFAULT_MAX_POOL_SIZE: usize = 16;

/// 默认端口。
pub const DEFAULT_PORT: u16 = 5432;

/// TLS / SSL 模式。
///
/// - [`SslMode::Disable`]：`NoTls`
/// - [`SslMode::Prefer`] / [`SslMode::Require`]：rustls（webpki-roots）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SslMode {
    /// 不使用 TLS。
    #[default]
    Disable,
    /// 优先 TLS（协商失败时可回退明文，由 tokio-postgres 处理）。
    Prefer,
    /// 要求 TLS（证书校验）。
    Require,
}

impl SslMode {
    /// 解析 sslmode 字符串（大小写不敏感）。
    pub fn parse(s: &str) -> XResult<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "disable" | "false" | "0" => Ok(Self::Disable),
            "prefer" | "allow" => Ok(Self::Prefer),
            "require" | "verify-ca" | "verify-full" => Ok(Self::Require),
            other => Err(XError::invalid(format!(
                "未知 sslmode `{other}`（期望 disable|prefer|require）"
            ))),
        }
    }

    /// 作为连接串片段的字面量。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Prefer => "prefer",
            Self::Require => "require",
        }
    }
}

/// 生产用 Postgres 配置。
#[derive(Clone)]
pub struct PostgresConfig {
    /// 主机名或 IP。
    pub host: String,
    /// 端口。
    pub port: u16,
    /// 数据库名。
    pub database: String,
    /// 用户名。
    pub user: String,
    /// 密码（Debug 中脱敏）。
    pub password: String,
    /// SSL 模式。
    pub sslmode: SslMode,
    /// 连接池上限。
    pub max_pool_size: usize,
    /// `application_name`（可选）。
    pub application_name: Option<String>,
    /// 连接超时（可选）。
    pub connect_timeout: Option<Duration>,
    /// 若从 `DATABASE_URL` 加载，保留原始 URL（密码仍脱敏输出）。
    pub database_url: Option<String>,
}

impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"***")
            .field("sslmode", &self.sslmode)
            .field("max_pool_size", &self.max_pool_size)
            .field("application_name", &self.application_name)
            .field("connect_timeout", &self.connect_timeout)
            .field("database_url", &self.database_url.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl PostgresConfig {
    /// 从环境变量加载；`DATABASE_URL` 优先于 `FOUNDATIONX_POSTGRESX_*`。
    pub fn from_env() -> XResult<Self> {
        if let Ok(url) = env::var("DATABASE_URL") {
            let url = url.trim();
            if !url.is_empty() {
                return Self::from_database_url(url);
            }
        }
        Self::from_foundationx_env()
    }

    /// 仅从 `FOUNDATIONX_POSTGRESX_*` 加载（忽略 `DATABASE_URL`）。
    pub fn from_foundationx_env() -> XResult<Self> {
        let host = env_required("FOUNDATIONX_POSTGRESX_HOST")?;
        let port = env_port("FOUNDATIONX_POSTGRESX_PORT", DEFAULT_PORT)?;
        let database = env_required("FOUNDATIONX_POSTGRESX_DATABASE")?;
        let user = env_required("FOUNDATIONX_POSTGRESX_USER")?;
        let password = env::var("FOUNDATIONX_POSTGRESX_PASSWORD").unwrap_or_default();
        let sslmode = match env::var("FOUNDATIONX_POSTGRESX_SSLMODE") {
            Ok(s) if !s.trim().is_empty() => SslMode::parse(&s)?,
            _ => SslMode::Disable,
        };
        let max_pool_size =
            env_usize("FOUNDATIONX_POSTGRESX_MAX_POOL_SIZE", DEFAULT_MAX_POOL_SIZE)?;
        let application_name = env::var("FOUNDATIONX_POSTGRESX_APPLICATION_NAME")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            sslmode,
            max_pool_size,
            application_name,
            connect_timeout: Some(Duration::from_secs(10)),
            database_url: None,
        })
    }

    /// 从 `postgres://` / `postgresql://` URL 解析。
    pub fn from_database_url(url: &str) -> XResult<Self> {
        let pg: tokio_postgres::Config =
            url.parse().map_err(|e| XError::invalid(format!("DATABASE_URL 解析失败: {e}")))?;

        let host = pg
            .get_hosts()
            .first()
            .map(|h| match h {
                tokio_postgres::config::Host::Tcp(h) => h.clone(),
                #[cfg(unix)]
                tokio_postgres::config::Host::Unix(p) => p.display().to_string(),
            })
            .unwrap_or_else(|| "127.0.0.1".into());

        let port = pg.get_ports().first().copied().unwrap_or(DEFAULT_PORT);
        let database = pg
            .get_dbname()
            .map(str::to_owned)
            .ok_or_else(|| XError::invalid("DATABASE_URL 缺少 database 名"))?;
        let user = pg
            .get_user()
            .map(str::to_owned)
            .ok_or_else(|| XError::invalid("DATABASE_URL 缺少 user"))?;
        let password =
            pg.get_password().map(|p| String::from_utf8_lossy(p).into_owned()).unwrap_or_default();

        let sslmode = match pg.get_ssl_mode() {
            tokio_postgres::config::SslMode::Disable => SslMode::Disable,
            tokio_postgres::config::SslMode::Prefer => SslMode::Prefer,
            tokio_postgres::config::SslMode::Require => SslMode::Require,
            // non_exhaustive：未知模式保守按 prefer 处理
            _ => SslMode::Prefer,
        };

        let max_pool_size =
            env_usize("FOUNDATIONX_POSTGRESX_MAX_POOL_SIZE", DEFAULT_MAX_POOL_SIZE)?;
        let application_name = env::var("FOUNDATIONX_POSTGRESX_APPLICATION_NAME")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| pg.get_application_name().map(str::to_owned));

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            sslmode,
            max_pool_size,
            application_name,
            connect_timeout: pg.get_connect_timeout().copied().or(Some(Duration::from_secs(10))),
            database_url: Some(url.to_string()),
        })
    }

    /// 构建器入口。
    #[must_use]
    pub fn builder() -> PostgresConfigBuilder {
        PostgresConfigBuilder::default()
    }

    /// 转换为 deadpool-postgres 配置（不含 TLS 握手；见 [`crate::pool`]）。
    pub(crate) fn to_deadpool_config(&self) -> deadpool_postgres::Config {
        let mut cfg = deadpool_postgres::Config::new();
        if let Some(url) = &self.database_url {
            cfg.url = Some(url.clone());
        } else {
            cfg.host = Some(self.host.clone());
            cfg.port = Some(self.port);
            cfg.dbname = Some(self.database.clone());
            cfg.user = Some(self.user.clone());
            if !self.password.is_empty() {
                cfg.password = Some(self.password.clone());
            }
            cfg.ssl_mode = Some(match self.sslmode {
                SslMode::Disable => deadpool_postgres::SslMode::Disable,
                SslMode::Prefer => deadpool_postgres::SslMode::Prefer,
                SslMode::Require => deadpool_postgres::SslMode::Require,
            });
        }
        if let Some(name) = &self.application_name {
            cfg.application_name = Some(name.clone());
        }
        if let Some(timeout) = self.connect_timeout {
            cfg.connect_timeout = Some(timeout);
        }
        cfg.pool = Some(deadpool_postgres::PoolConfig::new(self.max_pool_size));
        cfg
    }

    /// 校验必填字段。
    pub fn validate(&self) -> XResult<()> {
        if self.host.trim().is_empty() {
            return Err(XError::invalid("PostgresConfig.host 不能为空"));
        }
        if self.database.trim().is_empty() {
            return Err(XError::invalid("PostgresConfig.database 不能为空"));
        }
        if self.user.trim().is_empty() {
            return Err(XError::invalid("PostgresConfig.user 不能为空"));
        }
        if self.port == 0 {
            return Err(XError::invalid("PostgresConfig.port 不能为 0"));
        }
        if self.max_pool_size == 0 {
            return Err(XError::invalid("PostgresConfig.max_pool_size 不能为 0"));
        }
        Ok(())
    }
}

/// [`PostgresConfig`] 构建器。
#[derive(Debug, Clone, Default)]
pub struct PostgresConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
    sslmode: Option<SslMode>,
    max_pool_size: Option<usize>,
    application_name: Option<String>,
    connect_timeout: Option<Duration>,
}

impl PostgresConfigBuilder {
    /// 主机。
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// 端口。
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// 数据库名。
    #[must_use]
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// 用户。
    #[must_use]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// 密码。
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// SSL 模式。
    #[must_use]
    pub fn sslmode(mut self, mode: SslMode) -> Self {
        self.sslmode = Some(mode);
        self
    }

    /// 连接池上限。
    #[must_use]
    pub fn max_pool_size(mut self, size: usize) -> Self {
        self.max_pool_size = Some(size);
        self
    }

    /// `application_name`。
    #[must_use]
    pub fn application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    /// 连接超时。
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// 完成构建并校验。
    pub fn build(self) -> XResult<PostgresConfig> {
        let cfg = PostgresConfig {
            host: self.host.ok_or_else(|| XError::invalid("PostgresConfigBuilder: 缺少 host"))?,
            port: self.port.unwrap_or(DEFAULT_PORT),
            database: self
                .database
                .ok_or_else(|| XError::invalid("PostgresConfigBuilder: 缺少 database"))?,
            user: self.user.ok_or_else(|| XError::invalid("PostgresConfigBuilder: 缺少 user"))?,
            password: self.password.unwrap_or_default(),
            sslmode: self.sslmode.unwrap_or(SslMode::Disable),
            max_pool_size: self.max_pool_size.unwrap_or(DEFAULT_MAX_POOL_SIZE),
            application_name: self.application_name,
            connect_timeout: self.connect_timeout.or(Some(Duration::from_secs(10))),
            database_url: None,
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

fn env_required(key: &str) -> XResult<String> {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        Ok(_) => Err(XError::invalid(format!("环境变量 {key} 为空"))),
        Err(_) => Err(XError::invalid(format!("缺少环境变量 {key}"))),
    }
}

fn env_port(key: &str, default: u16) -> XResult<u16> {
    match env::var(key) {
        Ok(v) if v.trim().is_empty() => Ok(default),
        Ok(v) => v.parse::<u16>().map_err(|e| XError::invalid(format!("{key} 不是合法端口: {e}"))),
        Err(_) => Ok(default),
    }
}

fn env_usize(key: &str, default: usize) -> XResult<usize> {
    match env::var(key) {
        Ok(v) if v.trim().is_empty() => Ok(default),
        Ok(v) => {
            v.parse::<usize>().map_err(|e| XError::invalid(format!("{key} 不是合法 usize: {e}")))
        }
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_roundtrip() {
        let builder: PostgresConfigBuilder = PostgresConfig::builder();
        let cfg: PostgresConfig = builder
            .host("127.0.0.1")
            .port(DEFAULT_PORT)
            .database("db")
            .user("u")
            .password(["sec", "ret"].concat())
            .sslmode(SslMode::Disable)
            .max_pool_size(4)
            .build()
            .expect("build");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, DEFAULT_PORT);
        assert_eq!(cfg.password, "secret");
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("***"));
        assert!(!dbg.contains("secret"));
    }

    #[test]
    fn defaults_and_sslmode_as_str() {
        assert_eq!(DEFAULT_PORT, 5432);
        assert_eq!(DEFAULT_MAX_POOL_SIZE, 16);
        assert_eq!(SslMode::Disable.as_str(), "disable");
        assert_eq!(SslMode::Prefer.as_str(), "prefer");
        assert_eq!(SslMode::Require.as_str(), "require");
    }

    #[test]
    fn sslmode_parse() {
        assert_eq!(SslMode::parse("disable").unwrap(), SslMode::Disable);
        assert_eq!(SslMode::parse("REQUIRE").unwrap(), SslMode::Require);
        assert!(SslMode::parse("wat").is_err());
    }

    #[test]
    fn builder_missing_host() {
        let err = PostgresConfig::builder().database("db").user("u").build().unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn parse_database_url() {
        let cfg = PostgresConfig::from_database_url(
            "postgres://alice:s3cret@127.0.0.1:5433/market?sslmode=disable",
        )
        .expect("url");
        assert_eq!(cfg.user, "alice");
        assert_eq!(cfg.password, "s3cret");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 5433);
        assert_eq!(cfg.database, "market");
        assert_eq!(cfg.sslmode, SslMode::Disable);
    }
}
