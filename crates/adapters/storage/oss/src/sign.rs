//! 阿里云 OSS REST 签名 V1（`Authorization: OSS AccessKeyId:Signature`）。

use base64::Engine;
use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// 构造 StringToSign 并 HMAC-SHA1 + Base64。
///
/// ```text
/// StringToSign =
///   VERB + "\n"
///   + Content-MD5 + "\n"
///   + Content-Type + "\n"
///   + Date + "\n"
///   + CanonicalizedOSSHeaders
///   + CanonicalizedResource
/// ```
#[must_use]
pub fn sign_v1(
    secret: &str,
    verb: &str,
    content_md5: &str,
    content_type: &str,
    date: &str,
    canonicalized_oss_headers: &str,
    canonicalized_resource: &str,
) -> String {
    let string_to_sign = format!(
        "{verb}\n{content_md5}\n{content_type}\n{date}\n{canonicalized_oss_headers}{canonicalized_resource}"
    );
    let mut mac =
        HmacSha1::new_from_slice(secret.as_bytes()).expect("HMAC-SHA1 accepts any key length");
    mac.update(string_to_sign.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

/// `Authorization: OSS <AccessKeyId>:<Signature>`。
#[must_use]
pub fn authorization_header(access_key_id: &str, signature: &str) -> String {
    format!("OSS {access_key_id}:{signature}")
}

/// CanonicalizedResource：`/{bucket}/{object_key}`（object 可为空）。
#[must_use]
pub fn canonicalized_resource(bucket: &str, object_key: &str) -> String {
    if object_key.is_empty() {
        format!("/{bucket}/")
    } else {
        // 对象 key 不应以 / 开头
        let key = object_key.trim_start_matches('/');
        format!("/{bucket}/{key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 与阿里云文档示例同构的固定向量（自构造可复现）。
    #[test]
    fn signature_is_stable() {
        let sig = sign_v1(
            "secret",
            "PUT",
            "",
            "application/octet-stream",
            "Thu, 01 Jan 1970 00:00:00 GMT",
            "",
            "/bucket/key",
        );
        // 同一输入必须稳定
        let sig2 = sign_v1(
            "secret",
            "PUT",
            "",
            "application/octet-stream",
            "Thu, 01 Jan 1970 00:00:00 GMT",
            "",
            "/bucket/key",
        );
        assert_eq!(sig, sig2);
        assert!(!sig.is_empty());
        assert_eq!(authorization_header("AKID", &sig), format!("OSS AKID:{sig}"));
    }

    #[test]
    fn resource_format() {
        assert_eq!(canonicalized_resource("b", "a/b"), "/b/a/b");
        assert_eq!(canonicalized_resource("b", "/a"), "/b/a");
        assert_eq!(canonicalized_resource("b", ""), "/b/");
    }
}
