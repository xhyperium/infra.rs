//! OssPool — 共享 OSS 连接池，v0.4.0 新推荐入口。
//! 支持流式、multipart、SSE、预签名 URL、凭据提供者、健康检查、池统计。

use crate::config::OssConfig;
use crate::credential::{CredentialProvider, StaticCredentialProvider};
use crate::retry::{self, default_retry_config};
use crate::sign;
use crate::types::{DownloadOptions, ObjectMeta, UploadOptions};
use bytes::{Bytes, BytesMut};
use chrono::Utc;
use contracts::ObjectStore;
use futures_util::StreamExt;
use kernel::error::{XError, XResult};
use reqwest::header::HeaderValue;
use reqwest::{Client, Method, StatusCode, Url};
use resiliencx::RetryConfig;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time;

const MIN_PART: usize = 100 * 1024;
const MAX_PARTS: usize = 10_000;
const MAX_PART: usize = 512 * 1024 * 1024;

#[derive(Clone, Debug, Default)]
pub struct OssPoolStats {
    pub closed: bool,
    pub in_flight: usize,
    pub max_in_flight: usize,
    pub puts_ok: u64,
    pub puts_err: u64,
    pub gets_ok: u64,
    pub gets_err: u64,
    pub deletes_ok: u64,
    pub deletes_err: u64,
    pub timeouts: u64,
    pub cancelled: u64,
}

#[derive(Clone, Debug)]
pub struct OssHealth {
    pub ready: bool,
    pub bucket_accessible: bool,
    pub latency_ms: u64,
    pub detail: String,
}

struct PoolInner {
    http: Client,
    config: OssConfig,
    base: Url,
    closed: AtomicBool,
    sem: Arc<Semaphore>,
    retry: RetryConfig,
    credential_provider: Arc<dyn CredentialProvider>,
    puts_ok: AtomicU64,
    puts_err: AtomicU64,
    gets_ok: AtomicU64,
    gets_err: AtomicU64,
    deletes_ok: AtomicU64,
    deletes_err: AtomicU64,
    timeouts: AtomicU64,
    cancelled: AtomicU64,
}

#[derive(Clone)]
pub struct OssPool {
    inner: Arc<PoolInner>,
}

