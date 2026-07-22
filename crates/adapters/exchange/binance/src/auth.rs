//! Binance API v3 鉴权。

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Binance API 凭证。
///
/// 使用 HMAC-SHA256 对请求进行签名。Binance 要求所有已认证请求携带
/// `X-MBX-APIKEY` 头部，并在查询字符串中附加 `signature` 参数。
pub struct BinanceApiKey {
    pub api_key: String,
    secret_key: String,
}

impl BinanceApiKey {
    #[must_use]
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), secret_key: secret_key.into() }
    }

    #[must_use]
    pub fn api_key_header(&self) -> String {
        self.api_key.clone()
    }

    fn sign_payload(&self, payload: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC-SHA256: key size is valid");
        mac.update(payload.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn sign_query(&self, mut query: String) -> (String, String) {
        let ts = Self::now_millis();
        if !query.is_empty() {
            query.push('&');
        }
        query.push_str(&format!("timestamp={ts}&recvWindow=5000"));
        let sig = self.sign_payload(&query);
        query.push_str(&format!("&signature={sig}"));
        (self.api_key.clone(), query)
    }

    pub fn sign_params(&self, params: &[(&str, &str)]) -> (String, String) {
        let query = params
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        self.sign_query(query)
    }
}

impl fmt::Debug for BinanceApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinanceApiKey")
            .field("api_key", &format_args!("{}...***", &self.api_key[..4.min(self.api_key.len())]))
            .field("secret_key", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_query_includes_timestamp_and_signature() {
        let key = BinanceApiKey::new("test-key", "test-secret");
        let (api_key, query) = key.sign_query("symbol=BTCUSDT&side=BUY".into());
        assert_eq!(api_key, "test-key");
        assert!(query.starts_with("symbol=BTCUSDT&side=BUY&timestamp="));
        assert!(query.contains("&recvWindow=5000&signature="));
    }

    #[test]
    fn sign_params_preserves_order() {
        let key = BinanceApiKey::new("key", "secret");
        let (_, query) = key.sign_params(&[("symbol", "ETHUSDT"), ("side", "SELL")]);
        assert!(query.starts_with("symbol=ETHUSDT&side=SELL&timestamp="));
    }

    #[test]
    fn debug_redacts_secret() {
        let key = BinanceApiKey::new("my-api-key", "my-secret");
        let dbg = format!("{key:?}");
        assert!(!dbg.contains("my-secret"), "secret leaked: {dbg}");
    }
}
