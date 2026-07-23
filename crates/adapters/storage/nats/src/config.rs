//! NATS 配置：环境变量、TLS 策略与本地默认值。
//!
//! 环境变量（canonical `FOUNDATIONX_NATS_*`，兼容 `FOUNDATIONX_NATSX_*`）：
//! - `URL` / `SERVERS`
//! - `USER` / `USERNAME`
//! - `PASSWORD`
//! - `TLS` — `1`/`true`/`yes` 开启 TLS 布尔开关
//! - `TLS_POLICY` — `disable` / `prefer` / `require`
//! - `OPERATION_TIMEOUT_MS`、`SUBSCRIPTION_CAPACITY`、`CLIENT_CAPACITY`
//! - `MAX_RECONNECTS`、`RECONNECT_MAX_DELAY_MS`
//!
//! **默认值面向本地/草稿联调**；生产必须通过环境注入。`Debug` 脱敏密码。
//!
//! ## TLS 默认策略
//!
//! - 显式 [`TlsPolicy`] 优先
//! - 否则若 `tls == true` → [`TlsPolicy::Require`]
//! - 否则按 URL host 自动：
//!   - loopback（`127.0.0.1` / `localhost` / `::1`）→ [`TlsPolicy::Prefer`]（允许明文）
//!   - 非 loopback → [`TlsPolicy::Require`]

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 默认 URL（无认证；生产凭据必须经环境注入）。
pub const DEFAULT_URL: &str = "nats://127.0.0.1:4222";

/// TLS 策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TlsPolicy {
    /// 强制禁用 TLS；仅允许 loopback 地址。
    Disable,
    /// 优先 TLS、允许明文；仅允许 loopback 地址。
    #[default]
    Prefer,
    /// 必须 TLS；连接层 `require_tls(true)`。
    Require,
}

impl TlsPolicy {
    /// 解析策略字符串。
    pub fn parse(s: &str) -> XResult<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "disable" | "disabled" | "off" | "false" | "0" | "none" => Ok(Self::Disable),
            "prefer" | "optional" | "auto" => Ok(Self::Prefer),
            "require" | "required" | "on" | "true" | "1" | "mandatory" => Ok(Self::Require),
            other => Err(XError::invalid(format!(
                "natsx: 未知 TLS_POLICY={other}（期望 disable|prefer|require）"
            ))),
        }
    }

    /// 是否在 connect options 上设置 `require_tls(true)`。
    #[must_use]
    pub fn require_tls(self) -> bool {
        matches!(self, Self::Require)
    }
}

impl fmt::Display for TlsPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disable => write!(f, "disable"),
            Self::Prefer => write!(f, "prefer"),
            Self::Require => write!(f, "require"),
        }
    }
}

/// NATS 客户端配置。
#[derive(Clone)]
pub struct NatsConfig {
    /// 服务器 URL（可逗号分隔多节点）。
    pub url: String,
    /// 用户名。
    pub user: Option<String>,
    /// 密码。
    pub password: Option<String>,
    /// 连接超时。
    pub connect_timeout: Duration,
    /// Core NATS / JetStream 管理操作截止时间。
    pub operation_timeout: Duration,
    /// 客户端名。
    pub name: String,
    /// 遗留 TLS 布尔开关（`true` 等价于 Require，除非 `tls_policy` 已显式设置）。
    pub tls: bool,
    /// 显式 TLS 策略；`None` 时按 host 自动解析（见 [`NatsConfig::effective_tls_policy`]）。
    pub tls_policy: Option<TlsPolicy>,
    /// 是否期望使用 JetStream API（文档/校验标志；不影响 Core NATS 连接）。
    pub jetstream: bool,
    /// 驱动每订阅缓冲和本 crate 转发缓冲上限。
    pub subscription_capacity: usize,
    /// 驱动命令发送队列容量。
    pub client_capacity: usize,
    /// 连续重连最大尝试次数；必须有限且大于零。
    pub max_reconnects: usize,
    /// 单次重连退避上限。
    pub reconnect_max_delay: Duration,
    /// 是否忽略服务端发现的地址，仅重连显式 URL（固定 ingress/端口映射场景）。
    pub ignore_discovered_servers: bool,
    // —— 以下为 P1/P2 新增认证与 TLS 证书字段 ——
    /// NKey seed（用户或账户私钥；安全敏感，不进入 Debug）。
    pub nkey_seed: Option<String>,
    /// JWT token（安全敏感，不进入 Debug）。
    pub jwt: Option<String>,
    /// CA bundle 文件路径（PEM 格式）。
    pub tls_ca_file: Option<String>,
    /// mTLS client certificate 文件路径（PEM 格式）。
    pub tls_cert_file: Option<String>,
    /// mTLS client key 文件路径（PEM 格式）。
    pub tls_key_file: Option<String>,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.to_string(),
            // 无默认账号：避免把草稿/过期凭据写进库；由 FOUNDATIONX_NATS_* 注入
            user: None,
            password: None,
            connect_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(5),
            name: "natsx".to_string(),
            tls: false,
            tls_policy: None,
            jetstream: false,
            subscription_capacity: 256,
            client_capacity: 256,
            max_reconnects: 60,
            reconnect_max_delay: Duration::from_secs(5),
            ignore_discovered_servers: false,
            nkey_seed: None,
            jwt: None,
            tls_ca_file: None,
            tls_cert_file: None,
            tls_key_file: None,
        }
    }
}

