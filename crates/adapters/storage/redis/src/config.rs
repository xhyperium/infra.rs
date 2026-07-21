//! Redis 连接配置与 Builder（字段私有，防明文密码泄漏）。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 部署模式。P0 仅 Standalone 可用；Cluster/Sentinel 为占位，connect 时拒绝。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RedisMode {
    /// 单机。
    #[default]
    Standalone,
    /// 集群（P0 未实现）。
    Cluster,
    /// 哨兵（P0 未实现）。
    Sentinel,
}

/// Redis 连接配置（字段全部私有；通过 [`RedisConfig::builder`] / [`RedisConfig::from_env`] 构造）。
#[derive(Clone)]
pub struct RedisConfig {
    /// `host:port`，默认 `127.0.0.1:6379`。
    addr: String,
    /// ACL 用户名；`None` 表示不发送 username。
    username: Option<String>,
    /// 密码；Debug 中脱敏。
    password: Option<String>,
    /// 逻辑库编号。
    db: i64,
    /// 是否要求 TLS（P0 无 tls feature 时 connect 返回 Invalid）。
    tls: bool,
    mode: RedisMode,
    connect_timeout: Duration,
    command_timeout: Duration,
    acquire_timeout: Duration,
    /// 全局 in-flight 上限（Semaphore 许可数）。
    max_in_flight: usize,
    client_name: Option<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:6379".into(),
            username: None,
            password: None,
            db: 0,
            tls: false,
            mode: RedisMode::Standalone,
            connect_timeout: Duration::from_secs(5),
            command_timeout: Duration::from_secs(3),
            acquire_timeout: Duration::from_secs(3),
            max_in_flight: 256,
            client_name: None,
        }
    }
}

impl fmt::Debug for RedisConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisConfig")
            .field("addr", &self.addr)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .field("db", &self.db)
            .field("tls", &self.tls)
            .field("mode", &self.mode)
            .field("connect_timeout", &self.connect_timeout)
            .field("command_timeout", &self.command_timeout)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("max_in_flight", &self.max_in_flight)
            .field("client_name", &self.client_name)
            .finish()
    }
}

impl RedisConfig {
    /// 创建 Builder。
    #[must_use]
    pub fn builder() -> RedisConfigBuilder {
        RedisConfigBuilder { inner: Self::default() }
    }

    /// 从环境变量加载。
    ///
    /// 优先级：
    /// 1. `REDIS_URL`（若设置）覆盖地址/认证/库/TLS scheme；
    /// 2. 否则使用 `FOUNDATIONX_REDISX_*`：
    ///    - `FOUNDATIONX_REDISX_ADDR`（默认 `127.0.0.1:6379`）
    ///    - `FOUNDATIONX_REDISX_USERNAME`（默认 `default`）
    ///    - `FOUNDATIONX_REDISX_PASSWORD`（可选）
    ///    - `FOUNDATIONX_REDISX_DB`（默认 `0`）
    ///    - `FOUNDATIONX_REDISX_TLS`（默认 `false`）
    pub fn from_env() -> XResult<Self> {
        if let Ok(url) = std::env::var("REDIS_URL") {
            if !url.trim().is_empty() {
                return Self::from_url(&url);
            }
        }

        let mut b = Self::builder();
        let addr =
            std::env::var("FOUNDATIONX_REDISX_ADDR").unwrap_or_else(|_| "127.0.0.1:6379".into());
        b = b.addr(addr);

        // 规范默认 username=default；空字符串视为不设置
        let username =
            std::env::var("FOUNDATIONX_REDISX_USERNAME").unwrap_or_else(|_| "default".into());
        if !username.is_empty() {
            b = b.username(username);
        }

        if let Ok(pw) = std::env::var("FOUNDATIONX_REDISX_PASSWORD") {
            if !pw.is_empty() {
                b = b.password(pw);
            }
        }

        let db = match std::env::var("FOUNDATIONX_REDISX_DB") {
            Ok(s) if !s.is_empty() => s
                .parse::<i64>()
                .map_err(|e| XError::invalid(format!("FOUNDATIONX_REDISX_DB 非法: {e}")))?,
            _ => 0,
        };
        b = b.db(db);

        let tls = match std::env::var("FOUNDATIONX_REDISX_TLS") {
            Ok(s) => parse_bool(&s)?,
            Err(_) => false,
        };
        b = b.tls(tls);

        b.build()
    }

