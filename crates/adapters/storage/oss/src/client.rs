//! 生产 `OssClient`：reqwest + OSS V1 签名 + multipart + resiliencx 重试。

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use contracts::ObjectStore;
use kernel::{XError, XResult};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, DATE, ETAG, HeaderMap, HeaderName, HeaderValue,
};
use reqwest::{Client, Method, StatusCode};
use resiliencx::RetryConfig;
use url::Url;

use crate::config::OssConfig;
use crate::retry::{default_retry_config, with_retry_default};
use crate::sign::{
    authorization_header, canonicalized_resource, canonicalized_resource_with_subresources,
    sign_v1, split_parts,
};

/// 可克隆的阿里云 OSS 客户端（内部共享 `reqwest::Client` + 配置）。
#[derive(Clone)]
pub struct OssClient {
    inner: Arc<Inner>,
}

struct Inner {
    http: Client,
    config: OssConfig,
    /// 虚拟主机：`https://{bucket}.{endpoint_host}`
    base: Url,
    closed: std::sync::atomic::AtomicBool,
    retry: RetryConfig,
}

impl fmt::Debug for OssClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OssClient")
            .field("config", &self.inner.config)
            .field("base", &self.inner.base.as_str())
            .field("closed", &self.inner.closed.load(std::sync::atomic::Ordering::Relaxed))
            .field("retry_max_attempts", &self.inner.retry.max_attempts)
            .finish()
    }
}

impl OssClient {
    /// 用配置建立客户端（懒连接：首次请求才真正打网）。
    pub fn connect(config: OssConfig) -> XResult<Self> {
        Self::connect_with_retry(config, default_retry_config())
    }

    /// 用自定义重试配置建立客户端。
    pub fn connect_with_retry(config: OssConfig, retry: RetryConfig) -> XResult<Self> {
        config.validate()?;
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("ossx/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| XError::unavailable(format!("http client: {e}")))?;
        let base = virtual_host_base(&config.endpoint, &config.bucket)?;
        Ok(Self {
            inner: Arc::new(Inner {
                http,
                config,
                base,
                closed: std::sync::atomic::AtomicBool::new(false),
                retry,
            }),
        })
    }

    /// 从 `FOUNDATIONX_OSSX_*` 环境变量连接。
    pub fn from_env() -> XResult<Self> {
        Self::connect(OssConfig::from_env()?)
    }

    /// 配置只读视图。
    #[must_use]
    pub fn config(&self) -> &OssConfig {
        &self.inner.config
    }

    /// 当前重试配置。
    #[must_use]
    pub fn retry_config(&self) -> RetryConfig {
        self.inner.retry
    }

    /// 标记关闭（HTTP 连接池随 drop 释放；幂等）。
    pub fn close(&self) {
        self.inner.closed.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(XError::unavailable("oss client closed"));
        }
        Ok(())
    }

