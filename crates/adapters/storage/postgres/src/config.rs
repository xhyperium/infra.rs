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
//! - `FOUNDATIONX_POSTGRESX_ACQUIRE_TIMEOUT_MS` / `OPERATION_TIMEOUT_MS`
//! - 可选 TLS：`FOUNDATIONX_POSTGRESX_TLS_CA_FILE`（PEM 额外根/服务端证书）
//! - 可选 TLS：`FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME`（SNI/证书名；连接 host 为 IP 时使用）
//! - 可选 mTLS：`FOUNDATIONX_POSTGRESX_TLS_CLIENT_CERT` + `TLS_CLIENT_KEY`（必须成对）

use std::env;
use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use kernel::{XError, XResult};

/// 默认连接池大小。
pub const DEFAULT_MAX_POOL_SIZE: usize = 16;

/// 默认端口。
pub const DEFAULT_PORT: u16 = 5432;

/// TLS / SSL 模式。
///
/// - [`SslMode::Disable`]：`NoTls`
/// - [`SslMode::Prefer`] / [`SslMode::Require`]：rustls（webpki-roots + 可选额外 CA）
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
    /// 等待池连接的截止时间。
    pub acquire_timeout: Duration,
    /// SQL 与事务终结操作的调用侧截止时间；同时下发为服务端 `statement_timeout`。
    pub operation_timeout: Duration,
    /// 额外 PEM CA / 服务端证书文件（叠加 webpki 公共根；**非** insecure 旁路）。
    pub tls_ca_file: Option<PathBuf>,
    /// TLS SNI / 证书校验名。
    ///
    /// 当 [`Self::host`] 为 IP 且证书 CN/SAN 为 DNS 名时设置；建池时使用
    /// `hostaddr=IP` + `host=server_name` 以保证连接地址与证书名分离。
    pub tls_server_name: Option<String>,
    /// mTLS 客户端证书 PEM 路径（须与 [`Self::tls_client_key`] 成对）。
    pub tls_client_cert: Option<PathBuf>,
    /// mTLS 客户端私钥 PEM 路径（须与 [`Self::tls_client_cert`] 成对）。
    pub tls_client_key: Option<PathBuf>,
    /// 原始 `DATABASE_URL` 兼容字段；建池不直接消费此字段。
    ///
    /// 仅用于一个迁移周期的源码兼容。调用方修改后，`validate` 会核对其与结构化字段
    /// 完全一致，并拒绝未实现参数；请改用 [`Self::from_database_url`] 构造后只读消费。
    #[deprecated(note = "请使用 PostgresConfig::from_database_url；原始 URL 不再作为执行配置")]
    pub database_url: Option<String>,
}

