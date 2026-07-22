//! Kafka 配置：环境变量与本地默认值。
//!
//! 环境变量（`FOUNDATIONX_KAFKAX_*`）：
//! - `BROKERS` — bootstrap servers（逗号分隔）
//! - `SASL_MECHANISM` — 如 `PLAIN`；空串关闭 SASL
//! - `SASL_USERNAME` / `SASL_PASSWORD` — SASL 凭据
//! - `TLS` — `1`/`true`/`yes` 开启 TLS（`SASL_SSL` 或 `SSL`）
//! - `CONNECT_TIMEOUT_MS` / `OPERATION_TIMEOUT_MS` — 连接与元数据操作截止时间
//!
//! **默认值面向本地/草稿联调**，生产环境务必通过环境变量注入凭据；
//! 本类型的 `Debug` 会脱敏密码。

use std::fmt;
use std::path::PathBuf;
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
    /// 可选 PEM CA 文件；未设置时使用公开 webpki roots。
    pub tls_ca_file: Option<PathBuf>,
    /// 投递等待超时。
    pub delivery_timeout: Duration,
    /// 建连截止时间。
    pub connect_timeout: Duration,
    /// 元数据、分区客户端和管理操作截止时间。
    pub operation_timeout: Duration,
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
            tls_ca_file: None,
            delivery_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            operation_timeout: Duration::from_secs(10),
            event_bus_group_prefix: "kafkax-eventbus".to_string(),
        }
    }
}

impl fmt::Debug for KafkaConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KafkaConfig")
            .field("brokers", &redact_brokers(&self.brokers))
            .field("client_id", &self.client_id)
            .field("sasl_mechanism", &self.sasl_mechanism)
            .field("sasl_username", &self.sasl_username)
            .field("sasl_password", &self.sasl_password.as_ref().map(|_| "***"))
            .field("tls", &self.tls)
            .field("tls_ca_file", &self.tls_ca_file)
            .field("delivery_timeout", &self.delivery_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("operation_timeout", &self.operation_timeout)
            .field("event_bus_group_prefix", &self.event_bus_group_prefix)
            .finish()
    }
}

impl KafkaConfig {
    /// 从环境变量加载，缺省回落 [`KafkaConfig::default`]。
    pub fn from_env() -> XResult<Self> {
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
            cfg.tls = parse_bool(&v, "FOUNDATIONX_KAFKAX_TLS")?;
        }
        if let Ok(value) = std::env::var("FOUNDATIONX_KAFKAX_TLS_CA_FILE") {
            if !value.trim().is_empty() {
                cfg.tls_ca_file = Some(PathBuf::from(value));
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_CLIENT_ID") {
            if !v.trim().is_empty() {
                cfg.client_id = v;
            }
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_CONNECT_TIMEOUT_MS") {
            cfg.connect_timeout = Duration::from_millis(v.parse::<u64>().map_err(|error| {
                XError::invalid(format!("FOUNDATIONX_KAFKAX_CONNECT_TIMEOUT_MS 非法: {error}"))
            })?);
        }
        if let Ok(v) = std::env::var("FOUNDATIONX_KAFKAX_OPERATION_TIMEOUT_MS") {
            cfg.operation_timeout = Duration::from_millis(v.parse::<u64>().map_err(|error| {
                XError::invalid(format!("FOUNDATIONX_KAFKAX_OPERATION_TIMEOUT_MS 非法: {error}"))
            })?);
        }
        cfg.validate()?;
        Ok(cfg)
    }