    /// 上传对象（含重试）。
    pub async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
        let key = key.to_string();
        let data = data;
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "put_object", move || {
            let this = this.clone();
            let key = key.clone();
            let data = data.clone();
            async move { this.put_object_once(&key, data).await }
        })
        .await
    }

    async fn put_object_once(&self, key: &str, data: Bytes) -> XResult<()> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let content_type = "application/octet-stream";
        let date = gmt_now();
        let resource = canonicalized_resource(&self.inner.config.bucket, &key);
        let sig = sign_v1(
            &self.inner.config.access_key_secret,
            "PUT",
            "",
            content_type,
            &date,
            "",
            &resource,
        );
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let url = object_url(&self.inner.base, &key)?;
        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(CONTENT_TYPE, header_value(content_type)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::PUT, url)
            .headers(headers)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| map_network("PUT", &e))?;

        map_status("PUT", key.as_str(), resp.status(), resp).await?;
        Ok(())
    }

    /// 下载对象（含重试）。
    pub async fn get_object(&self, key: &str) -> XResult<Bytes> {
        let key = key.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "get_object", move || {
            let this = this.clone();
            let key = key.clone();
            async move { this.get_object_once(&key).await }
        })
        .await
    }

    async fn get_object_once(&self, key: &str) -> XResult<Bytes> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let date = gmt_now();
        let resource = canonicalized_resource(&self.inner.config.bucket, &key);
        let sig =
            sign_v1(&self.inner.config.access_key_secret, "GET", "", "", &date, "", &resource);
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let url = object_url(&self.inner.base, &key)?;
        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::GET, url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| map_network("GET", &e))?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(XError::missing(format!("object not found: {key}")));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(status_error("GET", &key, status, &body));
        }
        let bytes =
            resp.bytes().await.map_err(|e| XError::transient(format!("oss GET body: {e}")))?;
        Ok(bytes)
    }

    /// 删除对象（幂等：不存在亦视为成功；含重试）。
    pub async fn delete_object(&self, key: &str) -> XResult<()> {
        let key = key.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "delete_object", move || {
            let this = this.clone();
            let key = key.clone();
            async move { this.delete_object_once(&key).await }
        })
        .await
    }

    async fn delete_object_once(&self, key: &str) -> XResult<()> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let date = gmt_now();
        let resource = canonicalized_resource(&self.inner.config.bucket, &key);
        let sig =
            sign_v1(&self.inner.config.access_key_secret, "DELETE", "", "", &date, "", &resource);
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let url = object_url(&self.inner.base, &key)?;
        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::DELETE, url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| map_network("DELETE", &e))?;

        let status = resp.status();
        // OSS：204 No Content / 200 / 404 均视为删除成功（幂等）
        if status == StatusCode::NOT_FOUND
            || status == StatusCode::NO_CONTENT
            || status.is_success()
        {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(status_error("DELETE", &key, status, &body))
    }

    // ── Multipart ──────────────────────────────────────────────────────────

    /// 初始化分片上传，返回 `upload_id`（含重试）。
    pub async fn initiate_multipart(&self, key: &str) -> XResult<String> {
        let key = key.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "initiate_multipart", move || {
            let this = this.clone();
            let key = key.clone();
            async move { this.initiate_multipart_once(&key).await }
        })
        .await
    }

    async fn initiate_multipart_once(&self, key: &str) -> XResult<String> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let date = gmt_now();
        let resource = canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            &key,
            &[("uploads", None)],
        );
        let sig =
            sign_v1(&self.inner.config.access_key_secret, "POST", "", "", &date, "", &resource);
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let mut url = object_url(&self.inner.base, &key)?;
        url.set_query(Some("uploads"));

        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::POST, url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| map_network("InitiateMultipart", &e))?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(status_error("InitiateMultipart", &key, status, &body));
        }
        parse_upload_id(&body)
            .ok_or_else(|| XError::internal(format!("InitiateMultipart 缺 UploadId: {body}")))
    }

    /// 上传单个分片，返回 ETag（含重试）。
    pub async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> XResult<String> {
        if part_number == 0 {
            return Err(XError::invalid("part_number 必须 ≥ 1"));
        }
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "upload_part", move || {
            let this = this.clone();
            let key = key.clone();
            let upload_id = upload_id.clone();
            let data = data.clone();
            async move { this.upload_part_once(&key, &upload_id, part_number, data).await }
        })
        .await
    }

    async fn upload_part_once(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> XResult<String> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let pn = part_number.to_string();
        let date = gmt_now();
        let content_type = "application/octet-stream";
        let resource = canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            &key,
            &[("partNumber", Some(pn.as_str())), ("uploadId", Some(upload_id))],
        );
        let sig = sign_v1(
            &self.inner.config.access_key_secret,
            "PUT",
            "",
            content_type,
            &date,
            "",
            &resource,
        );
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let mut url = object_url(&self.inner.base, &key)?;
        url.query_pairs_mut().append_pair("partNumber", &pn).append_pair("uploadId", upload_id);

        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(CONTENT_TYPE, header_value(content_type)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::PUT, url)
            .headers(headers)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| map_network("UploadPart", &e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(status_error("UploadPart", &key, status, &body));
        }
        let etag = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| XError::internal("UploadPart 响应缺 ETag"))?;
        Ok(etag)
    }

    /// 完成分片上传（含重试）。
    ///
    /// `parts`：`(part_number, etag)`，将按 `part_number` 排序写入 Complete XML。
    pub async fn complete_multipart(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<(u32, String)>,
    ) -> XResult<()> {
        if parts.is_empty() {
            return Err(XError::invalid("complete_multipart 需要至少一个 part"));
        }
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "complete_multipart", move || {
            let this = this.clone();
            let key = key.clone();
            let upload_id = upload_id.clone();
            let parts = parts.clone();
            async move { this.complete_multipart_once(&key, &upload_id, parts).await }
        })
        .await
    }

    async fn complete_multipart_once(
        &self,
        key: &str,
        upload_id: &str,
        mut parts: Vec<(u32, String)>,
    ) -> XResult<()> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        parts.sort_by_key(|(n, _)| *n);
        let body = build_complete_xml(&parts);
        let date = gmt_now();
        let content_type = "application/xml";
        let resource = canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            &key,
            &[("uploadId", Some(upload_id))],
        );
        let sig = sign_v1(
            &self.inner.config.access_key_secret,
            "POST",
            "",
            content_type,
            &date,
            "",
            &resource,
        );
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let mut url = object_url(&self.inner.base, &key)?;
        url.query_pairs_mut().append_pair("uploadId", upload_id);

        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(CONTENT_TYPE, header_value(content_type)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::POST, url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| map_network("CompleteMultipart", &e))?;

        map_status("CompleteMultipart", key.as_str(), resp.status(), resp).await?;
        Ok(())
    }

    /// 中止分片上传（含重试；幂等）。
    pub async fn abort_multipart(&self, key: &str, upload_id: &str) -> XResult<()> {
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_default(&retry, "abort_multipart", move || {
            let this = this.clone();
            let key = key.clone();
            let upload_id = upload_id.clone();
            async move { this.abort_multipart_once(&key, &upload_id).await }
        })
        .await
    }

    async fn abort_multipart_once(&self, key: &str, upload_id: &str) -> XResult<()> {
        self.ensure_open()?;
        let key = normalize_key(key)?;
        let date = gmt_now();
        let resource = canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            &key,
            &[("uploadId", Some(upload_id))],
        );
        let sig =
            sign_v1(&self.inner.config.access_key_secret, "DELETE", "", "", &date, "", &resource);
        let auth = authorization_header(&self.inner.config.access_key_id, &sig);

        let mut url = object_url(&self.inner.base, &key)?;
        url.query_pairs_mut().append_pair("uploadId", upload_id);

        let mut headers = HeaderMap::new();
        headers.insert(DATE, header_value(&date)?);
        headers.insert(AUTHORIZATION, header_value(&auth)?);

        let resp = self
            .inner
            .http
            .request(Method::DELETE, url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| map_network("AbortMultipart", &e))?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND
            || status == StatusCode::NO_CONTENT
            || status.is_success()
        {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        Err(status_error("AbortMultipart", &key, status, &body))
    }

    /// 高层：按 `part_size` 切分并完成 multipart 上传。
    ///
    /// - 数据为空 → `Invalid`
    /// - 单片（`data.len() <= part_size` 或 `part_size == 0`）仍走 multipart 路径
    /// - 任一分片失败时尝试 `abort_multipart`（失败忽略）
    pub async fn put_object_multipart(
        &self,
        key: &str,
        data: Bytes,
        part_size: usize,
    ) -> XResult<()> {
        if data.is_empty() {
            return Err(XError::invalid("multipart 数据不能为空"));
        }
        let chunks = split_parts(&data, part_size);
        if chunks.is_empty() {
            return Err(XError::invalid("multipart 无有效分片"));
        }
        // 阿里云 OSS multipart 分片上限 10000
        const MAX_PARTS: usize = 10_000;
        if chunks.len() > MAX_PARTS {
            return Err(XError::invalid(format!(
                "multipart 分片数 {} 超过上限 {MAX_PARTS}；请增大 part_size",
                chunks.len()
            )));
        }

        let upload_id = self.initiate_multipart(key).await?;
        let mut completed: Vec<(u32, String)> = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let part_number =
                u32::try_from(i + 1).map_err(|_| XError::invalid("multipart part_number 溢出"))?;
            // 拷贝 chunk 为 Bytes（分片重试需要所有权）
            let part_data = Bytes::copy_from_slice(chunk);
            match self.upload_part(key, &upload_id, part_number, part_data).await {
                Ok(etag) => completed.push((part_number, etag)),
                Err(e) => {
                    let _ = self.abort_multipart(key, &upload_id).await;
                    return Err(e);
                }
            }
        }
        if let Err(e) = self.complete_multipart(key, &upload_id, completed).await {
            let _ = self.abort_multipart(key, &upload_id).await;
            return Err(e);
        }
        Ok(())
    }
}