impl fmt::Debug for PostgresConfig {
    #[allow(deprecated)] // 兼容字段仅以固定脱敏占位输出
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
            .field("acquire_timeout", &self.acquire_timeout)
            .field("operation_timeout", &self.operation_timeout)
            .field("tls_ca_file", &self.tls_ca_file)
            .field("tls_server_name", &self.tls_server_name)
            .field("tls_client_cert", &self.tls_client_cert)
            .field(
                "tls_client_key",
                &self.tls_client_key.as_ref().map(|_| PathBuf::from("<redacted>")),
            )
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
    #[allow(deprecated)] // 构造一个迁移周期内保留的空兼容字段
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
        let acquire_timeout =
            env_duration_ms("FOUNDATIONX_POSTGRESX_ACQUIRE_TIMEOUT_MS", Duration::from_secs(5))?;
        let operation_timeout =
            env_duration_ms("FOUNDATIONX_POSTGRESX_OPERATION_TIMEOUT_MS", Duration::from_secs(10))?;
        let tls_ca_file = env::var("FOUNDATIONX_POSTGRESX_TLS_CA_FILE")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);
        let tls_server_name =
            env::var("FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME").ok().filter(|s| !s.trim().is_empty());
        let tls_client_cert = env::var("FOUNDATIONX_POSTGRESX_TLS_CLIENT_CERT")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);
        let tls_client_key = env::var("FOUNDATIONX_POSTGRESX_TLS_CLIENT_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

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
            acquire_timeout,
            operation_timeout,
            tls_ca_file,
            tls_server_name,
            tls_client_cert,
            tls_client_key,
            database_url: None,
        })
    }

    /// 从 `postgres://` / `postgresql://` URL 解析。
    #[allow(deprecated)] // 权威构造器负责同步填充迁移兼容字段
    pub fn from_database_url(url: &str) -> XResult<Self> {
        let url = url.trim();
        validate_database_url_query(url)?;
        let pg: tokio_postgres::Config = url
            .parse()
            .map_err(|error| XError::invalid("DATABASE_URL 解析失败").with_source(error))?;

        if pg.get_hosts().len() > 1 {
            return Err(XError::invalid("DATABASE_URL 暂不支持多 host"));
        }

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
        // URL 中的显式约束优先；只有 URL 未指定时才允许环境变量补充。
        let application_name = pg.get_application_name().map(str::to_owned).or_else(|| {
            env::var("FOUNDATIONX_POSTGRESX_APPLICATION_NAME")
                .ok()
                .filter(|value| !value.trim().is_empty())
        });
        let acquire_timeout =
            env_duration_ms("FOUNDATIONX_POSTGRESX_ACQUIRE_TIMEOUT_MS", Duration::from_secs(5))?;
        let operation_timeout =
            env_duration_ms("FOUNDATIONX_POSTGRESX_OPERATION_TIMEOUT_MS", Duration::from_secs(10))?;
        let tls_ca_file = env::var("FOUNDATIONX_POSTGRESX_TLS_CA_FILE")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);
        let tls_server_name =
            env::var("FOUNDATIONX_POSTGRESX_TLS_SERVER_NAME").ok().filter(|s| !s.trim().is_empty());
        let tls_client_cert = env::var("FOUNDATIONX_POSTGRESX_TLS_CLIENT_CERT")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);
        let tls_client_key = env::var("FOUNDATIONX_POSTGRESX_TLS_CLIENT_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);
        let read_replicas = env::var("FOUNDATIONX_POSTGRESX_READ_REPLICAS")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.split(',').map(|h| h.trim().to_string()).collect())
            .unwrap_or_default();

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
            acquire_timeout,
            operation_timeout,
            tls_ca_file,
            tls_server_name,
            tls_client_cert,
            tls_client_key,
            database_url: Some(url.to_string()),
            read_replicas,
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
        // DATABASE_URL 在入口处只解析一次；此处始终从已校验字段重建配置，避免
        // 原始 URL 的 sslmode 与公开字段被分别修改后产生策略/执行不一致。
        //
        // TLS：若配置了 tls_server_name 且 host 为 IP，则 hostaddr=IP、host=server_name，
        // 使 SNI/证书校验名与 TCP 目标分离（自签/企业证书常见场景）。
        if let Some(server_name) = self.tls_server_name.as_deref().filter(|s| !s.is_empty()) {
            if let Ok(ip) = self.host.parse::<IpAddr>() {
                cfg.hostaddr = Some(ip);
                cfg.host = Some(server_name.to_owned());
            } else {
                cfg.host = Some(server_name.to_owned());
            }
        } else {
            cfg.host = Some(self.host.clone());
        }
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
        if let Some(name) = &self.application_name {
            cfg.application_name = Some(name.clone());
        }
        if let Some(timeout) = self.connect_timeout {
            cfg.connect_timeout = Some(timeout);
        }
        cfg.options = Some(format!("-c statement_timeout={}", self.operation_timeout.as_millis()));
        cfg.manager = Some(deadpool_postgres::ManagerConfig {
            // Clean 会拒绝/丢弃仍处于事务中的旧兼容 raw-pool 对象，并清理 session 状态。
            // recycle timeout 继续保证 busy raw object 不会无限阻塞获取路径。
            recycling_method: deadpool_postgres::RecyclingMethod::Clean,
        });
        let mut pool = deadpool_postgres::PoolConfig::new(self.max_pool_size);
        pool.timeouts = deadpool_postgres::Timeouts {
            wait: Some(self.acquire_timeout),
            create: self.connect_timeout,
            recycle: Some(self.acquire_timeout),
        };
        cfg.pool = Some(pool);
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
        if self.connect_timeout.is_some_and(|timeout| timeout.is_zero())
            || self.acquire_timeout.is_zero()
            || self.operation_timeout.is_zero()
        {
            return Err(XError::invalid("PostgresConfig timeout 必须大于零"));
        }
        if self.sslmode != SslMode::Require && self.has_remote_host()? {
            return Err(XError::invalid(
                "远程 PostgreSQL 必须使用 sslmode=require；disable/prefer 仅允许本机",
            ));
        }
        if let Some(path) = &self.tls_ca_file {
            if path.as_os_str().is_empty() {
                return Err(XError::invalid("PostgresConfig.tls_ca_file 不能为空路径"));
            }
        }
        if let Some(name) = &self.tls_server_name {
            if name.trim().is_empty() {
                return Err(XError::invalid("PostgresConfig.tls_server_name 不能为空"));
            }
        }
        match (&self.tls_client_cert, &self.tls_client_key) {
            (None, None) => {}
            (Some(cert), Some(key)) => {
                if cert.as_os_str().is_empty() || key.as_os_str().is_empty() {
                    return Err(XError::invalid("PostgresConfig.tls_client_cert/key 不能为空路径"));
                }
            }
            _ => {
                return Err(XError::invalid(
                    "PostgresConfig mTLS 需要同时设置 tls_client_cert 与 tls_client_key",
                ));
            }
        }
        self.validate_database_url_consistency()?;
        Ok(())
    }

    fn has_remote_host(&self) -> XResult<bool> {
        Ok(!self.host.starts_with('/') && !host_is_loopback(&self.host))
    }

    #[allow(deprecated)]
    fn validate_database_url_consistency(&self) -> XResult<()> {
        let Some(url) = &self.database_url else {
            return Ok(());
        };
        validate_database_url_query(url)?;
        let parsed: tokio_postgres::Config = url
            .parse()
            .map_err(|error| XError::invalid("DATABASE_URL 解析失败").with_source(error))?;
        if parsed.get_hosts().len() > 1 {
            return Err(XError::invalid("DATABASE_URL 暂不支持多 host"));
        }
        let parsed_host = parsed
            .get_hosts()
            .first()
            .map(|host| match host {
                tokio_postgres::config::Host::Tcp(host) => host.clone(),
                #[cfg(unix)]
                tokio_postgres::config::Host::Unix(path) => path.display().to_string(),
            })
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let parsed_port = parsed.get_ports().first().copied().unwrap_or(DEFAULT_PORT);
        let parsed_database = parsed.get_dbname().unwrap_or_default();
        let parsed_user = parsed.get_user().unwrap_or_default();
        let parsed_password =
            parsed.get_password().map(|value| String::from_utf8_lossy(value)).unwrap_or_default();
        let parsed_sslmode = match parsed.get_ssl_mode() {
            tokio_postgres::config::SslMode::Disable => SslMode::Disable,
            tokio_postgres::config::SslMode::Prefer => SslMode::Prefer,
            tokio_postgres::config::SslMode::Require => SslMode::Require,
            _ => SslMode::Prefer,
        };
        let application_name_matches = parsed
            .get_application_name()
            .is_none_or(|value| self.application_name.as_deref() == Some(value));
        let connect_timeout_matches =
            parsed.get_connect_timeout().is_none_or(|value| self.connect_timeout == Some(*value));
        if parsed_host != self.host
            || parsed_port != self.port
            || parsed_database != self.database
            || parsed_user != self.user
            || parsed_password != self.password
            || parsed_sslmode != self.sslmode
            || !application_name_matches
            || !connect_timeout_matches
        {
            return Err(XError::invalid("DATABASE_URL 与结构化连接字段不一致；禁止双配置漂移"));
        }
        Ok(())
    }
}

