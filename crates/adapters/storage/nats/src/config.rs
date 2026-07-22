//! NATS 配置：环境变量、TLS 策略与本地默认值。
//!
//! 环境变量（canonical `FOUNDATIONX_NATS_*`，兼容 `FOUNDATIONX_NATSX_*`）：
//! - `URL` / `SERVERS`
//! - `USER` / `USERNAME`
//! - `PASSWORD`
//! - `TLS` — `1`/`true`/`yes` 开启 TLS 布尔开关
//! - `TLS_POLICY` — `disable` / `prefer` / `require`
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
    /// 强制禁用 TLS（即使远端 URL 也不要求）。
    Disable,
    /// 优先 TLS；明文亦可（典型：本机联调）。
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
    /// 客户端名。
    pub name: String,
    /// 遗留 TLS 布尔开关（`true` 等价于 Require，除非 `tls_policy` 已显式设置）。
    pub tls: bool,
    /// 显式 TLS 策略；`None` 时按 host 自动解析（见 [`NatsConfig::effective_tls_policy`]）。
    pub tls_policy: Option<TlsPolicy>,
    /// 是否期望使用 JetStream API（文档/校验标志；不影响 Core NATS 连接）。
    pub jetstream: bool,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.to_string(),
            // 无默认账号：避免把草稿/过期凭据写进库；由 FOUNDATIONX_NATS_* 注入
            user: None,
            password: None,
            connect_timeout: Duration::from_secs(5),
            name: "natsx".to_string(),
            tls: false,
            tls_policy: None,
            jetstream: false,
        }
    }
}

impl fmt::Debug for NatsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NatsConfig")
            .field("url", &self.url)
            .field("user", &self.user)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .field("connect_timeout", &self.connect_timeout)
            .field("name", &self.name)
            .field("tls", &self.tls)
            .field("tls_policy", &self.tls_policy)
            .field("jetstream", &self.jetstream)
            .finish()
    }
}

impl NatsConfig {
    /// 从环境变量加载。
    ///
    /// 优先级：`FOUNDATIONX_NATS_*` > `FOUNDATIONX_NATSX_*`。
    pub fn from_env() -> Self {
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
            cfg.tls = parse_bool(&v);
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_TLS_POLICY", "FOUNDATIONX_NATSX_TLS_POLICY"])
        {
            if let Ok(p) = TlsPolicy::parse(&v) {
                cfg.tls_policy = Some(p);
            }
        }
        if let Some(v) = env_first(&["FOUNDATIONX_NATS_JETSTREAM", "FOUNDATIONX_NATSX_JETSTREAM"]) {
            cfg.jetstream = parse_bool(&v);
        }
        cfg
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
        match (&self.user, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {}
            (None, None) => {}
            _ => {
                return Err(XError::invalid("natsx: user/password 必须同时提供或同时缺省"));
            }
        }
        // Require 且 URL 非 TLS scheme：允许，但 connect 时会 require_tls(true)。
        // 若调用方显式 Disable 却给了 tls://，仅告警级——仍允许（由驱动处理）。
        let policy = self.effective_tls_policy();
        if policy == TlsPolicy::Require && !self.url_implies_tls() && !url_is_loopback(&self.url) {
            // 非 loopback + Require + 明文 scheme：合法，连接层强制 TLS
            // 此处不 fail；文档约定由 ConnectOptions.require_tls 保证
        }
        if let Some(explicit) = self.tls_policy {
            // 重新 parse 以保持 API 可测；此处 no-op 校验
            let _ = explicit;
        }
        Ok(())
    }
}

/// 判断 NATS URL 是否指向 loopback。
#[must_use]
pub fn url_is_loopback(url: &str) -> bool {
    let host = extract_host(url);
    matches!(
        host.to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "::1" | "[::1]" | "0.0.0.0"
    )
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

fn parse_bool(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
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
            password: Some("super-secret-pass".into()),
            user: Some("u".into()),
            ..NatsConfig::default()
        };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("super-secret-pass"));
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

        // Prefer 不强制
        let prefer = NatsConfig {
            url: "nats://broker.example.com:4222".into(),
            tls_policy: Some(TlsPolicy::Prefer),
            ..NatsConfig::default()
        };
        assert!(!prefer.effective_tls_policy().require_tls());
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
    }
}