#[async_trait]
impl ObjectStore for OssClient {
    async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
        OssClient::put_object(self, key, data).await
    }

    async fn get_object(&self, key: &str) -> XResult<Bytes> {
        OssClient::get_object(self, key).await
    }
}

fn virtual_host_base(endpoint: &str, bucket: &str) -> XResult<Url> {
    let ep = Url::parse(endpoint.trim_end_matches('/'))
        .map_err(|e| XError::invalid(format!("bad endpoint URL: {e}")))?;
    let host = ep.host_str().ok_or_else(|| XError::invalid("endpoint missing host"))?;
    let scheme = ep.scheme();
    let vh = format!("{scheme}://{bucket}.{host}");
    Url::parse(&vh).map_err(|e| XError::invalid(format!("virtual host URL: {e}")))
}

fn object_url(base: &Url, key: &str) -> XResult<Url> {
    // 逐段拼接，保留 key 内的 /
    let mut url = base.clone();
    {
        let mut segs =
            url.path_segments_mut().map_err(|_| XError::invalid("base URL cannot be a base"))?;
        segs.clear();
        for part in key.split('/') {
            if !part.is_empty() {
                segs.push(part);
            }
        }
    }
    Ok(url)
}

fn normalize_key(key: &str) -> XResult<String> {
    let k = key.trim().trim_start_matches('/');
    if k.is_empty() {
        return Err(XError::invalid("object key must not be empty"));
    }
    if k.contains("..") {
        return Err(XError::invalid("object key must not contain '..'"));
    }
    Ok(k.to_string())
}