fn validate_database_url_query(url: &str) -> XResult<()> {
    if !(url.starts_with("postgres://") || url.starts_with("postgresql://")) {
        return Err(XError::invalid(
            "DATABASE_URL 仅接受 postgres:// 或 postgresql:// URL；禁止 keyword DSN",
        ));
    }
    let Some((_, query_and_fragment)) = url.split_once('?') else {
        return Ok(());
    };
    let query = query_and_fragment.split('#').next().unwrap_or_default();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let key = pair.split_once('=').map_or(pair, |(key, _)| key);
        if !matches!(key, "sslmode" | "application_name" | "connect_timeout") {
            return Err(XError::invalid(format!(
                "DATABASE_URL 参数 `{key}` 未实现；禁止静默忽略认证或会话约束"
            )));
        }
    }
    Ok(())
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
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
    acquire_timeout: Option<Duration>,
    operation_timeout: Option<Duration>,
    tls_ca_file: Option<PathBuf>,
    tls_server_name: Option<String>,
    tls_client_cert: Option<PathBuf>,
    tls_client_key: Option<PathBuf>,
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

    /// 等待池连接的截止时间。
    #[must_use]
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = Some(timeout);
        self
    }

    /// SQL 与事务终结操作的截止时间。
    #[must_use]
    pub fn operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = Some(timeout);
        self
    }

    /// 额外 PEM CA 文件路径。
    #[must_use]
    pub fn tls_ca_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_ca_file = Some(path.into());
        self
    }

    /// TLS SNI / 证书校验服务器名。
    #[must_use]
    pub fn tls_server_name(mut self, name: impl Into<String>) -> Self {
        self.tls_server_name = Some(name.into());
        self
    }

    /// mTLS 客户端证书 PEM。
    #[must_use]
    pub fn tls_client_cert(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_client_cert = Some(path.into());
        self
    }

    /// mTLS 客户端私钥 PEM。
    #[must_use]
    pub fn tls_client_key(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_client_key = Some(path.into());
        self
    }

    /// 完成构建并校验。
    #[allow(deprecated)] // Builder 构造一个迁移周期内保留的空兼容字段
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
            acquire_timeout: self.acquire_timeout.unwrap_or(Duration::from_secs(5)),
            operation_timeout: self.operation_timeout.unwrap_or(Duration::from_secs(10)),
            tls_ca_file: self.tls_ca_file,
            tls_server_name: self.tls_server_name,
            tls_client_cert: self.tls_client_cert,
            tls_client_key: self.tls_client_key,
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
        Ok(v) => v
            .parse::<u16>()
            .map_err(|error| XError::invalid(format!("{key} 不是合法端口")).with_source(error)),
        Err(_) => Ok(default),
    }
}

