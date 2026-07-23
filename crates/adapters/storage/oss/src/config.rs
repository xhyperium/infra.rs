//! OSS 连接配置（环境变量 / Builder）。

use std::env;
use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

use kernel::{XError, XResult};
use url::Url;

/// 环境变量前缀（与 foundationx live 约定一致）。
pub const ENV_ENDPOINT: &str = "FOUNDATIONX_OSSX_ENDPOINT";
pub const ENV_BUCKET: &str = "FOUNDATIONX_OSSX_BUCKET";
pub const ENV_ACCESS_KEY_ID: &str = "FOUNDATIONX_OSSX_ACCESS_KEY_ID";
pub const ENV_ACCESS_KEY_SECRET: &str = "FOUNDATIONX_OSSX_ACCESS_KEY_SECRET";
pub const ENV_REGION: &str = "FOUNDATIONX_OSSX_REGION";
/// 单请求超时环境变量（毫秒）。
pub const ENV_REQUEST_TIMEOUT_MS: &str = "FOUNDATIONX_OSSX_REQUEST_TIMEOUT_MS";
/// 含重试在内的单操作 deadline 环境变量（毫秒）。
pub const ENV_OPERATION_DEADLINE_MS: &str = "FOUNDATIONX_OSSX_OPERATION_DEADLINE_MS";
/// 获取并发许可超时环境变量（毫秒）。
pub const ENV_ACQUIRE_TIMEOUT_MS: &str = "FOUNDATIONX_OSSX_ACQUIRE_TIMEOUT_MS";
/// 最大并发请求数环境变量。
pub const ENV_MAX_IN_FLIGHT: &str = "FOUNDATIONX_OSSX_MAX_IN_FLIGHT";
/// 最大对象字节数环境变量。
pub const ENV_MAX_OBJECT_BYTES: &str = "FOUNDATIONX_OSSX_MAX_OBJECT_BYTES";
/// 最大单次内存缓冲字节数环境变量。
pub const ENV_MAX_BUFFER_BYTES: &str = "FOUNDATIONX_OSSX_MAX_BUFFER_BYTES";
/// 最大错误响应体字节数环境变量。
pub const ENV_MAX_ERROR_BODY_BYTES: &str = "FOUNDATIONX_OSSX_MAX_ERROR_BODY_BYTES";

/// 配置允许的最大并发请求数硬上界。
pub const HARD_MAX_IN_FLIGHT: usize = 1_024;
/// 配置允许的最大对象字节数硬上界（5 GiB）。
pub const HARD_MAX_OBJECT_BYTES: usize = 5 * 1024 * 1024 * 1024;
/// 配置允许的最大内存缓冲字节数硬上界（512 MiB）。
pub const HARD_MAX_BUFFER_BYTES: usize = 512 * 1024 * 1024;
/// 配置允许的最大错误响应体字节数硬上界（1 MiB）。
pub const HARD_MAX_ERROR_BODY_BYTES: usize = 1024 * 1024;

const DEFAULT_MAX_IN_FLIGHT: usize = 64;
const DEFAULT_MAX_OBJECT_BYTES: usize = HARD_MAX_BUFFER_BYTES;
const DEFAULT_MAX_BUFFER_BYTES: usize = HARD_MAX_BUFFER_BYTES;
const DEFAULT_MAX_ERROR_BODY_BYTES: usize = 64 * 1024;
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_OPERATION_DEADLINE: Duration = Duration::from_secs(90);
const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

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
    /// 单请求超时。
    pub request_timeout: Duration,
    /// 含重试在内的单操作 deadline。
    pub operation_deadline: Duration,
    /// 获取 in-flight 许可超时。
    pub acquire_timeout: Duration,
    /// 全局 in-flight 请求上限。
    pub max_in_flight: usize,
    /// 对象大小上限。
    pub max_object_bytes: usize,
    /// 单次内存缓冲上限。
    pub max_buffer_bytes: usize,
    /// 错误响应体读取上限。
    pub max_error_body_bytes: usize,
    /// STS 临时安全令牌（可选）。
    pub security_token: Option<String>,
    /// 是否启用 SSE-S3 服务端加密。
    pub sse_enabled: bool,
}