fn gmt_now() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn header_value(s: &str) -> XResult<HeaderValue> {
    HeaderValue::from_str(s).map_err(|e| XError::invalid(format!("header value: {e}")))
}

async fn map_status(
    op: &str,
    key: &str,
    status: StatusCode,
    resp: reqwest::Response,
) -> XResult<()> {
    if status.is_success() {
        return Ok(());
    }
    let body = resp.text().await.unwrap_or_default();
    Err(status_error(op, key, status, &body))
}

fn map_network(op: &str, err: &reqwest::Error) -> XError {
    if err.is_timeout() {
        return XError::deadline_exceeded(format!("oss {op} timeout: {err}"));
    }
    // 网络抖动视为 Transient，便于 resiliencx 重试
    XError::transient(format!("oss {op} network: {err}"))
}

fn status_error(op: &str, key: &str, status: StatusCode, body: &str) -> XError {
    // 截断响应，避免日志爆炸；不回显凭据
    let snippet: String = body.chars().take(512).collect();
    if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        return XError::unavailable(format!(
            "oss {op} auth/forbidden status={status} key={key} body={snippet}"
        ));
    }
    if status == StatusCode::NOT_FOUND {
        return XError::missing(format!("oss {op} not found key={key}"));
    }
    if status.is_server_error() {
        return XError::transient(format!(
            "oss {op} server status={status} key={key} body={snippet}"
        ));
    }
    if status.is_client_error() {
        return XError::invalid(format!(
            "oss {op} client status={status} key={key} body={snippet}"
        ));
    }
    XError::unavailable(format!("oss {op} failed status={status} key={key} body={snippet}"))
}