impl fmt::Debug for NatsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NatsConfig")
            .field("url", &redact_url(&self.url))
            .field("user", &self.user)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .field("connect_timeout", &self.connect_timeout)
            .field("operation_timeout", &self.operation_timeout)
            .field("name", &self.name)
            .field("tls", &self.tls)
            .field("tls_policy", &self.tls_policy)
            .field("jetstream", &self.jetstream)
            .field("subscription_capacity", &self.subscription_capacity)
            .field("client_capacity", &self.client_capacity)
            .field("max_reconnects", &self.max_reconnects)
            .field("reconnect_max_delay", &self.reconnect_max_delay)
            .field("ignore_discovered_servers", &self.ignore_discovered_servers)
            .field("nkey_seed", &self.nkey_seed.as_ref().map(|_| "***"))
            .field("jwt", &self.jwt.as_ref().map(|_| "***"))
            .field("tls_ca_file", &self.tls_ca_file)
            .field("tls_cert_file", &self.tls_cert_file)
            .field("tls_key_file", &self.tls_key_file)
            .finish()
    }
}

fn redact_url(raw: &str) -> String {
    let Ok(mut parsed) = url::Url::parse(raw) else {
        return "<invalid-url>".to_string();
    };
    if !parsed.username().is_empty() {
        let _ = parsed.set_username("***");
    }
    if parsed.password().is_some() {
        let _ = parsed.set_password(Some("***"));
    }
    parsed.to_string()
}

