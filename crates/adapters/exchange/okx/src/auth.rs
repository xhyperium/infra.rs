//! OKX v5 API 鉴权。
//!
//! OKX 要求所有已认证请求携带四个自定义头部：
//! `OK-ACCESS-KEY`、`OK-ACCESS-SIGN`、`OK-ACCESS-TIMESTAMP`、`OK-ACCESS-PASSPHRASE`。
//! 签名算法：Base64(HMAC-SHA256(timestamp + method + path + body))。
//!
//! 时间戳：Unix epoch **秒**（可带毫秒小数，如 `1597026383.123`）或 ISO-8601。
//! 本实现默认使用秒.毫秒格式（官方 REST 推荐形态之一）。

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// OKX API 凭证。
pub struct OkxApiKey {
    pub api_key: String,
    secret_key: String,
    pub passphrase: String,
}

impl OkxApiKey {
    #[must_use]
    pub fn new(
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
            passphrase: passphrase.into(),
        }
    }

    /// 对 prehash 字符串做 HMAC-SHA256 后 Base64 编码（公开以便向量测试）。
    #[must_use]
    pub fn sign_prehash(&self, prehash: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC-SHA256: key size is valid");
        mac.update(prehash.as_bytes());
        BASE64.encode(mac.finalize().into_bytes())
    }

    /// 返回 (OK-ACCESS-KEY, OK-ACCESS-SIGN, OK-ACCESS-TIMESTAMP, OK-ACCESS-PASSPHRASE)。
    pub fn sign(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
    ) -> (String, String, String, String) {
        let prehash = format!("{timestamp}{method}{path}{body}");
        let sig = self.sign_prehash(&prehash);
        (self.api_key.clone(), sig, timestamp.to_string(), self.passphrase.clone())
    }

    /// 生成所有四个已认证头部，自动获取当前时间戳。
    ///
    /// 返回 `Vec<(String, String)>`，可直接传入 `HttpRequest::headers`。
    pub fn sign_headers(&self, method: &str, path: &str, body: &str) -> Vec<(String, String)> {
        let ts = Self::now_timestamp();
        let (key, sig, ts, pass) = self.sign(&ts, method, path, body);
        vec![
            ("OK-ACCESS-KEY".to_string(), key),
            ("OK-ACCESS-SIGN".to_string(), sig),
            ("OK-ACCESS-TIMESTAMP".to_string(), ts),
            ("OK-ACCESS-PASSPHRASE".to_string(), pass),
        ]
    }

    /// OKX 时间戳：`{unix_seconds}.{millis:03}`。
    #[must_use]
    pub fn now_timestamp() -> String {
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        format!("{}.{:03}", d.as_secs(), d.subsec_millis())
    }
}

impl fmt::Debug for OkxApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OkxApiKey")
            .field("api_key", &format_args!("{}...***", &self.api_key[..4.min(self.api_key.len())]))
            .field("secret_key", &"<redacted>")
            .field("passphrase", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_vector_is_reproducible_and_base64() {
        let key = OkxApiKey::new("test-key", "test-secret", "test-passphrase");
        let ts = "2024-01-01T00:00:00.000Z";
        let method = "GET";
        let path = "/api/v5/account/balance";
        let body = "";
        let prehash = format!("{ts}{method}{path}{body}");
        let sig = key.sign_prehash(&prehash);
        assert_eq!(sig, key.sign_prehash(&prehash));
        // Base64 alphabet
        assert!(!sig.is_empty());
        assert!(sig.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
        let (ak, s2, t2, pp) = key.sign(ts, method, path, body);
        assert_eq!(ak, "test-key");
        assert_eq!(s2, sig);
        assert_eq!(t2, ts);
        assert_eq!(pp, "test-passphrase");
    }

    #[test]
    fn sign_produces_correct_headers() {
        let key = OkxApiKey::new("test-key", "test-secret", "test-passphrase");
        let (ak, sig, ts, pp) =
            key.sign("2024-01-01T00:00:00.000Z", "GET", "/api/v5/account/balance", "");
        assert_eq!(ak, "test-key");
        assert_eq!(ts, "2024-01-01T00:00:00.000Z");
        assert_eq!(pp, "test-passphrase");
        assert!(!sig.is_empty());
    }

    #[test]
    fn sign_headers_returns_four_entries() {
        let key = OkxApiKey::new("k", "s", "p");
        let headers = key.sign_headers("POST", "/api/v5/trade/order", r#"{"key":"val"}"#);
        assert_eq!(headers.len(), 4);
        let keys: Vec<&str> = headers.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"OK-ACCESS-KEY"));
        assert!(keys.contains(&"OK-ACCESS-SIGN"));
        assert!(keys.contains(&"OK-ACCESS-TIMESTAMP"));
        assert!(keys.contains(&"OK-ACCESS-PASSPHRASE"));
        // 时间戳为秒.毫秒
        let ts = headers
            .iter()
            .find(|(k, _)| k == "OK-ACCESS-TIMESTAMP")
            .map(|(_, v)| v.as_str())
            .expect("ts");
        assert!(ts.contains('.'), "expected seconds.millis, got {ts}");
    }

    #[test]
    fn debug_redacts_secret_and_passphrase() {
        let key = OkxApiKey::new("my-key", "my-secret", "my-passphrase");
        let dbg = format!("{key:?}");
        assert!(!dbg.contains("my-secret"), "secret leaked: {dbg}");
        assert!(!dbg.contains("my-passphrase"), "passphrase leaked: {dbg}");
    }

    #[test]
    fn now_timestamp_shape() {
        let ts = OkxApiKey::now_timestamp();
        let parts: Vec<_> = ts.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<u64>().is_ok());
        assert_eq!(parts[1].len(), 3);
    }
}