/// 从 InitiateMultipartUploadResult XML 中提取 UploadId（轻量解析，无额外依赖）。
fn parse_upload_id(xml: &str) -> Option<String> {
    const OPEN: &str = "<UploadId>";
    const CLOSE: &str = "</UploadId>";
    let start = xml.find(OPEN)? + OPEN.len();
    let end = xml[start..].find(CLOSE)? + start;
    let id = xml[start..end].trim();
    if id.is_empty() { None } else { Some(id.to_string()) }
}

/// 构造 CompleteMultipartUpload XML。
fn build_complete_xml(parts: &[(u32, String)]) -> String {
    let mut xml = String::from("<CompleteMultipartUpload>");
    for (n, etag) in parts {
        // ETag 可能已带引号；Complete 请求要求保留原值
        xml.push_str(&format!("<Part><PartNumber>{n}</PartNumber><ETag>{etag}</ETag></Part>"));
    }
    xml.push_str("</CompleteMultipartUpload>");
    xml
}

/// 可选：自定义头（保留扩展位；当前未使用）。
#[allow(dead_code)]
fn insert_header(map: &mut HeaderMap, name: &str, value: &str) -> XResult<()> {
    let n = HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| XError::invalid(format!("header name: {e}")))?;
    map.insert(n, header_value(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_host_builds() {
        let u = virtual_host_base("https://oss-ap-northeast-1.aliyuncs.com", "x-go").unwrap();
        assert_eq!(u.as_str(), "https://x-go.oss-ap-northeast-1.aliyuncs.com/");
    }

    #[test]
    fn object_url_nested() {
        let base = Url::parse("https://b.oss.example.com").unwrap();
        let u = object_url(&base, "infra-draft/a/b.txt").unwrap();
        assert_eq!(u.as_str(), "https://b.oss.example.com/infra-draft/a/b.txt");
    }

    #[test]
    fn reject_empty_key() {
        assert!(normalize_key("").is_err());
        assert!(normalize_key("  /  ").is_err());
    }

    #[test]
    fn parse_upload_id_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>b</Bucket>
  <Key>k</Key>
  <UploadId>0004B9894A22E5B1888A1E29F8236E2D</UploadId>
</InitiateMultipartUploadResult>"#;
        assert_eq!(parse_upload_id(xml).as_deref(), Some("0004B9894A22E5B1888A1E29F8236E2D"));
        assert!(parse_upload_id("<root/>").is_none());
    }

    #[test]
    fn complete_xml_orders_parts() {
        let xml = build_complete_xml(&[(1, "\"etag1\"".into()), (2, "\"etag2\"".into())]);
        assert!(xml.contains("<PartNumber>1</PartNumber>"));
        assert!(xml.contains("<ETag>\"etag1\"</ETag>"));
        assert!(xml.starts_with("<CompleteMultipartUpload>"));
        assert!(xml.ends_with("</CompleteMultipartUpload>"));
    }

    #[test]
    fn connect_validates_config() {
        let err = OssClient::connect(OssConfig {
            endpoint: String::new(),
            bucket: "b".into(),
            access_key_id: "id".into(),
            access_key_secret: "sec".into(),
            region: "r".into(),
        });
        assert!(err.is_err());
    }
}