impl OssPool {
    pub fn connect(config: OssConfig) -> XResult<Self> {
        Self::connect_with_retry(config, default_retry_config())
    }
    pub fn connect_with_retry(config: OssConfig, retry: RetryConfig) -> XResult<Self> {
        Self::connect_with_provider(config, retry, None)
    }
    pub fn connect_with_provider(
        config: OssConfig,
        retry: RetryConfig,
        cp: Option<Arc<dyn CredentialProvider>>,
    ) -> XResult<Self> {
        config.validate()?;
        if retry.max_attempts == 0 || retry.max_attempts > retry::MAX_RETRY_ATTEMPTS {
            return Err(XError::invalid("invalid retry config"));
        }
        let ak_id = config.access_key_id.clone();
        let ak_sec = config.access_key_secret.clone();
        let st = config.security_token.clone();
        let base = virtual_host_base(&config.endpoint, &config.bucket)?;
        let http = Client::builder()
            .timeout(config.request_timeout)
            .pool_max_idle_per_host(config.max_in_flight)
            .user_agent(concat!("ossx/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| XError::internal(format!("http client: {e}")))?;
        let cp = cp.unwrap_or_else(|| Arc::new(StaticCredentialProvider::new(ak_id, ak_sec, st)));
        let max_in_flight = config.max_in_flight;
        Ok(Self {
            inner: Arc::new(PoolInner {
                http,
                config,
                base,
                closed: AtomicBool::new(false),
                sem: Arc::new(Semaphore::new(max_in_flight)),
                retry,
                credential_provider: cp,
                puts_ok: AtomicU64::new(0),
                puts_err: AtomicU64::new(0),
                gets_ok: AtomicU64::new(0),
                gets_err: AtomicU64::new(0),
                deletes_ok: AtomicU64::new(0),
                deletes_err: AtomicU64::new(0),
                timeouts: AtomicU64::new(0),
                cancelled: AtomicU64::new(0),
            }),
        })
    }
    pub fn from_env() -> XResult<Self> {
        Self::connect(OssConfig::from_env()?)
    }
    pub fn config(&self) -> &OssConfig {
        &self.inner.config
    }
    pub fn stats(&self) -> OssPoolStats {
        let i = &self.inner;
        OssPoolStats {
            closed: i.closed.load(Ordering::SeqCst),
            in_flight: i.config.max_in_flight.saturating_sub(i.sem.available_permits()),
            max_in_flight: i.config.max_in_flight,
            puts_ok: i.puts_ok.load(Ordering::Relaxed),
            puts_err: i.puts_err.load(Ordering::Relaxed),
            gets_ok: i.gets_ok.load(Ordering::Relaxed),
            gets_err: i.gets_err.load(Ordering::Relaxed),
            deletes_ok: i.deletes_ok.load(Ordering::Relaxed),
            deletes_err: i.deletes_err.load(Ordering::Relaxed),
            timeouts: i.timeouts.load(Ordering::Relaxed),
            cancelled: i.cancelled.load(Ordering::Relaxed),
        }
    }
    pub async fn health(&self, deadline: Duration) -> OssHealth {
        let start = std::time::Instant::now();
        let creds = match self.inner.credential_provider.get_credentials().await {
            Ok(c) => c,
            Err(e) => {
                return OssHealth {
                    ready: false,
                    bucket_accessible: false,
                    latency_ms: 0,
                    detail: format!("creds: {e}"),
                };
            }
        };
        match time::timeout(deadline, async {
            let date = gmt_now();
            let r = sign::canonicalized_resource(&self.inner.config.bucket, "");
            let sig = sign::sign_v1(&creds.access_key_secret, "HEAD", "", "", &date, "", &r);
            self.inner
                .http
                .request(Method::HEAD, self.inner.base.clone())
                .header(
                    "Authorization",
                    header_value(&sign::authorization_header(&creds.access_key_id, &sig)),
                )
                .header("Date", header_value(&date))
                .header("Host", host_header(&self.inner.base))
                .send()
                .await
        })
        .await
        {
            Ok(Ok(r)) if r.status().is_success() => OssHealth {
                ready: true,
                bucket_accessible: true,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: format!(
                    "{} bucket={}",
                    self.inner.config.endpoint, self.inner.config.bucket
                ),
            },
            Ok(Ok(_)) => OssHealth {
                ready: false,
                bucket_accessible: false,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: "bucket not accessible".into(),
            },
            _ => OssHealth {
                ready: false,
                bucket_accessible: false,
                latency_ms: start.elapsed().as_millis() as u64,
                detail: "timeout".into(),
            },
        }
    }
    pub async fn close(&self, _: Duration) -> XResult<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.sem.close();
        Ok(())
    }

    #[tracing::instrument(skip(self, key, data))]
    pub async fn put_object(&self, key: impl Into<String>, data: Bytes) -> XResult<()> {
        let key = self.norm(key)?;
        self.valid_size(data.len())?;
        let r = retry::with_retry_deadline(
            &self.inner.retry,
            "put_object",
            self.inner.config.operation_deadline,
            || async {
                let _p = self.acquire().await?;
                let k = key.clone();
                let d = data.clone();
                self.put_once(&k, &d).await
            },
        )
        .await;
        match &r {
            Ok(()) => {
                self.inner.puts_ok.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.inner.puts_err.fetch_add(1, Ordering::Relaxed);
            }
        }
        r
    }
    #[tracing::instrument(skip(self, key))]
    pub async fn get_object(&self, key: impl Into<String>) -> XResult<Bytes> {
        let key = self.norm(key)?;
        let r = retry::with_retry_deadline(
            &self.inner.retry,
            "get_object",
            self.inner.config.operation_deadline,
            || async {
                let _p = self.acquire().await?;
                let k = key.clone();
                self.get_once(&k).await
            },
        )
        .await;
        match &r {
            Ok(_) => {
                self.inner.gets_ok.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.inner.gets_err.fetch_add(1, Ordering::Relaxed);
            }
        }
        r
    }
    #[tracing::instrument(skip(self, key))]
    pub async fn delete_object(&self, key: impl Into<String>) -> XResult<()> {
        let key = self.norm(key)?;
        let r = retry::with_retry_deadline(
            &self.inner.retry,
            "delete_object",
            self.inner.config.operation_deadline,
            || async {
                let _p = self.acquire().await?;
                let k = key.clone();
                self.delete_once(&k).await
            },
        )
        .await;
        match &r {
            Ok(()) => {
                self.inner.deletes_ok.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.inner.deletes_err.fetch_add(1, Ordering::Relaxed);
            }
        }
        r
    }
    #[tracing::instrument(skip(self, key))]
    pub async fn head(&self, key: impl Into<String>) -> XResult<ObjectMeta> {
        let key = self.norm(key)?;
        retry::with_retry_deadline(
            &self.inner.retry,
            "head",
            self.inner.config.operation_deadline,
            || async {
                let _p = self.acquire().await?;
                let k = key.clone();
                self.head_once(&k).await
            },
        )
        .await
    }
    #[tracing::instrument(skip(self, key, stream))]
    pub async fn put_stream(
        &self,
        key: impl Into<String>,
        stream: crate::types::ByteStream,
        opts: UploadOptions,
    ) -> XResult<ObjectMeta> {
        let key = self.norm(key)?;
        let ps = if opts.part_size > 0 { opts.part_size } else { 5 * 1024 * 1024 };
        if !(MIN_PART..=MAX_PART).contains(&ps) {
            return Err(XError::invalid("invalid part size"));
        }
        let mut stream = stream;
        let mut parts: Vec<Bytes> = Vec::new();
        {
            let mut cur = BytesMut::with_capacity(ps);
            loop {
                match time::timeout(self.inner.config.request_timeout, stream.next()).await {
                    Ok(Some(Ok(c))) => {
                        cur.extend_from_slice(&c);
                        if cur.len() >= ps {
                            parts.push(cur.split().freeze());
                        }
                        if parts.len() > MAX_PARTS {
                            return Err(XError::invalid("too many parts"));
                        }
                    }
                    Ok(Some(Err(e))) => return Err(e),
                    Ok(None) => {
                        if !cur.is_empty() {
                            parts.push(cur.freeze());
                        }
                        break;
                    }
                    Err(_) => return Err(XError::deadline_exceeded("stream timeout")),
                }
            }
        }
        let ts: usize = parts.iter().map(|p| p.len()).sum();
        if ts > self.inner.config.max_object_bytes {
            return Err(XError::invalid(format!(
                "object exceeding max size {ts} > {}",
                self.inner.config.max_object_bytes
            )));
        }
        if parts.len() == 1 {
            self.put_once(&key, &parts[0]).await?;
            return Ok(ObjectMeta::with_size(ts as u64));
        }
        let _p = self.acquire().await?;
        let uid = self.init_mp_once(&key, opts.sse_enabled).await?;
        let mut done: Vec<(u32, String)> = Vec::with_capacity(parts.len());
        let mut hs = Vec::new();
        for (i, pd) in parts.into_iter().enumerate() {
            let pn = (i + 1) as u32;
            let c = self.clone();
            let k = key.clone();
            let u = uid.clone();
            let rt = self.inner.retry;
            let sse = opts.sse_enabled;
            hs.push(tokio::spawn(async move {
                retry::with_retry_deadline(
                    &rt,
                    "upload_part",
                    c.config().operation_deadline,
                    || {
                        let p = c.clone();
                        let k = k.clone();
                        let u = u.clone();
                        let d = pd.clone();
                        async move {
                            let _ = p.acquire().await?;
                            p.upload_part_once(&k, &u, pn, &d, sse).await
                        }
                    },
                )
                .await
                .map(|e| (pn, e))
            }));
        }
        for h in hs {
            match h.await {
                Ok(Ok(p)) => done.push(p),
                _ => {
                    let _ = self.abort_mp_once(&key, &uid).await;
                    return Err(XError::transient("part upload failed"));
                }
            }
        }
        done.sort_by_key(|(n, _)| *n);
        let _p = self.acquire().await?;
        self.complete_mp_once(&key, &uid, &done).await?;
        Ok(ObjectMeta::with_size(ts as u64))
    }
    #[tracing::instrument(skip(self, key))]
    pub async fn get_stream(
        &self,
        key: impl Into<String>,
        opts: DownloadOptions,
    ) -> XResult<(ObjectMeta, crate::types::ByteStream)> {
        let key = self.norm(key)?;
        let creds = self.inner.credential_provider.get_credentials().await?;
        let url = obj_url(&self.inner.base, &key)?;
        let date = gmt_now();
        let r = sign::canonicalized_resource(&self.inner.config.bucket, &key);
        let sig = sign::sign_v1(&creds.access_key_secret, "GET", "", "", &date, "", &r);
        let _p = self.acquire().await?;
        let mut req = self
            .inner
            .http
            .request(Method::GET, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&creds.access_key_id, &sig)),
            )
            .header("Date", header_value(&date))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &creds.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        if let Some(r) = &opts.range {
            req = req.header("Range", header_value(r));
        }
        if let Some(m) = &opts.if_match {
            req = req.header("If-Match", header_value(m));
        }
        if let Some(m) = &opts.if_none_match {
            req = req.header("If-None-Match", header_value(m));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status() == StatusCode::NOT_MODIFIED {
            return Ok((ObjectMeta::with_size(0), Box::pin(futures_util::stream::empty())));
        }
        if resp.status() == StatusCode::PRECONDITION_FAILED {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            return Err(status_error(s, &b));
        }
        if !resp.status().is_success() {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            return Err(status_error(s, &b));
        }
        let h = resp.headers().clone();
        let meta = ObjectMeta {
            size: h
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            etag: h
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim_matches('"').to_string()),
            version_id: h.get("x-oss-version-id").and_then(|v| v.to_str().ok()).map(String::from),
            checksum: h.get("x-oss-hash-crc64ecma").and_then(|v| v.to_str().ok()).map(String::from),
            content_type: h.get("content-type").and_then(|v| v.to_str().ok()).map(String::from),
        };
        let mc = self.inner.config.max_buffer_bytes;
        let bs: crate::types::ByteStream =
            Box::pin(futures_util::stream::unfold((resp, mc), move |(mut resp, mc)| async move {
                match resp.chunk().await {
                    Ok(Some(c)) => Some((Ok(c), (resp, mc))),
                    Ok(None) => None,
                    Err(e) => Some((Err(map_network(e)), (resp, mc))),
                }
            }));
        Ok((meta, bs))
    }

    // ── 内部 ─────────────────────────────────────────────────────────────────

    async fn acquire(&self) -> XResult<tokio::sync::OwnedSemaphorePermit> {
        if self.inner.closed.load(Ordering::SeqCst) {
            self.inner.cancelled.fetch_add(1, Ordering::Relaxed);
            return Err(XError::cancelled("pool closed"));
        }
        let p = time::timeout(
            self.inner.config.acquire_timeout,
            self.inner.sem.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            self.inner.timeouts.fetch_add(1, Ordering::Relaxed);
            XError::deadline_exceeded("acquire timeout")
        })?;
        if self.inner.closed.load(Ordering::SeqCst) {
            self.inner.cancelled.fetch_add(1, Ordering::Relaxed);
            return Err(XError::cancelled("pool closed"));
        }
        p.map_err(|e| XError::cancelled(format!("sem closed: {e}")))
    }

    fn norm(&self, key: impl Into<String>) -> XResult<String> {
        let key = key.into();
        let t = key.trim().trim_start_matches('/');
        if t.is_empty() || t.contains("..") || t.len() > 1023 || t.chars().any(|c| c.is_control()) {
            return Err(XError::invalid("invalid key"));
        }
        Ok(t.to_string())
    }

    fn valid_size(&self, size: usize) -> XResult<()> {
        if size > self.inner.config.max_object_bytes {
            return Err(XError::invalid("object too large".to_string()));
        }
        if size > self.inner.config.max_buffer_bytes {
            return Err(XError::invalid("use put_stream for large objects".to_string()));
        }
        Ok(())
    }

    async fn put_once(&self, key: &str, data: &[u8]) -> XResult<()> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = obj_url(&self.inner.base, key)?;
        let d = gmt_now();
        let r = sign::canonicalized_resource(&self.inner.config.bucket, key);
        let sig =
            sign::sign_v1(&c.access_key_secret, "PUT", "", "application/octet-stream", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::PUT, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base))
            .header("Content-Type", header_value("application/octet-stream"))
            .header("Content-Length", header_value(&data.len().to_string()));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.body(data.to_vec()).send().await.map_err(map_network)?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn get_once(&self, key: &str) -> XResult<Bytes> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = obj_url(&self.inner.base, key)?;
        let d = gmt_now();
        let r = sign::canonicalized_resource(&self.inner.config.bucket, key);
        let sig = sign::sign_v1(&c.access_key_secret, "GET", "", "", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::GET, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status().is_success() {
            if let Some(cl) = resp.content_length() {
                if cl > self.inner.config.max_object_bytes as u64 {
                    return Err(XError::invalid("object too large, use get_stream"));
                }
            }
            Ok(read_body(resp, self.inner.config.max_buffer_bytes).await)
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn delete_once(&self, key: &str) -> XResult<()> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = obj_url(&self.inner.base, key)?;
        let d = gmt_now();
        let r = sign::canonicalized_resource(&self.inner.config.bucket, key);
        let sig = sign::sign_v1(&c.access_key_secret, "DELETE", "", "", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::DELETE, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status().is_success()
            || resp.status() == StatusCode::NO_CONTENT
            || resp.status() == StatusCode::NOT_FOUND
        {
            Ok(())
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn head_once(&self, key: &str) -> XResult<ObjectMeta> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = obj_url(&self.inner.base, key)?;
        let d = gmt_now();
        let r = sign::canonicalized_resource(&self.inner.config.bucket, key);
        let sig = sign::sign_v1(&c.access_key_secret, "HEAD", "", "", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::HEAD, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status().is_success() {
            let h = resp.headers();
            Ok(ObjectMeta {
                size: h
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                etag: h
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.trim_matches('"').to_string()),
                version_id: h
                    .get("x-oss-version-id")
                    .and_then(|v| v.to_str().ok())
                    .map(String::from),
                checksum: h
                    .get("x-oss-hash-crc64ecma")
                    .and_then(|v| v.to_str().ok())
                    .map(String::from),
                content_type: h.get("content-type").and_then(|v| v.to_str().ok()).map(String::from),
            })
        } else if resp.status() == StatusCode::NOT_FOUND {
            Err(XError::missing(format!("object not found: {key}")))
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn init_mp_once(&self, key: &str, sse: bool) -> XResult<String> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = {
            let mut u = obj_url(&self.inner.base, key)?;
            u.query_pairs_mut().append_pair("uploads", "");
            u
        };
        let d = gmt_now();
        let r = sign::canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            key,
            &[("uploads", None)],
        );
        let sig = sign::sign_v1(&c.access_key_secret, "POST", "", "", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::POST, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        if sse {
            req = req.header("x-oss-server-side-encryption", header_value("AES256"));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status().is_success() {
            let body = read_body(resp, self.inner.config.max_error_body_bytes).await;
            let xml = String::from_utf8_lossy(&body);
            if let Some(s) = xml.find("<UploadId>") {
                let c = &xml[s + 10..];
                if let Some(e) = c.find("</UploadId>") {
                    return Ok(c[..e].to_string());
                }
            }
            Err(XError::internal("UploadId not found"))
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn upload_part_once(
        &self,
        key: &str,
        uid: &str,
        pn: u32,
        data: &[u8],
        sse: bool,
    ) -> XResult<String> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = {
            let mut u = obj_url(&self.inner.base, key)?;
            u.query_pairs_mut()
                .append_pair("partNumber", &pn.to_string())
                .append_pair("uploadId", uid);
            u
        };
        let d = gmt_now();
        let r = sign::canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            key,
            &[("partNumber", Some(&pn.to_string())), ("uploadId", Some(uid))],
        );
        let sig =
            sign::sign_v1(&c.access_key_secret, "PUT", "", "application/octet-stream", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::PUT, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base))
            .header("Content-Length", header_value(&data.len().to_string()));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        if sse {
            req = req.header("x-oss-server-side-encryption", header_value("AES256"));
        }
        let resp = req.body(data.to_vec()).send().await.map_err(map_network)?;
        if resp.status().is_success() {
            resp.headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim_matches('"').to_string())
                .ok_or_else(|| XError::internal("ETag missing"))
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn complete_mp_once(&self, key: &str, uid: &str, parts: &[(u32, String)]) -> XResult<()> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = {
            let mut u = obj_url(&self.inner.base, key)?;
            u.query_pairs_mut().append_pair("uploadId", uid);
            u
        };
        let d = gmt_now();
        let r = sign::canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            key,
            &[("uploadId", Some(uid))],
        );
        let sig = sign::sign_v1(&c.access_key_secret, "POST", "", "application/xml", &d, "", &r);
        let mut xml = String::from("<CompleteMultipartUpload>");
        for (n, e) in parts {
            xml.push_str(&format!("<Part><PartNumber>{n}</PartNumber><ETag>\"{e}\"</ETag></Part>"));
        }
        xml.push_str("</CompleteMultipartUpload>");
        let mut req = self
            .inner
            .http
            .request(Method::POST, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base))
            .header("Content-Type", header_value("application/xml"));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.body(xml).send().await.map_err(map_network)?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }

    async fn abort_mp_once(&self, key: &str, uid: &str) -> XResult<()> {
        let c = self.inner.credential_provider.get_credentials().await?;
        let url = {
            let mut u = obj_url(&self.inner.base, key)?;
            u.query_pairs_mut().append_pair("uploadId", uid);
            u
        };
        let d = gmt_now();
        let r = sign::canonicalized_resource_with_subresources(
            &self.inner.config.bucket,
            key,
            &[("uploadId", Some(uid))],
        );
        let sig = sign::sign_v1(&c.access_key_secret, "DELETE", "", "", &d, "", &r);
        let mut req = self
            .inner
            .http
            .request(Method::DELETE, url)
            .header(
                "Authorization",
                header_value(&sign::authorization_header(&c.access_key_id, &sig)),
            )
            .header("Date", header_value(&d))
            .header("Host", host_header(&self.inner.base));
        if let Some(t) = &c.security_token {
            req = req.header("x-oss-security-token", header_value(t));
        }
        let resp = req.send().await.map_err(map_network)?;
        if resp.status().is_success()
            || resp.status() == StatusCode::NO_CONTENT
            || resp.status() == StatusCode::NOT_FOUND
        {
            Ok(())
        } else {
            let s = resp.status();
            let b = read_body(resp, self.inner.config.max_error_body_bytes).await;
            Err(status_error(s, &b))
        }
    }
}

#[async_trait::async_trait]
impl ObjectStore for OssPool {
    async fn put_object(&self, k: &str, d: Bytes) -> XResult<()> {
        OssPool::put_object(self, k, d).await
    }
    async fn get_object(&self, k: &str) -> XResult<Bytes> {
        OssPool::get_object(self, k).await
    }
}

// ── 辅助函数 ────────────────────────────────────────────────────────────────

fn virtual_host_base(ep: &str, b: &str) -> XResult<Url> {
    let ep = ep.trim_end_matches('/');
    let u = if let Some(h) = ep.strip_prefix("https://") {
        format!("https://{b}.{h}")
    } else if let Some(h) = ep.strip_prefix("http://") {
        format!("http://{b}.{h}")
    } else {
        return Err(XError::invalid("scheme required"));
    };
    Url::parse(&u).map_err(|e| XError::invalid(format!("invalid url: {e}")))
}
fn obj_url(base: &Url, key: &str) -> XResult<Url> {
    let mut u = base.clone();
    {
        u.path_segments_mut().map_err(|_| XError::invalid("url path"))?.clear();
    }
    for s in key.split('/').filter(|s| !s.is_empty()) {
        u.path_segments_mut().map_err(|_| XError::invalid("url path"))?.push(s);
    }
    Ok(u)
}
fn host_header(base: &Url) -> HeaderValue {
    header_value(base.host_str().unwrap_or(""))
}
fn gmt_now() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
fn header_value(v: &str) -> HeaderValue {
    HeaderValue::from_str(v).unwrap_or_else(|_| HeaderValue::from_static(""))
}
async fn read_body(resp: reqwest::Response, limit: usize) -> Bytes {
    use bytes::BufMut;
    let mut buf = bytes::BytesMut::with_capacity(limit.min(64 * 1024));
    let mut s = resp;
    while buf.len() < limit {
        match s.chunk().await {
            Ok(Some(c)) => {
                let r = limit - buf.len();
                buf.put(if c.len() <= r { c } else { c.slice(..r) });
            }
            _ => break,
        }
    }
    buf.freeze()
}
fn map_network(e: reqwest::Error) -> XError {
    if e.is_timeout() {
        XError::deadline_exceeded(format!("timeout: {e}"))
    } else {
        XError::transient(format!("network: {e}"))
    }
}
fn status_error(status: reqwest::StatusCode, body: &[u8]) -> XError {
    let p: String = String::from_utf8_lossy(body).chars().take(512).collect();
    match status.as_u16() {
        401 | 403 => XError::unavailable(format!("HTTP {status}: {p}")),
        404 => XError::missing(format!("HTTP 404: {p}")),
        s if (500..=599).contains(&s) => XError::transient(format!("HTTP {status}: {p}")),
        s if (400..=499).contains(&s) => XError::invalid(format!("HTTP {status}: {p}")),
        _ => XError::unavailable(format!("HTTP {status}: {p}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tp() -> OssPool {
        OssPool::connect(
            OssConfig::builder()
                .endpoint("https://oss.example.com")
                .bucket("test-bucket")
                .access_key_id("test-id")
                .access_key_secret("test-secret")
                .region("ap-northeast-1")
                .build()
                .unwrap(),
        )
        .unwrap()
    }
    #[test]
    fn vhost() {
        assert!(
            virtual_host_base("https://oss.example.com", "b")
                .unwrap()
                .host_str()
                .unwrap()
                .contains("b.")
        );
    }
    #[test]
    fn stats() {
        let s = tp().stats();
        assert!(!s.closed);
        assert_eq!(s.max_in_flight, 64);
    }
    #[tokio::test]
    async fn close_rejects() {
        let p = tp();
        p.close(Duration::from_secs(1)).await.unwrap();
        assert!(p.put_object("k", Bytes::from("v")).await.is_err());
    }
}
