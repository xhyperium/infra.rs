//! OSS 连接配置（环境变量 / Builder）。

use std::env;
use std::fmt;

use kernel::{XError, XResult};

/// 环境变量前缀（与 foundationx live 约定一致）。
pub const ENV_ENDPOINT: &str = "FOUNDATIONX_OSSX_ENDPOINT";
pub const ENV_BUCKET: &str = "FOUNDATIONX_OSSX_BUCKET";
pub const ENV_ACCESS_KEY_ID: &str = "FOUNDATIONX_OSSX_ACCESS_KEY_ID";
pub const ENV_ACCESS_KEY_SECRET: &str = "FOUNDATIONX_OSSX_ACCESS_KEY_SECRET";
pub const ENV_REGION: &str = "FOUNDATIONX_OSSX_REGION";

/// 阿里云 OSS 配置。
///
/// `Debug` 会脱敏密钥；切勿在日志中打印原始 secret。
#[derive(Clone)]
pub struct OssConfig {
    /// 地域 endpoint，例如 `https://oss-ap-northeast-1.aliyuncs.com`。
    pub endpoint: String,
    /// Bucket 名称。
    pub bucket: String,
    /// AccessKeyId。
    pub access_key_id: String,
    /// AccessKeySecret（敏感）。
    pub access_key_secret: String,
    /// 区域 id（元数据；签名 V1 不强制使用）。
    pub region: String,
}

impl fmt::Debug for OssConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OssConfig")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key_id", &redact_mid(&self.access_key_id))
            .field("access_key_secret", &"<redacted>")
            .field("region", &self.region)
            .finish()
    }
}

impl OssConfig {
    /// 从 `FOUNDATIONX_OSSX_*` 环境变量加载；缺项 fail-closed。
    pub fn from_env() -> XResult<Self> {
        let endpoint = require_env(ENV_ENDPOINT)?;
        let bucket = require_env(ENV_BUCKET)?;
        let access_key_id = require_env(ENV_ACCESS_KEY_ID)?;
        let access_key_secret = require_env(ENV_ACCESS_KEY_SECRET)?;
        let region = env::var(ENV_REGION).unwrap_or_else(|_| "ap-northeast-1".into());
        Self::builder()
            .endpoint(endpoint)
            .bucket(bucket)
            .access_key_id(access_key_id)
            .access_key_secret(access_key_secret)
            .region(region)
            .build()
    }

    /// Builder 入口。
    #[must_use]
    pub fn builder() -> OssConfigBuilder {
        OssConfigBuilder::default()
    }

    /// 校验非空字段。
    pub fn validate(&self) -> XResult<()> {
        for (name, v) in [
            ("endpoint", self.endpoint.as_str()),
            ("bucket", self.bucket.as_str()),
            ("access_key_id", self.access_key_id.as_str()),
            ("access_key_secret", self.access_key_secret.as_str()),
        ] {
            if v.trim().is_empty() {
                return Err(XError::invalid(format!("oss config {name} is empty")));
            }
        }
        if !(self.endpoint.starts_with("https://") || self.endpoint.starts_with("http://")) {
            return Err(XError::invalid("oss endpoint must be http(s) URL"));
        }
        Ok(())
    }
}

/// `OssConfig` 构建器。
#[derive(Debug, Default, Clone)]
pub struct OssConfigBuilder {
    endpoint: Option<String>,
    bucket: Option<String>,
    access_key_id: Option<String>,
    access_key_secret: Option<String>,
    region: Option<String>,
}

impl OssConfigBuilder {
    #[must_use]
    pub fn endpoint(mut self, v: impl Into<String>) -> Self {
        self.endpoint = Some(v.into());
        self
    }

    #[must_use]
    pub fn bucket(mut self, v: impl Into<String>) -> Self {
        self.bucket = Some(v.into());
        self
    }

    #[must_use]
    pub fn access_key_id(mut self, v: impl Into<String>) -> Self {
        self.access_key_id = Some(v.into());
        self
    }

    #[must_use]
    pub fn access_key_secret(mut self, v: impl Into<String>) -> Self {
        self.access_key_secret = Some(v.into());
        self
    }

    #[must_use]
    pub fn region(mut self, v: impl Into<String>) -> Self {
        self.region = Some(v.into());
        self
    }

    /// 构建并校验。
    pub fn build(self) -> XResult<OssConfig> {
        let cfg = OssConfig {
            endpoint: self.endpoint.ok_or_else(|| XError::invalid("oss endpoint required"))?,
            bucket: self.bucket.ok_or_else(|| XError::invalid("oss bucket required"))?,
            access_key_id: self
                .access_key_id
                .ok_or_else(|| XError::invalid("oss access_key_id required"))?,
            access_key_secret: self
                .access_key_secret
                .ok_or_else(|| XError::invalid("oss access_key_secret required"))?,
            region: self.region.unwrap_or_else(|| "ap-northeast-1".into()),
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

fn require_env(key: &str) -> XResult<String> {
    env::var(key).map_err(|_| XError::missing(format!("env {key} not set")))
}

/// 中间脱敏（保留首尾少量字符）。
fn redact_mid(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 6 {
        return "***".into();
    }
    format!(
        "{}***{}",
        chars[..3].iter().collect::<String>(),
        chars[chars.len() - 2..].iter().collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_redacts_secret() {
        let cfg = OssConfig::builder()
            .endpoint("https://oss-ap-northeast-1.aliyuncs.com")
            .bucket("demo")
            .access_key_id("LTAI5tABCDEFGH")
            .access_key_secret("super-secret-value")
            .region("ap-northeast-1")
            .build()
            .expect("cfg");
        let d = format!("{cfg:?}");
        assert!(d.contains("<redacted>"));
        assert!(!d.contains("super-secret-value"));
        assert!(!d.contains("LTAI5tABCDEFGH"));
    }

    #[test]
    fn builder_requires_fields() {
        let err = OssConfig::builder().build().unwrap_err();
        assert!(format!("{err}").contains("endpoint"));
    }

    /// 覆盖 crate-root 导出的 `ENV_*` 常量与 `OssConfigBuilder` 类型名。
    #[test]
    fn env_constants_and_builder_type() {
        assert_eq!(ENV_ENDPOINT, "FOUNDATIONX_OSSX_ENDPOINT");
        assert_eq!(ENV_BUCKET, "FOUNDATIONX_OSSX_BUCKET");
        assert_eq!(ENV_ACCESS_KEY_ID, "FOUNDATIONX_OSSX_ACCESS_KEY_ID");
        assert_eq!(ENV_ACCESS_KEY_SECRET, "FOUNDATIONX_OSSX_ACCESS_KEY_SECRET");
        assert_eq!(ENV_REGION, "FOUNDATIONX_OSSX_REGION");

        let builder: OssConfigBuilder = OssConfig::builder();
        let cfg: OssConfig = builder
            .endpoint("https://oss-cn-hangzhou.aliyuncs.com")
            .bucket("b")
            .access_key_id("id")
            .access_key_secret("secret")
            .region("cn-hangzhou")
            .build()
            .expect("cfg");
        cfg.validate().expect("validate");
        assert_eq!(cfg.bucket, "b");
    }
}