impl fmt::Debug for OssConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OssConfig")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key_id", &redact_mid(&self.access_key_id))
            .field("access_key_secret", &"<redacted>")
            .field("region", &self.region)
            .field("request_timeout", &self.request_timeout)
            .field("operation_deadline", &self.operation_deadline)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("max_in_flight", &self.max_in_flight)
            .field("max_object_bytes", &self.max_object_bytes)
            .field("max_buffer_bytes", &self.max_buffer_bytes)
            .field("max_error_body_bytes", &self.max_error_body_bytes)
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
        let region = match env::var(ENV_REGION) {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "ap-northeast-1".into(),
            Err(error) => {
                return Err(XError::invalid(format!("env {ENV_REGION} 非法")).with_source(error));
            }
        };
        let mut builder = Self::builder()
            .endpoint(endpoint)
            .bucket(bucket)
            .access_key_id(access_key_id)
            .access_key_secret(access_key_secret)
            .region(region);
        if let Some(value) = optional_usize_env(ENV_MAX_IN_FLIGHT)? {
            builder = builder.max_in_flight(value);
        }
        if let Some(value) = optional_usize_env(ENV_MAX_OBJECT_BYTES)? {
            builder = builder.max_object_bytes(value);
        }
        if let Some(value) = optional_usize_env(ENV_MAX_BUFFER_BYTES)? {
            builder = builder.max_buffer_bytes(value);
        }
        if let Some(value) = optional_usize_env(ENV_MAX_ERROR_BODY_BYTES)? {
            builder = builder.max_error_body_bytes(value);
        }
        if let Some(value) = optional_duration_env(ENV_REQUEST_TIMEOUT_MS)? {
            builder = builder.request_timeout(value);
        }
        if let Some(value) = optional_duration_env(ENV_OPERATION_DEADLINE_MS)? {
            builder = builder.operation_deadline(value);
        }
        if let Some(value) = optional_duration_env(ENV_ACQUIRE_TIMEOUT_MS)? {
            builder = builder.acquire_timeout(value);
        }
        builder.build()
    }

    /// Builder 入口。
    #[must_use]
    pub fn builder() -> OssConfigBuilder {
        OssConfigBuilder::default()
    }

    /// 校验传输安全与所有资源硬上界。
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
        let endpoint = Url::parse(&self.endpoint)
            .map_err(|error| XError::invalid(format!("oss endpoint URL 非法: {error}")))?;
        let host = endpoint.host_str().ok_or_else(|| XError::invalid("oss endpoint 缺少 host"))?;
        if endpoint.scheme() != "https" && !(endpoint.scheme() == "http" && host_is_loopback(host))
        {
            return Err(XError::invalid("远程 OSS endpoint 必须使用 HTTPS"));
        }
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !matches!(endpoint.path(), "" | "/")
        {
            return Err(XError::invalid("oss endpoint 禁止 userinfo、path、query 或 fragment"));
        }
        if !valid_bucket(&self.bucket) {
            return Err(XError::invalid("oss bucket 只能包含小写字母、数字和连字符"));
        }
        if self.request_timeout.is_zero()
            || self.operation_deadline.is_zero()
            || self.acquire_timeout.is_zero()
        {
            return Err(XError::invalid("oss timeout/deadline 必须大于零"));
        }
        if self.operation_deadline < self.request_timeout {
            return Err(XError::invalid("operation_deadline 不得小于 request_timeout"));
        }
        validate_limit("max_in_flight", self.max_in_flight, HARD_MAX_IN_FLIGHT)?;
        validate_limit("max_object_bytes", self.max_object_bytes, HARD_MAX_OBJECT_BYTES)?;
        validate_limit("max_buffer_bytes", self.max_buffer_bytes, HARD_MAX_BUFFER_BYTES)?;
        validate_limit(
            "max_error_body_bytes",
            self.max_error_body_bytes,
            HARD_MAX_ERROR_BODY_BYTES,
        )?;
        if self.max_object_bytes > self.max_buffer_bytes {
            return Err(XError::invalid(
                "当前 Bytes API 要求 max_object_bytes 不得超过 max_buffer_bytes",
            ));
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
    request_timeout: Option<Duration>,
    operation_deadline: Option<Duration>,
    acquire_timeout: Option<Duration>,
    max_in_flight: Option<usize>,
    max_object_bytes: Option<usize>,
    max_buffer_bytes: Option<usize>,
    max_error_body_bytes: Option<usize>,
    security_token: Option<String>,
    sse_enabled: Option<bool>,
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

    /// 设置单请求超时。
    #[must_use]
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = Some(value);
        self
    }

    /// 设置含重试在内的单操作 deadline。
    #[must_use]
    pub fn operation_deadline(mut self, value: Duration) -> Self {
        self.operation_deadline = Some(value);
        self
    }

    /// 设置获取 in-flight 许可超时。
    #[must_use]
    pub fn acquire_timeout(mut self, value: Duration) -> Self {
        self.acquire_timeout = Some(value);
        self
    }

    /// 设置全局 in-flight 上限。
    #[must_use]
    pub fn max_in_flight(mut self, value: usize) -> Self {
        self.max_in_flight = Some(value);
        self
    }

    /// 设置对象大小上限。
    #[must_use]
    pub fn max_object_bytes(mut self, value: usize) -> Self {
        self.max_object_bytes = Some(value);
        self
    }

    /// 设置单次内存缓冲上限。
    #[must_use]
    pub fn max_buffer_bytes(mut self, value: usize) -> Self {
        self.max_buffer_bytes = Some(value);
        self
    }

    /// 设置错误响应体读取上限。
    #[must_use]
    pub fn max_error_body_bytes(mut self, value: usize) -> Self {
        self.max_error_body_bytes = Some(value);
        self
    }

    /// 设置 STS 临时安全令牌（可选）。
    #[must_use]
    pub fn security_token(mut self, value: impl Into<String>) -> Self {
        self.security_token = Some(value.into());
        self
    }

    /// 设置是否启用 SSE-S3 加密（默认 false）。
    #[must_use]
    pub fn sse_enabled(mut self, value: bool) -> Self {
        self.sse_enabled = Some(value);
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
            request_timeout: self.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
            operation_deadline: self.operation_deadline.unwrap_or(DEFAULT_OPERATION_DEADLINE),
            acquire_timeout: self.acquire_timeout.unwrap_or(DEFAULT_ACQUIRE_TIMEOUT),
            max_in_flight: self.max_in_flight.unwrap_or(DEFAULT_MAX_IN_FLIGHT),
            max_object_bytes: self.max_object_bytes.unwrap_or(DEFAULT_MAX_OBJECT_BYTES),
            max_buffer_bytes: self.max_buffer_bytes.unwrap_or(DEFAULT_MAX_BUFFER_BYTES),
            max_error_body_bytes: self.max_error_body_bytes.unwrap_or(DEFAULT_MAX_ERROR_BODY_BYTES),
            security_token: self.security_token,
            sse_enabled: self.sse_enabled.unwrap_or(false),
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

fn require_env(key: &str) -> XResult<String> {
    env::var(key).map_err(|_| XError::missing(format!("env {key} not set")))
}

fn optional_usize_env(key: &str) -> XResult<Option<usize>> {
    match env::var(key) {
        Ok(value) => value
            .parse::<usize>()
            .map(Some)
            .map_err(|error| XError::invalid(format!("env {key} 非法")).with_source(error)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(XError::invalid(format!("env {key} 非法")).with_source(error)),
    }
}

fn optional_duration_env(key: &str) -> XResult<Option<Duration>> {
    optional_usize_env(key)?
        .map(|value| {
            u64::try_from(value)
                .map(Duration::from_millis)
                .map_err(|error| XError::invalid(format!("env {key} 过大")).with_source(error))
        })
        .transpose()
}

fn validate_limit(name: &str, value: usize, hard_max: usize) -> XResult<()> {
    if value == 0 || value > hard_max {
        return Err(XError::invalid(format!("{name} 必须在 1..={hard_max} 范围内")));
    }
    Ok(())
}

fn host_is_loopback(host: &str) -> bool {
    let host = host.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(host);
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|address| address.is_loopback())
}

fn valid_bucket(bucket: &str) -> bool {
    let bytes = bucket.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 63
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
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

    #[test]
    fn remote_plaintext_endpoint_fails_closed() {
        let error = OssConfig::builder()
            .endpoint("http://oss.example.com")
            .bucket("example-bucket")
            .access_key_id("id")
            .access_key_secret("secret")
            .build()
            .expect_err("远程 HTTP 必须 fail-closed");
        assert!(error.context().contains("HTTPS"));

        OssConfig::builder()
            .endpoint("http://127.0.0.1:9000")
            .bucket("example-bucket")
            .access_key_id("id")
            .access_key_secret("secret")
            .build()
            .expect("loopback 开发端点可使用 HTTP");
    }

    #[test]
    fn resource_limits_reject_zero_or_excessive_values() {
        let error = OssConfig::builder()
            .endpoint("https://oss.example.com")
            .bucket("example-bucket")
            .access_key_id("id")
            .access_key_secret("secret")
            .max_in_flight(0)
            .build()
            .expect_err("零并发必须被拒绝");
        assert!(error.context().contains("max_in_flight"));

        let error = OssConfig::builder()
            .endpoint("https://oss.example.com")
            .bucket("example-bucket")
            .access_key_id("id")
            .access_key_secret("secret")
            .max_error_body_bytes(HARD_MAX_ERROR_BODY_BYTES + 1)
            .build()
            .expect_err("错误体上限不得超过硬上界");
        assert!(error.context().contains("max_error_body_bytes"));
    }
}