    /// 从 `redis://` / `rediss://` URL 解析配置。
    pub fn from_url(url: &str) -> XResult<Self> {
        use redis::IntoConnectionInfo;
        let info = url
            .into_connection_info()
            .map_err(|e| XError::invalid(format!("REDIS_URL 非法: {e}")))?;

        let (addr, tls) = match info.addr {
            redis::ConnectionAddr::Tcp(host, port) => (format!("{host}:{port}"), false),
            redis::ConnectionAddr::TcpTls { host, port, .. } => (format!("{host}:{port}"), true),
            redis::ConnectionAddr::Unix(path) => {
                return Err(XError::invalid(format!("不支持 unix socket URL: {}", path.display())));
            }
        };

        let mut b = Self::builder().addr(addr).db(info.redis.db).tls(tls);
        if let Some(u) = info.redis.username {
            b = b.username(u);
        }
        if let Some(p) = info.redis.password {
            b = b.password(p);
        }
        b.build()
    }

    /// 端点 `host:port`（不含凭据）。
    #[must_use]
    pub fn addr(&self) -> &str {
        &self.addr
    }

    /// 逻辑库。
    #[must_use]
    pub fn db(&self) -> i64 {
        self.db
    }

    /// 是否 TLS。
    #[must_use]
    pub fn tls(&self) -> bool {
        self.tls
    }

    /// 部署模式。
    #[must_use]
    pub fn mode(&self) -> RedisMode {
        self.mode
    }

    /// 连接超时。
    #[must_use]
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// 命令超时。
    #[must_use]
    pub fn command_timeout(&self) -> Duration {
        self.command_timeout
    }

    /// 获取 in-flight 许可的超时。
    #[must_use]
    pub fn acquire_timeout(&self) -> Duration {
        self.acquire_timeout
    }

    /// 最大 in-flight。
    #[must_use]
    pub fn max_in_flight(&self) -> usize {
        self.max_in_flight
    }

    /// 客户端名称（可选）。
    #[must_use]
    pub fn client_name(&self) -> Option<&str> {
        self.client_name.as_deref()
    }

    /// 脱敏展示端点（用于日志 / endpoint()）。
    #[must_use]
    pub fn display_endpoint(&self) -> String {
        let scheme = if self.tls { "rediss" } else { "redis" };
        let user = self.username.as_deref().unwrap_or("");
        if self.password.is_some() {
            if user.is_empty() {
                format!("{scheme}://***@{}/{}", self.addr, self.db)
            } else {
                format!("{scheme}://{user}:***@{}/{}", self.addr, self.db)
            }
        } else if user.is_empty() {
            format!("{scheme}://{}/{}", self.addr, self.db)
        } else {
            format!("{scheme}://{user}@{}/{}", self.addr, self.db)
        }
    }

    /// 构造底层 `redis::ConnectionInfo`（含密码，不得写入日志）。
    pub(crate) fn to_connection_info(&self) -> XResult<redis::ConnectionInfo> {
        if self.tls {
            return Err(XError::invalid(
                "TLS 已配置但当前 redisx 构建未启用 redis tls feature；请关闭 TLS 或后续升级依赖",
            ));
        }
        let (host, port) = parse_host_port(&self.addr)?;
        Ok(redis::ConnectionInfo {
            addr: redis::ConnectionAddr::Tcp(host, port),
            redis: redis::RedisConnectionInfo {
                db: self.db,
                username: self.username.clone(),
                password: self.password.clone(),
                protocol: Default::default(),
            },
        })
    }

    pub(crate) fn validate(&self) -> XResult<()> {
        if self.addr.trim().is_empty() {
            return Err(XError::invalid("Redis 地址不能为空"));
        }
        parse_host_port(&self.addr)?;
        if self.db < 0 {
            return Err(XError::invalid("Redis db 不能为负数"));
        }
        if self.max_in_flight == 0 {
            return Err(XError::invalid("max_in_flight 必须 ≥ 1"));
        }
        if self.connect_timeout.is_zero()
            || self.command_timeout.is_zero()
            || self.acquire_timeout.is_zero()
        {
            return Err(XError::invalid("超时时间必须 > 0"));
        }
        match self.mode {
            RedisMode::Standalone => Ok(()),
            RedisMode::Cluster => Err(XError::invalid("Cluster 模式 P0 未实现")),
            RedisMode::Sentinel => Err(XError::invalid("Sentinel 模式 P0 未实现")),
        }
    }
}

/// [`RedisConfig`] 的 Builder。
#[derive(Debug, Clone)]
pub struct RedisConfigBuilder {
    inner: RedisConfig,
}

impl RedisConfigBuilder {
    /// 设置 `host:port`。
    #[must_use]
    pub fn addr(mut self, addr: impl Into<String>) -> Self {
        self.inner.addr = addr.into();
        self
    }

