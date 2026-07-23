use async_trait::async_trait;
use kernel::error::XResult;

#[derive(Clone, Debug)]
pub struct OssCredentials {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: Option<String>,
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