impl NatsConfig {
    /// 从环境变量加载。
    ///
    /// 优先级：`FOUNDATIONX_NATS_*` > `FOUNDATIONX_NATSX_*`。
    pub fn from_env() -> XResult<Self> {
        let mut cfg = Self::default();
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_URL", "FOUNDATIONX_NATSX_URL"]) {
            if !v.trim().is_empty() {
                cfg.url = v;
            }
        } else if let Some(v) =
            env_first(&["FOUNDATIONX_NATS_SERVERS", "FOUNDATIONX_NATSX_SERVERS"])
        {
            if !v.trim().is_empty() {
                // servers 列表 → 取第一项或原样
                cfg.url = v.split(',').next().unwrap_or(&v).trim().to_string();
            }
        }
        if let Some(v) = env_first(&[
            "FOUNDATIONX_NATS_USER",
            "FOUNDATIONX_NATS_USERNAME",
            "FOUNDATIONX_NATSX_USER",
            "FOUNDATIONX_NATSX_USERNAME",
        ]) {
            cfg.user = Some(v);
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_PASSWORD", "FOUNDATIONX_NATSX_PASSWORD"]) {
            cfg.password = Some(v);
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_NAME", "FOUNDATIONX_NATSX_NAME"]) {
            if !v.trim().is_empty() {
                cfg.name = v;
            }
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_TLS", "FOUNDATIONX_NATSX_TLS"]) {
            cfg.tls = parse_bool(&v, "FOUNDATIONX_NATS_TLS")?;
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_TLS_POLICY", "FOUNDATIONX_NATSX_TLS_POLICY"])
        {
            if !v.trim().is_empty() {
                cfg.tls_policy = Some(TlsPolicy::parse(&v)?);
            }
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_JETSTREAM", "FOUNDATIONX_NATSX_JETSTREAM"]) {
            cfg.jetstream = parse_bool(&v, "FOUNDATIONX_NATS_JETSTREAM")?;
        }
        apply_usize_env(
            &mut cfg.subscription_capacity,
            &["FOUNDATIONX_NATS_SUBSCRIPTION_CAPACITY", "FOUNDATIONX_NATSX_SUBSCRIPTION_CAPACITY"],
        )?;
        apply_usize_env(
            &mut cfg.client_capacity,
            &["FOUNDATIONX_NATS_CLIENT_CAPACITY", "FOUNDATIONX_NATSX_CLIENT_CAPACITY"],
        )?;
        apply_usize_env(
            &mut cfg.max_reconnects,
            &["FOUNDATIONX_NATS_MAX_RECONNECTS", "FOUNDATIONX_NATSX_MAX_RECONNECTS"],
        )?;
        apply_duration_ms_env(
            &mut cfg.operation_timeout,
            &["FOUNDATIONX_NATS_OPERATION_TIMEOUT_MS", "FOUNDATIONX_NATSX_OPERATION_TIMEOUT_MS"],
        )?;
        apply_duration_ms_env(
            &mut cfg.reconnect_max_delay,
            &[
                "FOUNDATIONX_NATS_RECONNECT_MAX_DELAY_MS",
                "FOUNDATIONX_NATSX_RECONNECT_MAX_DELAY_MS",
            ],
        )?;
        // NKey 与 JWT
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_NKEY_SEED", "FOUNDATIONX_NATSX_NKEY_SEED"]) {
            if !v.trim().is_empty() {
                cfg.nkey_seed = Some(v);
            }
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_JWT", "FOUNDATIONX_NATSX_JWT"]) {
            if !v.trim().is_empty() {
                cfg.jwt = Some(v);
            }
        }
        // TLS 证书文件路径
        if let Some(v) =
            env_first(&["FOUNDATIONX_NATS_TLS_CA_FILE", "FOUNDATIONX_NATSX_TLS_CA_FILE"])
        {
            if !v.trim().is_empty() {
                cfg.tls_ca_file = Some(v);
            }
        }
        if let Some(v) =
            env_first(&["FOUNDATIONX_NATS_TLS_CERT_FILE", "FOUNDATIONX_NATSX_TLS_CERT_FILE"])
        {
            if !v.trim().is_empty() {
                cfg.tls_cert_file = Some(v);
            }
        }
        if let Some(v) =
            env_first(&["FOUNDATIONX_NATS_TLS_KEY_FILE", "FOUNDATIONX_NATSX_TLS_KEY_FILE"])
        {
            if !v.trim().is_empty() {
                cfg.tls_key_file = Some(v);
            }
        };
        cfg.validate()?;
        Ok(cfg)
    }

    /// 生效的 TLS 策略（显式 > tls 布尔 > host 自动）。
    #[must_use]
    pub fn effective_tls_policy(&self) -> TlsPolicy {
        if let Some(p) = self.tls_policy {
            return p;
        }
        if self.tls {
            return TlsPolicy::Require;
        }
        if url_is_loopback(&self.url) { TlsPolicy::Prefer } else { TlsPolicy::Require }
    }

    /// URL 是否声明 TLS scheme（`tls://` / `nats+tls://` / `wss://`）。
    #[must_use]
    pub fn url_implies_tls(&self) -> bool {
        let lower = self.url.trim().to_ascii_lowercase();
        lower.starts_with("tls://")
            || lower.starts_with("nats+tls://")
            || lower.starts_with("wss://")
    }

    /// 校验。
    pub fn validate(&self) -> XResult<()> {
        if self.url.trim().is_empty() {
            return Err(XError::invalid("natsx: url 不能为空"));
        }
        let parsed = url::Url::parse(self.url.trim())
            .map_err(|error| XError::invalid("natsx: URL 非法").with_source(error))?;
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(XError::invalid(
                "natsx: URL 禁止内嵌 userinfo；请使用独立 user/password 字段",
            ));
        }
        match (&self.user, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {}
            (None, None) => {}
            _ => {
                return Err(XError::invalid("natsx: user/password 必须同时提供或同时缺省"));
            }
        }
        let policy = self.effective_tls_policy();
        if !url_is_loopback(&self.url) && policy != TlsPolicy::Require {
            return Err(XError::invalid("natsx: 远程服务必须使用 require TLS 策略"));
        }
        if self.connect_timeout.is_zero()
            || self.operation_timeout.is_zero()
            || self.reconnect_max_delay.is_zero()
        {
            return Err(XError::invalid("natsx: timeout 必须大于零"));
        }
        if self.subscription_capacity == 0 || self.client_capacity == 0 {
            return Err(XError::invalid("natsx: capacity 必须大于零"));
        }
        if self.max_reconnects == 0 {
            return Err(XError::invalid("natsx: max_reconnects 必须为有限正数"));
        }
        // NKey/JWT 必须同时提供或同时缺省；与 user/password 互斥
        match (&self.nkey_seed, &self.jwt) {
            (Some(_), Some(_)) => {
                if self.user.is_some() || self.password.is_some() {
                    return Err(XError::invalid(
                        "natsx: NKey/JWT 与 user/password 互斥，只能二选一",
                    ));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(XError::invalid("natsx: NKey seed 需要 JWT 同时提供"));
            }
            (None, Some(_)) => {
                return Err(XError::invalid("natsx: JWT 需要 NKey seed 同时提供"));
            }
        }
        // TLS 证书配置校验
        match (&self.tls_cert_file, &self.tls_key_file) {
            (Some(cert), Some(key)) => {
                if std::path::Path::new(cert).exists() && std::path::Path::new(key).exists() {
                    // cert + key 必须成对使用
                } else {
                    return Err(XError::invalid("natsx: TLS cert/key 文件路径不存在或不可访问"));
                }
                if policy == TlsPolicy::Disable {
                    return Err(XError::invalid("natsx: TLS 证书已配置但 TLS 策略为 Disable"));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(XError::invalid("natsx: TLS cert 文件需要 key 文件同时提供"));
            }
            (None, Some(_)) => {
                return Err(XError::invalid("natsx: TLS key 文件需要 cert 文件同时提供"));
            }
        }
        if let Some(ca) = &self.tls_ca_file {
            if !std::path::Path::new(ca).exists() {
                return Err(XError::invalid("natsx: TLS CA 文件路径不存在或不可访问"));
            }
        }
        Ok(())
    }
}