    /// 校验配置完整性。
    ///
    /// # Errors
    ///
    /// broker/截止时间非法、远程明文、未知 SASL 机制或凭据不完整时返回 `Invalid`。
    pub fn validate(&self) -> XResult<()> {
        if self.brokers.trim().is_empty() {
            return Err(XError::invalid("kafkax: brokers 不能为空"));
        }
        if self.delivery_timeout.is_zero()
            || self.connect_timeout.is_zero()
            || self.operation_timeout.is_zero()
        {
            return Err(XError::invalid("kafkax: timeout 必须大于零"));
        }
        if self.tls_ca_file.is_some() && !self.tls {
            return Err(XError::invalid("kafkax: 配置 TLS_CA_FILE 时必须启用 TLS"));
        }
        let brokers = self.brokers.split(',').map(str::trim).filter(|broker| !broker.is_empty());
        for broker in brokers {
            let host = broker_host(broker)?;
            if !self.tls && !host_is_loopback(&host) {
                return Err(XError::invalid(format!("kafkax: 远程 broker `{host}` 必须启用 TLS")));
            }
        }
        if let Some(mechanism) = &self.sasl_mechanism {
            if !mechanism.eq_ignore_ascii_case(DEFAULT_SASL_MECHANISM) {
                return Err(XError::invalid(format!(
                    "kafkax: 当前仅支持 SASL/PLAIN，拒绝机制 `{mechanism}`"
                )));
            }
            if self.sasl_username.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(XError::invalid("kafkax: 已启用 SASL 但缺少 username"));
            }
            if self.sasl_password.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(XError::invalid("kafkax: 已启用 SASL 但缺少 password"));
            }
        } else if self.sasl_username.is_some() || self.sasl_password.is_some() {
            return Err(XError::invalid("kafkax: 提供了 SASL 凭据但未启用 PLAIN 机制"));
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

fn broker_host(broker: &str) -> XResult<String> {
    let candidate =
        if broker.contains("://") { broker.to_string() } else { format!("kafka://{broker}") };
    let parsed = url::Url::parse(&candidate)
        .map_err(|error| XError::invalid(format!("kafkax: broker 地址非法: {error}")))?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(XError::invalid("kafkax: broker 地址禁止内嵌 userinfo"));
    }
    parsed
        .host_str()
        .filter(|host| !host.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| XError::invalid("kafkax: broker 缺少 host"))
}

fn redact_brokers(brokers: &str) -> String {
    brokers
        .split(',')
        .map(|broker| if broker.contains('@') { "<redacted-userinfo>" } else { broker.trim() })
        .collect::<Vec<_>>()
        .join(",")
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
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
            brokers: "kafka://embedded:secret@localhost:9092".into(),
            sasl_mechanism: Some(DEFAULT_SASL_MECHANISM.into()),
            sasl_username: Some("admin".into()),
            sasl_password: Some("super-secret-kafka".into()),
            ..KafkaConfig::default()
        };
        let s = format!("{c:?}");
        assert!(s.contains("***"));
        assert!(!s.contains("super-secret-kafka"));
        assert!(!s.contains("embedded"));
        assert!(!s.contains("secret@"));
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
    fn tls_allows_remote_and_plaintext_rejects_remote() {
        let tls = KafkaConfig {
            brokers: "broker.example.com:9093".into(),
            tls: true,
            ..KafkaConfig::default()
        };
        tls.validate().expect("远程 TLS 配置应通过");

        let plain =
            KafkaConfig { brokers: "broker.example.com:9092".into(), ..KafkaConfig::default() };
        let error = plain.validate().expect_err("远程明文必须 fail-closed");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        assert!(error.context().contains("必须启用 TLS"));
    }

    #[test]
    fn only_plain_sasl_is_accepted_and_half_credentials_fail() {
        let unknown = KafkaConfig {
            sasl_mechanism: Some("SCRAM-SHA-256".into()),
            sasl_username: Some("user".into()),
            sasl_password: Some("secret".into()),
            ..KafkaConfig::default()
        };
        assert_eq!(
            unknown.validate().expect_err("未知机制必须失败").kind(),
            kernel::ErrorKind::Invalid
        );

        let credentials_without_mechanism = KafkaConfig {
            sasl_username: Some("user".into()),
            sasl_password: Some("secret".into()),
            ..KafkaConfig::default()
        };
        assert_eq!(
            credentials_without_mechanism.validate().expect_err("凭据不得静默忽略").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn ca_file_requires_tls() {
        let config =
            KafkaConfig { tls_ca_file: Some("/tmp/kafka-ca.pem".into()), ..KafkaConfig::default() };
        assert_eq!(
            config.validate().expect_err("CA 不得用于明文").kind(),
            kernel::ErrorKind::Invalid
        );
    }

    #[test]
    fn rejects_zero_deadlines_and_accepts_ipv6_loopback() {
        let zero = KafkaConfig { connect_timeout: Duration::ZERO, ..KafkaConfig::default() };
        assert_eq!(zero.validate().expect_err("零超时必须失败").kind(), kernel::ErrorKind::Invalid);

        let ipv6 = KafkaConfig { brokers: "[::1]:9092".into(), ..KafkaConfig::default() };
        ipv6.validate().expect("IPv6 loopback 明文可用于本机实验");
    }
}
