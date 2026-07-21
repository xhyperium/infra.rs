//! NATS 配置：环境变量与本地默认值。
//!
//! 环境变量（canonical `FOUNDATIONX_NATS_*`，兼容 `FOUNDATIONX_NATSX_*`）：
//! - `URL` / `SERVERS`
//! - `USER` / `USERNAME`
//! - `PASSWORD`
//!
//! **默认值面向本地/草稿联调**；生产必须通过环境注入。`Debug` 脱敏密码。

use std::fmt;
use std::time::Duration;

use kernel::{XError, XResult};

/// 默认 URL（无认证；生产凭据必须经环境注入）。
pub const DEFAULT_URL: &str = "nats://127.0.0.1:4222";

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
                // servers 列表 → 取第一项或原样（async-nats 接受逗号？我们拆第一项为主 URL）
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
        cfg
    }

    /// 校验。
    pub fn validate(&self) -> XResult<()> {
        if self.url.trim().is_empty() {
            return Err(XError::invalid("natsx: url 不能为空"));
        }
        match (&self.user, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => Ok(()),
            (None, None) => Ok(()),
            _ => Err(XError::invalid("natsx: user/password 必须同时提供或同时缺省")),
        }
    }
}

fn env_first(keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            return Some(v);
        }
    }
    None
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
        assert!(c.validate().is_ok());
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
}
