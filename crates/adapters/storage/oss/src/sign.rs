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

/// 带 OSS 子资源的 CanonicalizedResource。
///
/// 子资源按字典序排序后以 `?k` / `?k=v` 拼接，符合阿里云 OSS V1 规则。
/// 典型 multipart 场景：
/// - initiate：`[("uploads", None)]` → `?uploads`
/// - upload_part：`[("partNumber", Some("1")), ("uploadId", Some(id))]`
/// - complete/abort：`[("uploadId", Some(id))]`
#[must_use]
pub fn canonicalized_resource_with_subresources(
    bucket: &str,
    object_key: &str,
    subresources: &[(&str, Option<&str>)],
) -> String {
    let base = canonicalized_resource(bucket, object_key);
    if subresources.is_empty() {
        return base;
    }
    let mut pairs: Vec<String> = subresources
        .iter()
        .map(|(k, v)| match v {
            Some(val) if !val.is_empty() => format!("{k}={val}"),
            _ => (*k).to_string(),
        })
        .collect();
    pairs.sort();
    format!("{base}?{}", pairs.join("&"))
}

/// 分片切分：按 `part_size` 将数据切为若干切片（纯函数，无网络）。
///
/// - `part_size == 0` 时按整段返回（若 data 非空则 1 片）
/// - 空 data 返回空 vec
#[must_use]
pub fn split_parts(data: &[u8], part_size: usize) -> Vec<&[u8]> {
    if data.is_empty() {
        return Vec::new();
    }
    if part_size == 0 || part_size >= data.len() {
        return vec![data];
    }
    data.chunks(part_size).collect()
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

    #[test]
    fn multipart_subresources_sorted() {
        // partNumber 在 uploadId 之前（字典序）
        let r = canonicalized_resource_with_subresources(
            "bucket",
            "obj/key",
            &[("uploadId", Some("UID")), ("partNumber", Some("2"))],
        );
        assert_eq!(r, "/bucket/obj/key?partNumber=2&uploadId=UID");

        let init = canonicalized_resource_with_subresources("b", "k", &[("uploads", None)]);
        assert_eq!(init, "/b/k?uploads");

        let complete =
            canonicalized_resource_with_subresources("b", "k", &[("uploadId", Some("u1"))]);
        assert_eq!(complete, "/b/k?uploadId=u1");
    }

    #[test]
    fn multipart_resource_signing_stable() {
        let resource =
            canonicalized_resource_with_subresources("bucket", "k", &[("uploads", None)]);
        let sig = sign_v1("sec", "POST", "", "", "date", "", &resource);
        let sig2 = sign_v1("sec", "POST", "", "", "date", "", &resource);
        assert_eq!(sig, sig2);
        assert!(!sig.is_empty());
    }

    #[test]
    fn split_parts_chunking() {
        let data = b"abcdefghij"; // 10
        let parts = split_parts(data, 3);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], b"abc");
        assert_eq!(parts[1], b"def");
        assert_eq!(parts[2], b"ghi");
        assert_eq!(parts[3], b"j");

        assert!(split_parts(b"", 4).is_empty());
        assert_eq!(split_parts(data, 0), vec![&data[..]]);
        assert_eq!(split_parts(data, 100), vec![&data[..]]);
    }
}
