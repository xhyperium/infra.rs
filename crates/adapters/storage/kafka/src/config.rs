//! Kafka 配置：环境变量与本地默认值。
//!
//! 环境变量（`FOUNDATIONX_KAFKAX_*`）：
//! - `BROKERS` — bootstrap servers（逗号分隔）
//! - `SASL_MECHANISM` — 如 `PLAIN`；空串关闭 SASL
//! - `SASL_USERNAME` / `SASL_PASSWORD` — SASL 凭据
//! - `TLS` — `1`/`true`/`yes` 开启 TLS（`SASL_SSL` 或 `SSL`）
//!
//! **默认值面向本地/草稿联调**，生产环境务必通过环境变量注入凭据；
//! 本类型的 `Debug` 会脱敏密码。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 默认 bootstrap。
pub const DEFAULT_BROKERS: &str = "127.0.0.1:9092";
/// 默认 SASL 机制名（启用 SASL 时使用；凭据必须经环境注入）。
pub const DEFAULT_SASL_MECHANISM: &str = "PLAIN";

/// Kafka 客户端配置。
#[derive(Clone)]
pub struct KafkaConfig {
    /// bootstrap.servers
    pub brokers: String,
    /// client.id
    pub client_id: String,
    /// SASL 机制；`None` 表示明文无认证。
    pub sasl_mechanism: Option<String>,
    /// SASL 用户名。
    pub sasl_username: Option<String>,
    /// SASL 密码（Debug 脱敏）。
    pub sasl_password: Option<String>,
    /// 是否启用 TLS。
    pub tls: bool,
    /// 投递等待超时。
    pub delivery_timeout: Duration,
    /// EventBus::subscribe 使用的默认消费组前缀。
    pub event_bus_group_prefix: String,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: DEFAULT_BROKERS.to_string(),
            client_id: "kafkax".to_string(),
            // 无默认账号：避免把真实/草稿凭据写进库
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            tls: false,
            delivery_timeout: Duration::from_secs(30),
            event_bus_group_prefix: "kafkax-eventbus".to_string(),
        }
    }
}

impl fmt::Debug for KafkaConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KafkaConfig")
            .field("brokers", &self.brokers)
            .field("client_id", &self.client_id)
            .field("sasl_mechanism", &self.sasl_mechanism)
            .field("sasl_username", &self.sasl_username)
            .field("sasl_password", &self.sasl_password.as_ref().map(|_| "***"))
            .field("tls", &self.tls)
            .field("delivery_timeout", &self.delivery_timeout)
            .field("event_bus_group_prefix", &self.event_bus_group_prefix)
            .finish()
    }
}

impl KafkaConfig {
    /// 从环境变量加载，缺省回落 [`KafkaConfig::default`]。
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_BROKERS") {
            if !v.trim().is_empty() {
                cfg.brokers = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_SASL_MECHANISM") {
            let t = v.trim();
            if t.is_empty() || t.eq_ignore_ascii_case("none") {
                cfg.sasl_mechanism = None;
                cfg.sasl_username = None;
                cfg.sasl_password = None;
            } else {
                cfg.sasl_mechanism = Some(t.to_string());
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_SASL_USERNAME") {
            cfg.sasl_username = Some(v);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_SASL_PASSWORD") {
            cfg.sasl_password = Some(v);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_TLS") {
            cfg.tls = parse_bool(&v);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_CLIENT_ID") {
            if !v.trim().is_empty() {
                cfg.client_id = v;
            }
        }
        cfg
    }

    /// 校验配置完整性。
    pub fn validate(&self) -> XResult<()> {
        if self.brokers.trim().is_empty() {
            return Err(XError::invalid("kafkax: brokers 不能为空"));
        }
        if self.tls {
            return Err(XError::invalid("kafkax: 当前 rskafka 构建未接入 TLS，拒绝静默降级为明文"));
        }
        if self.sasl_mechanism.is_some() {
            if self.sasl_username.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(XError::invalid("kafkax: 已启用 SASL 但缺少 username"));
            }
            if self.sasl_password.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(XError::invalid("kafkax: 已启用 SASL 但缺少 password"));
            }
        }
        Ok(())
    }

    /// 安全协议字符串。
    pub fn security_protocol(&self) -> &'static str {
        match (self.tls, self.sasl_mechanism.is_some()) {
            (true, true) => "SASL_SSL",
            (true, false) => "SSL",
            (false, true) => "SASL_PLAINTEXT",
            (false, false) => "PLAINTEXT",
        }
    }
}

fn parse_bool(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_embedded_credentials() {
        let c = KafkaConfig::default();
        assert_eq!(c.brokers, DEFAULT_BROKERS);
        assert!(c.sasl_mechanism.is_none());
        assert!(c.sasl_username.is_none());
        assert!(c.sasl_password.is_none());
        assert!(!c.tls);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn debug_redacts_password() {
        let c = KafkaConfig {
            sasl_mechanism: Some(DEFAULT_SASL_MECHANISM.into()),
            sasl_username: Some("admin".into()),
            sasl_password: Some("super-secret-kafka".into()),
            ..KafkaConfig::default()
        };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("super-secret-kafka"));
    }

    #[test]
    fn security_protocol_matrix() {
        let mut c = KafkaConfig::default();
        assert_eq!(c.security_protocol(), "PLAINTEXT");
        c.sasl_mechanism = Some(DEFAULT_SASL_MECHANISM.into());
        assert_eq!(c.security_protocol(), "SASL_PLAINTEXT");
        c.tls = true;
        assert_eq!(c.security_protocol(), "SASL_SSL");
        c.sasl_mechanism = None;
        assert_eq!(c.security_protocol(), "SSL");
        c.tls = false;
        assert_eq!(c.security_protocol(), "PLAINTEXT");
    }

    #[test]
    fn tls_fails_closed_until_transport_is_implemented() {
        let cfg = KafkaConfig { tls: true, ..KafkaConfig::default() };
        let error = cfg.validate().expect_err("不得静默忽略 TLS");
        assert!(error.context().contains("未接入 TLS"));
    }
}
