use crate::sign;
use kernel::error::{XError, XResult};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct PresignOptions {
    pub method: String,
    pub expires: Duration,
    pub content_type: Option<String>,
}
impl Default for PresignOptions {
    fn default() -> Self {
        Self { method: "GET".into(), expires: Duration::from_secs(3600), content_type: None }
    }
}

pub fn presign_url(
    ep: &str,
    b: &str,
    k: &str,
    id: &str,
    sec: &str,
    o: &PresignOptions,
) -> XResult<String> {
    let host = ep
        .trim_end_matches('/')
        .strip_prefix("https://")
        .ok_or_else(|| XError::invalid("HTTPS required"))?;
    let expires = chrono::Utc::now().timestamp() + o.expires.as_secs() as i64;
    let r = sign::canonicalized_resource(b, k);
    let sig = sign::sign_v1(
        sec,
        &o.method,
        "",
        o.content_type.as_deref().unwrap_or(""),
        &expires.to_string(),
        "",
        &r,
    );
    let enc = sig.replace('+', "%2B").replace('/', "%2F");
    Ok(format!("https://{b}.{host}/{k}?OSSAccessKeyId={id}&Expires={expires}&Signature={enc}"))
}
