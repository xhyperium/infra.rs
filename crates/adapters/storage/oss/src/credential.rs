use async_trait::async_trait;
use kernel::error::XResult;

#[derive(Clone)]
pub struct OssCredentials {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: Option<String>,
}

impl std::fmt::Debug for OssCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OssCredentials")
            .field("access_key_id", &self.access_key_id)
            .field("access_key_secret", &"<redacted>")
            .field("security_token", &self.security_token.as_ref().map(|_| "<present>"))
            .finish()
    }
}

#[async_trait]
pub trait CredentialProvider: Send + Sync {
    async fn get_credentials(&self) -> XResult<OssCredentials>;
    fn provider_name(&self) -> &'static str;
}

pub struct StaticCredentialProvider {
    creds: OssCredentials,
}

impl StaticCredentialProvider {
    pub fn new(id: String, sec: String, tok: Option<String>) -> Self {
        Self {
            creds: OssCredentials {
                access_key_id: id,
                access_key_secret: sec,
                security_token: tok,
            },
        }
    }
}

#[async_trait]
impl CredentialProvider for StaticCredentialProvider {
    async fn get_credentials(&self) -> XResult<OssCredentials> {
        Ok(self.creds.clone())
    }
    fn provider_name(&self) -> &'static str {
        "static"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_provider_returns_credentials() {
        let provider = StaticCredentialProvider::new("test-id".into(), "test-secret".into(), None);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let creds = rt.block_on(provider.get_credentials()).unwrap();
        assert_eq!(creds.access_key_id, "test-id");
        assert_eq!(creds.access_key_secret, "test-secret");
        assert!(creds.security_token.is_none());
    }

    #[test]
    fn static_provider_with_security_token() {
        let provider =
            StaticCredentialProvider::new("id".into(), "sec".into(), Some("sts-token".into()));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let creds = rt.block_on(provider.get_credentials()).unwrap();
        assert_eq!(creds.security_token.as_deref(), Some("sts-token"));
    }

    #[test]
    fn static_provider_name() {
        let provider = StaticCredentialProvider::new("id".into(), "sec".into(), None);
        assert_eq!(provider.provider_name(), "static");
    }

    #[test]
    fn oss_credentials_debug_does_not_leak_secret() {
        let creds = OssCredentials {
            access_key_id: "AK123".into(),
            access_key_secret: "super-secret".into(),
            security_token: None,
        };
        let debug = format!("{creds:?}");
        assert!(debug.contains("AK123"), "id should be visible");
        assert!(!debug.contains("super-secret"), "secret MUST NOT be leaked");
    }
}