    /// 设置 ACL 用户名。
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.inner.username = Some(username.into());
        self
    }

    /// 清除用户名。
    #[must_use]
    pub fn clear_username(mut self) -> Self {
        self.inner.username = None;
        self
    }

    /// 设置密码。
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.inner.password = Some(password.into());
        self
    }

    /// 清除密码。
    #[must_use]
    pub fn clear_password(mut self) -> Self {
        self.inner.password = None;
        self
    }

    /// 设置逻辑库。
    #[must_use]
    pub fn db(mut self, db: i64) -> Self {
        self.inner.db = db;
        self
    }

    /// 设置 TLS 开关。
    #[must_use]
    pub fn tls(mut self, tls: bool) -> Self {
        self.inner.tls = tls;
        self
    }

    /// 设置部署模式。
    #[must_use]
    pub fn mode(mut self, mode: RedisMode) -> Self {
        self.inner.mode = mode;
        self
    }

    /// 连接超时。
    #[must_use]
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.inner.connect_timeout = d;
        self
    }

    /// 单命令超时。
    #[must_use]
    pub fn command_timeout(mut self, d: Duration) -> Self {
        self.inner.command_timeout = d;
        self
    }

    /// 获取 in-flight 许可超时。
    #[must_use]
    pub fn acquire_timeout(mut self, d: Duration) -> Self {
        self.inner.acquire_timeout = d;
        self
    }

    /// 最大并发 in-flight 命令数。
    #[must_use]
    pub fn max_in_flight(mut self, n: usize) -> Self {
        self.inner.max_in_flight = n;
        self
    }

    /// 客户端名称（`CLIENT SETNAME` 由驱动/后续扩展使用）。
    #[must_use]
    pub fn client_name(mut self, name: impl Into<String>) -> Self {
        self.inner.client_name = Some(name.into());
        self
    }

    /// 校验并生成配置。
    pub fn build(self) -> XResult<RedisConfig> {
        self.inner.validate()?;
        Ok(self.inner)
    }
}

fn parse_host_port(addr: &str) -> XResult<(String, u16)> {
    let addr = addr.trim();
    // 支持 [ipv6]:port
    if let Some(rest) = addr.strip_prefix('[') {
        let (host, port_part) = rest
            .split_once("]:")
            .ok_or_else(|| XError::invalid(format!("非法 IPv6 地址: {addr}")))?;
        let port: u16 = port_part.parse().map_err(|e| XError::invalid(format!("非法端口: {e}")))?;
        return Ok((host.to_string(), port));
    }
    match addr.rsplit_once(':') {
        Some((host, port_s)) if !host.is_empty() => {
            let port: u16 =
                port_s.parse().map_err(|e| XError::invalid(format!("非法端口: {e}")))?;
            Ok((host.to_string(), port))
        }
        _ => Ok((addr.to_string(), 6379)),
    }
}

fn parse_bool(s: &str) -> XResult<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(XError::invalid(format!("布尔环境变量非法: {other}（期望 true/false）"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_password() {
        let cfg =
            RedisConfig::builder().password("super-secret").username("alice").build().expect("cfg");
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("***"), "password must be redacted: {dbg}");
        assert!(!dbg.contains("super-secret"), "leaked password: {dbg}");
        assert!(dbg.contains("alice"));
    }

    #[test]
    fn display_endpoint_redacts() {
        let cfg = RedisConfig::builder()
            .addr("10.0.0.1:6379")
            .username("u")
            .password("p")
            .db(2)
            .build()
            .expect("cfg");
        let ep = cfg.display_endpoint();
        assert!(ep.contains("***"));
        assert!(!ep.contains(":p@"));
        assert!(ep.contains("10.0.0.1:6379"));
    }

    #[test]
    fn rejects_zero_max_in_flight() {
        let err = RedisConfig::builder().max_in_flight(0).build().expect_err("must fail");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn rejects_cluster_mode_p0() {
        let err =
            RedisConfig::builder().mode(RedisMode::Cluster).build().expect_err("cluster not in p0");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn from_url_parses_auth() {
        let cfg = RedisConfig::from_url("redis://user:secret@127.0.0.1:6380/3").expect("url");
        assert_eq!(cfg.addr(), "127.0.0.1:6380");
        assert_eq!(cfg.db(), 3);
        assert_eq!(cfg.username.as_deref(), Some("user"));
        assert_eq!(cfg.password.as_deref(), Some("secret"));
        assert!(!cfg.tls());
    }

    #[test]
    fn parse_host_port_default() {
        let (h, p) = parse_host_port("localhost").unwrap();
        assert_eq!((h.as_str(), p), ("localhost", 6379));
    }
}