fn env_usize(key: &str, default: usize) -> XResult<usize> {
    match env::var(key) {
        Ok(v) if v.trim().is_empty() => Ok(default),
        Ok(v) => v
            .parse::<usize>()
            .map_err(|error| XError::invalid(format!("{key} 不是合法 usize")).with_source(error)),
        Err(_) => Ok(default),
    }
}

fn env_duration_ms(key: &str, default: Duration) -> XResult<Duration> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => Ok(default),
        Ok(value) => value
            .parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|error| XError::invalid(format!("{key} 不是合法毫秒数")).with_source(error)),
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
    fn tls_server_name_with_ip_host_sets_hostaddr() {
        let cfg = PostgresConfig::builder()
            .host("84.247.154.45")
            .database("postgres")
            .user("postgres")
            .sslmode(SslMode::Require)
            .tls_server_name("X-16v-64g")
            .build()
            .expect("cfg");
        let dp = cfg.to_deadpool_config();
        assert_eq!(dp.host.as_deref(), Some("X-16v-64g"));
        assert_eq!(
            dp.hostaddr,
            Some("84.247.154.45".parse().expect("ip")),
            "IP 必须走 hostaddr，SNI 走 host"
        );
    }

    #[test]
    fn mtls_requires_paired_client_identity() {
        let err = PostgresConfig::builder()
            .host("127.0.0.1")
            .database("db")
            .user("u")
            .sslmode(SslMode::Require)
            .tls_client_cert("/tmp/only.crt")
            .build()
            .expect_err("cert only");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(err.context().contains("tls_client_cert"));
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
    fn remote_plaintext_and_prefer_fail_closed() {
        for mode in [SslMode::Disable, SslMode::Prefer] {
            let error = PostgresConfig::builder()
                .host("db.example.com")
                .database("db")
                .user("user")
                .sslmode(mode)
                .build()
                .expect_err("远程非 require 必须失败");
            assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        }

        PostgresConfig::builder()
            .host("db.example.com")
            .database("db")
            .user("user")
            .sslmode(SslMode::Require)
            .build()
            .expect("远程 require 应通过");
    }

    #[test]
    fn rejects_zero_timeouts() {
        for result in [
            PostgresConfig::builder()
                .host("127.0.0.1")
                .database("db")
                .user("user")
                .connect_timeout(Duration::ZERO)
                .build(),
            PostgresConfig::builder()
                .host("127.0.0.1")
                .database("db")
                .user("user")
                .acquire_timeout(Duration::ZERO)
                .build(),
            PostgresConfig::builder()
                .host("127.0.0.1")
                .database("db")
                .user("user")
                .operation_timeout(Duration::ZERO)
                .build(),
        ] {
            assert_eq!(result.expect_err("零 timeout 必须失败").kind(), kernel::ErrorKind::Invalid);
        }
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

    #[test]
    fn database_url_cannot_bypass_mutated_tls_policy() {
        #[allow(deprecated)]
        let mut cfg = PostgresConfig::from_database_url(
            "postgres://alice:secret@db.example.com:5432/market?sslmode=disable",
        )
        .expect("url 可解析，连接前再执行远程策略校验");
        cfg.sslmode = SslMode::Require;
        let error = cfg.validate().expect_err("URL 与结构化 TLS 字段漂移必须 fail-closed");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn database_url_rejects_unimplemented_security_and_session_parameters() {
        for url in [
            "postgres://alice:secret@db.example.com/market?sslmode=require&channel_binding=require",
            "postgres://alice:secret@db.example.com/market?sslmode=require&target_session_attrs=read-write",
            "postgres://alice:secret@db.example.com/market?sslmode=require&options=-c%20role%3Dadmin",
        ] {
            let error = PostgresConfig::from_database_url(url)
                .expect_err("未传播的 URL 参数必须 fail-closed");
            assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        }
    }

    #[test]
    fn database_url_rejects_keyword_dsn_and_allowed_field_drift() {
        let keyword = PostgresConfig::from_database_url(
            "host=127.0.0.1 user=alice dbname=market sslmode=disable channel_binding=require",
        )
        .expect_err("keyword DSN 不得绕过 URL 参数 allowlist");
        assert_eq!(keyword.kind(), kernel::ErrorKind::Invalid);

        let mut application = PostgresConfig::from_database_url(
            "postgres://alice:secret@127.0.0.1/market?sslmode=disable&application_name=expected",
        )
        .expect("显式 application_name 可解析");
        application.application_name = Some("drifted".to_string());
        assert!(application.validate().is_err(), "application_name 漂移必须失败");

        let mut timeout = PostgresConfig::from_database_url(
            "postgres://alice:secret@127.0.0.1/market?sslmode=disable&connect_timeout=7",
        )
        .expect("显式 connect_timeout 可解析");
        timeout.connect_timeout = Some(Duration::from_secs(8));
        assert!(timeout.validate().is_err(), "connect_timeout 漂移必须失败");
    }
}
