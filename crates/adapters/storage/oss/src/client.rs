//! 生产 `OssClient`：reqwest + OSS V1 签名。

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use contracts::ObjectStore;
use kernel::{XError, XResult};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, DATE, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method, StatusCode};
use url::Url;

use crate::config::OssConfig;
use crate::sign::{authorization_header, canonicalized_resource, sign_v1};

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
}

impl fmt::Debug for OssClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OssClient")
            .field("config", &self.inner.config)
            .field("base", &self.inner.base.as_str())
            .field("closed", &self.inner.closed.load(std::sync::atomic::Ordering::Relaxed))
            .finish()
    }
}

impl OssClient {
    /// 用配置建立客户端（懒连接：首次请求才真正打网）。
    pub fn connect(config: OssConfig) -> XResult<Self> {
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

    /// 上传对象。
    pub async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
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
            .map_err(|e| XError::unavailable(format!("oss PUT network: {e}")))?;

        map_status("PUT", key.as_str(), resp.status(), resp).await?;
        Ok(())
    }

    /// 下载对象。
    pub async fn get_object(&self, key: &str) -> XResult<Bytes> {
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
            .map_err(|e| XError::unavailable(format!("oss GET network: {e}")))?;

        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(XError::missing(format!("object not found: {key}")));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(status_error("GET", &key, status, &body));
        }
        let bytes =
            resp.bytes().await.map_err(|e| XError::unavailable(format!("oss GET body: {e}")))?;
        Ok(bytes)
    }

    /// 删除对象（幂等：不存在亦视为成功）。
    pub async fn delete_object(&self, key: &str) -> XResult<()> {
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
            .map_err(|e| XError::unavailable(format!("oss DELETE network: {e}")))?;

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
    XError::unavailable(format!("oss {op} failed status={status} key={key} body={snippet}"))
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
}
