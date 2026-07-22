//! HTTP 代理配置（密码 Debug 脱敏）。

use std::fmt;

/// 代理配置。
#[derive(Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    /// 代理 URL（如 `http://proxy.local:8080`）。
    pub url: String,
    /// 可选用户名。
    pub username: Option<String>,
    /// 可选密码（Debug 脱敏）。
    pub password: Option<String>,
}

impl ProxyConfig {
    /// 仅 URL。
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into(), username: None, password: None }
    }

    /// 带基本认证。
    #[must_use]
    pub fn with_auth(
        url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self { url: url.into(), username: Some(username.into()), password: Some(password.into()) }
    }

    /// 是否配置了认证。
    #[must_use]
    pub fn has_auth(&self) -> bool {
        self.username.is_some() || self.password.is_some()
    }
}

impl fmt::Debug for ProxyConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProxyConfig")
            .field("url", &self.url)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .finish()
    }
}

/// 将 [`ProxyConfig`] 转为 reqwest `Proxy`。
pub fn build_reqwest_proxy(cfg: &ProxyConfig) -> Result<reqwest::Proxy, crate::TransportError> {
    let mut proxy = reqwest::Proxy::all(&cfg.url)
        .map_err(|e| crate::TransportError::ProtocolViolation(format!("invalid proxy url: {e}")))?;
    if let (Some(u), Some(p)) = (&cfg.username, &cfg.password) {
        proxy = proxy.basic_auth(u, p);
    }
    Ok(proxy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_debug_redacts_password() {
        let p = ProxyConfig::with_auth("http://proxy:1", "u", "super-secret");
        let d = format!("{p:?}");
        assert!(d.contains("***"));
        assert!(!d.contains("super-secret"));
        assert!(p.has_auth());
        let plain = ProxyConfig::new("http://proxy:1");
        assert!(!plain.has_auth());
    }
}
