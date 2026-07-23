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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presign_url_basic_format() {
        let url = presign_url(
            "https://oss.example.com",
            "test-bucket",
            "test-key",
            "test-id",
            "test-secret",
            &PresignOptions::default(),
        )
        .unwrap();
        assert!(url.starts_with("https://test-bucket.oss.example.com/test-key?"));
        assert!(url.contains("OSSAccessKeyId=test-id"));
        assert!(url.contains("&Expires="));
        assert!(url.contains("&Signature="));
    }

    #[test]
    fn presign_url_put_method() {
        let opts = PresignOptions {
            method: "PUT".into(),
            expires: Duration::from_secs(600),
            content_type: Some("application/octet-stream".into()),
        };
        let url = presign_url("https://oss.example.com", "b", "k", "id", "sec", &opts).unwrap();
        assert!(url.contains("OSSAccessKeyId=id"));
        assert!(url.contains("&Signature="));
    }

    #[test]
    fn presign_url_rejects_http() {
        let result = presign_url(
            "http://oss.example.com",
            "b",
            "k",
            "id",
            "sec",
            &PresignOptions::default(),
        );
        assert!(result.is_err(), "HTTP should be rejected");
    }

    #[test]
    fn presign_url_deterministic() {
        let opts = PresignOptions {
            expires: Duration::from_secs(9999), // fixed for deterministic test
            ..Default::default()
        };
        let u1 = presign_url("https://oss.example.com", "b", "k", "id", "sec", &opts).unwrap();
        let u2 = presign_url("https://oss.example.com", "b", "k", "id", "sec", &opts).unwrap();
        // Same inputs same outputs (except Expires which is absolute timestamp)
        assert_eq!(u1, u2, "same inputs should produce same URL");
    }

    #[test]
    fn presign_options_defaults() {
        let opts = PresignOptions::default();
        assert_eq!(opts.method, "GET");
        assert_eq!(opts.expires, Duration::from_secs(3600));
        assert!(opts.content_type.is_none());
    }

    #[test]
    fn presign_url_trailing_slash_endpoint() {
        let url = presign_url(
            "https://oss.example.com/",
            "bucket",
            "key",
            "id",
            "sec",
            &PresignOptions::default(),
        )
        .unwrap();
        assert!(url.starts_with("https://bucket.oss.example.com/key?"));
    }
}
