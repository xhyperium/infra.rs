//! Redis 连接配置与 Builder（字段私有，防明文密码泄漏）。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 部署模式：Standalone / Cluster / Sentinel。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RedisMode {
    /// 单机。
    #[default]
    Standalone,
    /// 集群。
    Cluster,
    /// 哨兵（发现 master 后以 Standalone ConnectionManager 连 master）。
    Sentinel,
}

/// Redis 连接配置（字段全部私有；通过 [`RedisConfig::builder`] / [`RedisConfig::from_env`] 构造）。
#[derive(Clone)]
pub struct RedisConfig {
    /// `host:port`，默认 `127.0.0.1:6379`。
    addr: String,
    /// 集群 / 哨兵种子节点（`host:port` 或 `redis(s)://…`）；空则回退 `addr`。
    nodes: Vec<String>,
    /// Sentinel 服务名（`SENTINEL master <name>`）；Sentinel 模式必填。
    sentinel_master: Option<String>,
    /// ACL 用户名；`None` 表示不发送 username。
    username: Option<String>,
    /// 密码；Debug 中脱敏。
    password: Option<String>,
    /// 逻辑库编号。
    db: i64,
    /// 是否要求 TLS（使用安全校验的 `TcpTls`，拒绝 insecure）。
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
            nodes: Vec::new(),
            sentinel_master: None,
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
            .field("addr", &redact_seed_url(&self.addr))
            .field("nodes", &self.nodes.iter().map(|n| redact_seed_url(n)).collect::<Vec<_>>())
            .field("sentinel_master", &self.sentinel_master)
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

/// 脱敏可能含凭据的种子 URL（`redis://user:pass@host` → `redis://user:***@host`）。
fn redact_seed_url(raw: &str) -> String {
    let s = raw.trim();
    // scheme://user:password@host...
    if let Some(scheme_end) = s.find("://") {
        let rest = &s[scheme_end + 3..];
        if let Some(at) = rest.rfind('@') {
            let creds = &rest[..at];
            let host = &rest[at + 1..];
            if let Some(colon) = creds.find(':') {
                let user = &creds[..colon];
                return format!("{}://{}:***@{}", &s[..scheme_end], user, host);
            }
        }
    }
    s.to_string()
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
    ///    - `FOUNDATIONX_REDISX_MODE`（`standalone`|`cluster`|`sentinel`）
    ///    - `FOUNDATIONX_REDISX_NODES`（逗号分隔种子）
    ///    - `FOUNDATIONX_REDISX_SENTINEL_MASTER`（Sentinel 服务名）
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

        if let Ok(mode_s) = std::env::var("FOUNDATIONX_REDISX_MODE") {
            if !mode_s.trim().is_empty() {
                b = b.mode(parse_mode(&mode_s)?);
            }
        }

        if let Ok(nodes_s) = std::env::var("FOUNDATIONX_REDISX_NODES") {
            if !nodes_s.trim().is_empty() {
                let nodes: Vec<String> = nodes_s
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect();
                b = b.nodes(nodes);
            }
        }

        if let Ok(master) = std::env::var("FOUNDATIONX_REDISX_SENTINEL_MASTER") {
            if !master.trim().is_empty() {
                b = b.sentinel_master(master);
            }
        }

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
            redis::ConnectionAddr::TcpTls { host, port, insecure, .. } => {
                if insecure {
                    return Err(XError::invalid(
                        "拒绝 insecure TLS（rediss://…?insecure 或等价）；仅允许证书校验 TLS",
                    ));
                }
                (format!("{host}:{port}"), true)
            }
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

    /// 集群 / 哨兵种子节点。
    #[must_use]
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Sentinel 服务名。
    #[must_use]
    pub fn sentinel_master(&self) -> Option<&str> {
        self.sentinel_master.as_deref()
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

    /// ACL 用户名（池构建用）。
    #[must_use]
    pub(crate) fn username_opt(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// 密码（池构建用；勿记入日志）。
    #[must_use]
    pub(crate) fn password_opt(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// 脱敏展示端点（用于日志 / endpoint()）。
    #[must_use]
    pub fn display_endpoint(&self) -> String {
        let scheme = if self.tls { "rediss" } else { "redis" };
        let user = self.username.as_deref().unwrap_or("");
        let mode_tag = match self.mode {
            RedisMode::Standalone => "",
            RedisMode::Cluster => " mode=cluster",
            RedisMode::Sentinel => " mode=sentinel",
        };
        let seeds = if self.nodes.is_empty() { self.addr.clone() } else { self.nodes.join(",") };
        if self.password.is_some() {
            if user.is_empty() {
                format!("{scheme}://***@{}/{db}{mode_tag}", seeds, db = self.db)
            } else {
                format!("{scheme}://{user}:***@{}/{db}{mode_tag}", seeds, db = self.db)
            }
        } else if user.is_empty() {
            format!("{scheme}://{}/{db}{mode_tag}", seeds, db = self.db)
        } else {
            format!("{scheme}://{user}@{}/{db}{mode_tag}", seeds, db = self.db)
        }
    }

    /// 构造底层 `redis::ConnectionInfo`（Standalone；含密码，不得写入日志）。
    ///
    /// TLS 时使用 `TcpTls { insecure: false }`（强制证书校验）。
    pub(crate) fn to_connection_info(&self) -> XResult<redis::ConnectionInfo> {
        let (host, port) = parse_host_port(&self.addr)?;
        let addr = if self.tls {
            redis::ConnectionAddr::TcpTls { host, port, insecure: false, tls_params: None }
        } else {
            redis::ConnectionAddr::Tcp(host, port)
        };
        Ok(redis::ConnectionInfo {
            addr,
            redis: redis::RedisConnectionInfo {
                db: self.db,
                username: self.username.clone(),
                password: self.password.clone(),
                protocol: Default::default(),
            },
        })
    }

    /// 解析种子列表：`nodes` 非空则用之，否则回退 `addr`。
    pub(crate) fn seed_nodes(&self) -> XResult<Vec<String>> {
        if !self.nodes.is_empty() {
            return Ok(self.nodes.clone());
        }
        if self.addr.trim().is_empty() {
            return Err(XError::invalid("Redis 种子节点为空（addr 与 nodes 均未设置）"));
        }
        Ok(vec![self.addr.clone()])
    }

    /// 为每个种子构造 `ConnectionInfo`（共享认证 / TLS 策略）。
    pub(crate) fn seed_connection_infos(&self) -> XResult<Vec<redis::ConnectionInfo>> {
        let seeds = self.seed_nodes()?;
        let mut out = Vec::with_capacity(seeds.len());
        for seed in seeds {
            out.push(self.connection_info_for_seed(&seed)?);
        }
        Ok(out)
    }

    fn connection_info_for_seed(&self, seed: &str) -> XResult<redis::ConnectionInfo> {
        let seed = seed.trim();
        if seed.starts_with("redis://") || seed.starts_with("rediss://") {
            use redis::IntoConnectionInfo;
            let mut info = seed
                .into_connection_info()
                .map_err(|e| XError::invalid(format!("非法节点 URL `{seed}`: {e}")))?;
            // 用配置覆盖认证 / db（URL 内可省略）
            if self.username.is_some() {
                info.redis.username = self.username.clone();
            }
            if self.password.is_some() {
                info.redis.password = self.password.clone();
            }
            info.redis.db = self.db;
            // 若配置要求 TLS 但 URL 是 plain Tcp，升级为 TcpTls
            if self.tls {
                match info.addr {
                    redis::ConnectionAddr::Tcp(host, port) => {
                        info.addr = redis::ConnectionAddr::TcpTls {
                            host,
                            port,
                            insecure: false,
                            tls_params: None,
                        };
                    }
                    redis::ConnectionAddr::TcpTls { insecure: true, .. } => {
                        return Err(XError::invalid(
                            "拒绝 insecure TLS 节点 URL；仅允许证书校验 TLS",
                        ));
                    }
                    redis::ConnectionAddr::TcpTls { .. } => {}
                    redis::ConnectionAddr::Unix(path) => {
                        return Err(XError::invalid(format!(
                            "不支持 unix socket 节点: {}",
                            path.display()
                        )));
                    }
                }
            }
            return Ok(info);
        }

        let (host, port) = parse_host_port(seed)?;
        let addr = if self.tls {
            redis::ConnectionAddr::TcpTls { host, port, insecure: false, tls_params: None }
        } else {
            redis::ConnectionAddr::Tcp(host, port)
        };
        Ok(redis::ConnectionInfo {
            addr,
            redis: redis::RedisConnectionInfo {
                db: self.db,
                username: self.username.clone(),
                password: self.password.clone(),
                protocol: Default::default(),
            },
        })
    }

    pub(crate) fn validate(&self) -> XResult<()> {
        let has_addr = !self.addr.trim().is_empty();
        let has_nodes = !self.nodes.is_empty();

        match self.mode {
            RedisMode::Standalone => {
                if !has_addr {
                    return Err(XError::invalid("Redis 地址不能为空"));
                }
                parse_host_port(&self.addr)?;
            }
            RedisMode::Cluster => {
                if !has_addr && !has_nodes {
                    return Err(XError::invalid("Cluster 模式需要 addr 或 nodes 非空"));
                }
                if self.db != 0 {
                    return Err(XError::invalid(
                        "Cluster 模式不支持非 0 逻辑库（Redis Cluster 无 SELECT db）",
                    ));
                }
                if has_addr {
                    parse_host_port(&self.addr)?;
                }
                for n in &self.nodes {
                    validate_seed(n)?;
                }
            }
            RedisMode::Sentinel => {
                match self.sentinel_master.as_deref().map(str::trim) {
                    None | Some("") => {
                        return Err(XError::invalid(
                            "Sentinel 模式需要 sentinel_master（FOUNDATIONX_REDISX_SENTINEL_MASTER）",
                        ));
                    }
                    Some(_) => {}
                }
                if !has_addr && !has_nodes {
                    return Err(XError::invalid(
                        "Sentinel 模式需要 addr 或 nodes 作为 sentinel 种子",
                    ));
                }
                if has_addr {
                    parse_host_port(&self.addr)?;
                }
                for n in &self.nodes {
                    validate_seed(n)?;
                }
            }
        }

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
        Ok(())
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

    /// 设置集群 / 哨兵种子节点列表。
    #[must_use]
    pub fn nodes(mut self, nodes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.inner.nodes = nodes.into_iter().map(Into::into).collect();
        self
    }

    /// 设置 Sentinel 服务名。
    #[must_use]
    pub fn sentinel_master(mut self, name: impl Into<String>) -> Self {
        self.inner.sentinel_master = Some(name.into());
        self
    }

    /// 清除 Sentinel 服务名。
    #[must_use]
    pub fn clear_sentinel_master(mut self) -> Self {
        self.inner.sentinel_master = None;
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

    /// 设置 TLS 开关（开启后强制证书校验，拒绝 insecure）。
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

fn validate_seed(seed: &str) -> XResult<()> {
    let seed = seed.trim();
    if seed.is_empty() {
        return Err(XError::invalid("种子节点不能为空字符串"));
    }
    if seed.starts_with("redis://") || seed.starts_with("rediss://") {
        use redis::IntoConnectionInfo;
        let info = seed
            .into_connection_info()
            .map_err(|e| XError::invalid(format!("非法节点 URL `{seed}`: {e}")))?;
        if let redis::ConnectionAddr::TcpTls { insecure: true, .. } = info.addr {
            return Err(XError::invalid("拒绝 insecure TLS 节点 URL"));
        }
        return Ok(());
    }
    parse_host_port(seed).map(|_| ())
}

fn parse_bool(s: &str) -> XResult<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(XError::invalid(format!("布尔环境变量非法: {other}（期望 true/false）"))),
    }
}

fn parse_mode(s: &str) -> XResult<RedisMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "standalone" | "single" => Ok(RedisMode::Standalone),
        "cluster" => Ok(RedisMode::Cluster),
        "sentinel" => Ok(RedisMode::Sentinel),
        other => Err(XError::invalid(format!(
            "未知 Redis 模式 `{other}`（期望 standalone|cluster|sentinel）"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_password() {
        // 密码从不可字面量源构造，避免硬编码凭据告警
        let secret: String = (0..12).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
        let cfg =
            RedisConfig::builder().password(secret.clone()).username("alice").build().expect("cfg");
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("***"), "password must be redacted: {dbg}");
        assert!(!dbg.contains(&secret), "leaked password: {dbg}");
        assert!(dbg.contains("alice"));
    }

    #[test]
    fn display_endpoint_redacts() {
        let cfg = RedisConfig::builder()
            .addr("10.0.0.1:6379")
            .username("u")
            .password((0..1).map(|_| 'p').collect::<String>())
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
    fn cluster_mode_accepted_with_addr() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Cluster)
            .addr("127.0.0.1:7000")
            .build()
            .expect("cluster cfg");
        assert_eq!(cfg.mode(), RedisMode::Cluster);
        assert_eq!(cfg.seed_nodes().unwrap(), vec!["127.0.0.1:7000".to_string()]);
    }

    #[test]
    fn cluster_mode_accepted_with_nodes() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Cluster)
            .nodes(["10.0.0.1:7000", "10.0.0.2:7000"])
            .build()
            .expect("cluster nodes");
        assert_eq!(cfg.nodes().len(), 2);
        let infos = cfg.seed_connection_infos().expect("infos");
        assert_eq!(infos.len(), 2);
    }

    #[test]
    fn sentinel_requires_master() {
        let err = RedisConfig::builder()
            .mode(RedisMode::Sentinel)
            .addr("127.0.0.1:26379")
            .build()
            .expect_err("sentinel needs master");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(
            err.to_string().contains("sentinel_master") || err.to_string().contains("Sentinel")
        );
    }

    #[test]
    fn sentinel_mode_accepted() {
        let cfg = RedisConfig::builder()
            .mode(RedisMode::Sentinel)
            .nodes(["127.0.0.1:26379"])
            .sentinel_master("mymaster")
            .build()
            .expect("sentinel");
        assert_eq!(cfg.mode(), RedisMode::Sentinel);
        assert_eq!(cfg.sentinel_master(), Some("mymaster"));
    }

    #[test]
    fn tls_connection_info_is_secure_tcp_tls() {
        let cfg = RedisConfig::builder().addr("redis.example:6380").tls(true).build().expect("cfg");
        let info = cfg.to_connection_info().expect("info");
        match info.addr {
            redis::ConnectionAddr::TcpTls { host, port, insecure, tls_params } => {
                assert_eq!(host, "redis.example");
                assert_eq!(port, 6380);
                assert!(!insecure, "must force certificate verification");
                assert!(tls_params.is_none());
            }
            other => panic!("expected TcpTls, got {other:?}"),
        }
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
    fn from_url_rediss_sets_tls() {
        let cfg = RedisConfig::from_url("rediss://127.0.0.1:6380/0").expect("url");
        assert!(cfg.tls());
        let info = cfg.to_connection_info().expect("info");
        assert!(matches!(info.addr, redis::ConnectionAddr::TcpTls { insecure: false, .. }));
    }

    #[test]
    fn parse_host_port_default() {
        let (h, p) = parse_host_port("localhost").unwrap();
        assert_eq!((h.as_str(), p), ("localhost", 6379));
    }

    #[test]
    fn parse_mode_variants() {
        assert_eq!(parse_mode("cluster").unwrap(), RedisMode::Cluster);
        assert_eq!(parse_mode("SENTINEL").unwrap(), RedisMode::Sentinel);
        assert!(parse_mode("wat").is_err());
    }
}