/// 判断 NATS URL 是否指向 loopback。
#[must_use]
pub fn url_is_loopback(url: &str) -> bool {
    let host = extract_host(url);
    matches!(host.to_ascii_lowercase().as_str(), "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

fn extract_host(url: &str) -> String {
    let s = url.trim();
    // 去掉 scheme
    let without_scheme = if let Some(idx) = s.find("://") { &s[idx + 3..] } else { s };
    // userinfo@host
    let after_user = without_scheme.rsplit('@').next().unwrap_or(without_scheme);
    // host:port 或 [ipv6]:port
    if let Some(rest) = after_user.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return format!("[{}]", &rest[..end]);
        }
    }
    let hostport = after_user.split('/').next().unwrap_or(after_user);
    hostport.split(':').next().unwrap_or(hostport).to_string()
}

fn env_first(keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            return Some(v);
        }
    }
    None
}

fn apply_usize_env(target: &mut usize, keys: &[&str]) -> XResult<()> {
    if let Some(value) = env_first(keys) {
        *target = value
            .parse::<usize>()
            .map_err(|error| XError::invalid(format!("{} 非法", keys[0])).with_source(error))?;
    }
    Ok(())
}

fn apply_duration_ms_env(target: &mut Duration, keys: &[&str]) -> XResult<()> {
    if let Some(value) = env_first(keys) {
        *target = value
            .parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|error| XError::invalid(format!("{} 非法", keys[0])).with_source(error))?;
    }
    Ok(())
}

fn parse_bool(value: &str, name: &str) -> XResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(XError::invalid(format!("{name} 非法: {value}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = NatsConfig::default();
        assert_eq!(c.url, DEFAULT_URL);
        assert!(c.user.is_none());
        assert!(c.password.is_none());
        assert!(!c.tls);
        assert!(c.tls_policy.is_none());
        assert!(!c.jetstream);
        assert!(c.validate().is_ok());
        // 默认 loopback → Prefer
        assert_eq!(c.effective_tls_policy(), TlsPolicy::Prefer);
    }

    #[test]
    fn debug_redacts_password() {
        let c = NatsConfig {
            url: "nats://embedded-user:embedded-secret@localhost:4222".into(),
            password: Some("super-secret-pass".into()),
            user: Some("u".into()),
            ..NatsConfig::default()
        };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("super-secret-pass"));
        assert!(!s.contains("embedded-user"));
        assert!(!s.contains("embedded-secret"));
    }

    #[test]
    fn tls_loopback_vs_remote_defaults() {
        let loopback = NatsConfig::default();
        assert_eq!(loopback.effective_tls_policy(), TlsPolicy::Prefer);

        let remote =
            NatsConfig { url: "nats://broker.example.com:4222".into(), ..NatsConfig::default() };
        assert_eq!(remote.effective_tls_policy(), TlsPolicy::Require);

        let remote_explicit_disable = NatsConfig {
            url: "nats://broker.example.com:4222".into(),
            tls_policy: Some(TlsPolicy::Disable),
            ..NatsConfig::default()
        };
        assert_eq!(remote_explicit_disable.effective_tls_policy(), TlsPolicy::Disable);
        assert_eq!(
            remote_explicit_disable.validate().expect_err("远程明文必须失败").kind(),
            kernel::ErrorKind::Invalid
        );

        let tls_bool =
            NatsConfig { url: "nats://127.0.0.1:4222".into(), tls: true, ..NatsConfig::default() };
        assert_eq!(tls_bool.effective_tls_policy(), TlsPolicy::Require);
    }

    #[test]
    fn require_reject_plain_remote_connect_flag() {
        // Require 对非 TLS scheme 远端：validate 通过，但 require_tls() == true
        let remote = NatsConfig {
            url: "nats://kafka-proxy.prod:4222".into(),
            tls_policy: Some(TlsPolicy::Require),
            ..NatsConfig::default()
        };
        assert!(remote.validate().is_ok());
        assert!(!remote.url_implies_tls());
        assert!(remote.effective_tls_policy().require_tls());

        // 远程 Prefer 不强制 TLS，配置层必须拒绝
        let prefer = NatsConfig {
            url: "nats://broker.example.com:4222".into(),
            tls_policy: Some(TlsPolicy::Prefer),
            ..NatsConfig::default()
        };
        assert!(!prefer.effective_tls_policy().require_tls());
        assert_eq!(
            prefer.validate().expect_err("远程 Prefer 必须失败").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn tls_policy_parse() {
        assert_eq!(TlsPolicy::parse("require").unwrap(), TlsPolicy::Require);
        assert_eq!(TlsPolicy::parse("DISABLE").unwrap(), TlsPolicy::Disable);
        assert_eq!(TlsPolicy::parse("prefer").unwrap(), TlsPolicy::Prefer);
        assert!(TlsPolicy::parse("weird").is_err());
    }

    #[test]
    fn url_is_loopback_matrix() {
        assert!(url_is_loopback("nats://127.0.0.1:4222"));
        assert!(url_is_loopback("nats://localhost:4222"));
        assert!(url_is_loopback("nats://[::1]:4222"));
        assert!(!url_is_loopback("nats://10.0.0.5:4222"));
        assert!(!url_is_loopback("tls://nats.prod.internal:4222"));
        assert!(!url_is_loopback("nats://0.0.0.0:4222"));
    }

    #[test]
    fn rejects_url_userinfo_and_unbounded_resources() {
        let userinfo =
            NatsConfig { url: "nats://user:secret@127.0.0.1:4222".into(), ..NatsConfig::default() };
        let debug = format!("{userinfo:?}");
        assert!(!debug.contains("secret"));
        assert_eq!(
            userinfo.validate().expect_err("userinfo 必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );

        for invalid in [
            NatsConfig { operation_timeout: Duration::ZERO, ..NatsConfig::default() },
            NatsConfig { subscription_capacity: 0, ..NatsConfig::default() },
            NatsConfig { client_capacity: 0, ..NatsConfig::default() },
            NatsConfig { max_reconnects: 0, ..NatsConfig::default() },
        ] {
            assert_eq!(
                invalid.validate().expect_err("无界配置必须拒绝").kind(),
                kernel::ErrorKind::Invalid
            );
        }
    }

    #[test]
    fn nkey_debug_redacts_seed_and_jwt() {
        let c = NatsConfig {
            nkey_seed: Some("SUACB...".into()),
            jwt: Some("eyJhbG...".into()),
            ..NatsConfig::default()
        };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("SUACB"));
        assert!(!s.contains("eyJhbG"));
    }

    #[test]
    fn nkey_requires_both_seed_and_jwt() {
        let seed_only =
            NatsConfig { nkey_seed: Some("s".into()), jwt: None, ..NatsConfig::default() };
        assert_eq!(
            seed_only.validate().expect_err("只有 seed 必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );

        let jwt_only =
            NatsConfig { nkey_seed: None, jwt: Some("j".into()), ..NatsConfig::default() };
        assert_eq!(
            jwt_only.validate().expect_err("只有 jwt 必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );

        let both = NatsConfig {
            nkey_seed: Some("s".into()),
            jwt: Some("j".into()),
            ..NatsConfig::default()
        };
        assert!(both.validate().is_ok());
    }

    #[test]
    fn nkey_mutually_exclusive_with_user_password() {
        let c = NatsConfig {
            nkey_seed: Some("s".into()),
            jwt: Some("j".into()),
            user: Some("u".into()),
            password: Some("p".into()),
            ..NatsConfig::default()
        };
        assert_eq!(
            c.validate().expect_err("NKey+user 必须互斥").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn tls_cert_requires_both_cert_and_key() {
        // 只设置 cert 不加 key
        let cert_only = NatsConfig {
            tls_cert_file: Some("/nonexistent/cert.pem".into()),
            ..NatsConfig::default()
        };
        assert_eq!(
            cert_only.validate().expect_err("只有 cert 没有 key 必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );

        // 都设置但路径不存在
        let both = NatsConfig {
            tls_cert_file: Some("/nonexistent/cert.pem".into()),
            tls_key_file: Some("/nonexistent/key.pem".into()),
            ..NatsConfig::default()
        };
        assert_eq!(
            both.validate().expect_err("路径不存在必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn tls_cert_rejects_disable_policy() {
        let c = NatsConfig {
            tls_policy: Some(TlsPolicy::Disable),
            tls_cert_file: Some("/nonexistent/cert.pem".into()),
            tls_key_file: Some("/nonexistent/key.pem".into()),
            ..NatsConfig::default()
        };
        assert_eq!(
            c.validate().expect_err("证书+Disable 必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn tls_ca_file_nonexistent_is_rejected() {
        let c =
            NatsConfig { tls_ca_file: Some("/nonexistent/ca.pem".into()), ..NatsConfig::default() };
        assert_eq!(
            c.validate().expect_err("CA 文件不存在必须拒绝").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn defaults_have_no_nkey_or_tls_certs() {
        let c = NatsConfig::default();
        assert!(c.nkey_seed.is_none());
        assert!(c.jwt.is_none());
        assert!(c.tls_ca_file.is_none());
        assert!(c.tls_cert_file.is_none());
        assert!(c.tls_key_file.is_none());
    }
}
