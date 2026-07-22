//! 生产 `OssClient`：reqwest + OSS V1 签名 + multipart + resiliencx 重试。

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use chrono::Utc;
use contracts::ObjectStore;
use kernel::{ErrorKind, XError, XResult};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, DATE, ETAG, HeaderMap, HeaderName, HeaderValue,
};
use reqwest::{Client, Method, StatusCode};
use resiliencx::RetryConfig;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, timeout};
use url::Url;

use crate::config::OssConfig;
use crate::retry::{MAX_RETRY_ATTEMPTS, default_retry_config, with_retry_deadline};
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
    closed: AtomicBool,
    retry: RetryConfig,
    permits: Arc<Semaphore>,
    orphan_audits: Mutex<VecDeque<MultipartOrphanAudit>>,
    orphan_audit_overflow: AtomicU64,
}

/// 阿里云 OSS multipart 最小非末片大小（100 KiB）。
pub const MIN_MULTIPART_PART_BYTES: usize = 100 * 1024;
/// 阿里云 OSS 对象 key 的 UTF-8 字节硬上界。
pub const MAX_OBJECT_KEY_BYTES: usize = 1_023;
/// 阿里云 OSS multipart 最大分片数。
pub const MAX_MULTIPART_PARTS: usize = 10_000;
/// 本客户端允许的最大单片大小（512 MiB），受内存缓冲硬上界约束。
pub const MAX_MULTIPART_PART_BYTES: usize = 512 * 1024 * 1024;
const MAX_UPLOAD_ID_BYTES: usize = 2 * 1024;
const MAX_ETAG_BYTES: usize = 1024;
/// 进程内保留的 multipart orphan 审计记录硬上界。
pub const ORPHAN_AUDIT_CAPACITY: usize = 1_024;

/// multipart future 被取消或清理失败后留下的补偿记录。
///
/// UploadId 不是凭据，但仍属于运维敏感标识；调用方只应将其交给受控清理流程，禁止写入
/// 公共日志或低基数指标标签。
#[derive(Clone, PartialEq, Eq)]
pub struct MultipartOrphanAudit {
    key: String,
    upload_id: String,
}

impl fmt::Debug for MultipartOrphanAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultipartOrphanAudit")
            .field("key", &"<redacted>")
            .field("upload_id", &"<redacted>")
            .finish()
    }
}

impl MultipartOrphanAudit {
    /// 待清理对象 key；仅交给受控补偿流程。
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 待清理 UploadId；仅交给 [`OssClient::abort_multipart`]。
    #[must_use]
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
}

struct MultipartAuditGuard {
    inner: Arc<Inner>,
    audit: Option<MultipartOrphanAudit>,
}

impl MultipartAuditGuard {
    fn new(client: &OssClient, key: &str, upload_id: &str) -> Self {
        Self {
            inner: Arc::clone(&client.inner),
            audit: Some(MultipartOrphanAudit {
                key: key.to_owned(),
                upload_id: upload_id.to_owned(),
            }),
        }
    }

    fn disarm(&mut self) {
        self.audit = None;
    }
}

impl Drop for MultipartAuditGuard {
    fn drop(&mut self) {
        let Some(audit) = self.audit.take() else {
            return;
        };
        let mut audits = match self.inner.orphan_audits.lock() {
            Ok(audits) => audits,
            Err(poisoned) => poisoned.into_inner(),
        };
        if audits.len() < ORPHAN_AUDIT_CAPACITY {
            audits.push_back(audit);
        } else {
            self.inner.orphan_audit_overflow.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl fmt::Debug for OssClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OssClient")
            .field("config", &self.inner.config)
            .field("base", &self.inner.base.as_str())
            .field("closed", &self.inner.closed.load(Ordering::Relaxed))
            .field("retry_max_attempts", &self.inner.retry.max_attempts)
            .field("available_permits", &self.inner.permits.available_permits())
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
        if retry.max_attempts == 0 || retry.max_attempts > MAX_RETRY_ATTEMPTS {
            return Err(XError::invalid(format!(
                "oss retry max_attempts 必须在 1..={MAX_RETRY_ATTEMPTS} 范围内"
            )));
        }
        let http = Client::builder()
            .timeout(config.request_timeout)
            .pool_max_idle_per_host(config.max_in_flight)
            .user_agent(concat!("ossx/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| XError::unavailable(format!("http client: {e}")))?;
        let base = virtual_host_base(&config.endpoint, &config.bucket)?;
        let permits = Arc::new(Semaphore::new(config.max_in_flight));
        Ok(Self {
            inner: Arc::new(Inner {
                http,
                config,
                base,
                closed: AtomicBool::new(false),
                retry,
                permits,
                orphan_audits: Mutex::new(VecDeque::new()),
                orphan_audit_overflow: AtomicU64::new(0),
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

    /// 返回当前进程已发现的 multipart orphan 候选快照。
    ///
    /// 记录在成功补偿前不会自动删除；调用方可据此执行受控 `abort_multipart`。
    #[must_use]
    pub fn multipart_orphan_audits(&self) -> Vec<MultipartOrphanAudit> {
        let audits = match self.inner.orphan_audits.lock() {
            Ok(audits) => audits,
            Err(poisoned) => poisoned.into_inner(),
        };
        audits.iter().cloned().collect()
    }

    /// 审计队列达到硬上界后未能保存详细记录的累计数。
    #[must_use]
    pub fn orphan_audit_overflow_count(&self) -> u64 {
        self.inner.orphan_audit_overflow.load(Ordering::Relaxed)
    }

    /// 标记关闭（HTTP 连接池随 drop 释放；幂等）。
    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.permits.close();
    }

    fn ensure_open(&self) -> XResult<()> {
        if self.inner.closed.load(Ordering::SeqCst) {
            return Err(XError::cancelled("oss client 已关闭"));
        }
        Ok(())
    }

    async fn acquire(&self) -> XResult<OwnedSemaphorePermit> {
        self.ensure_open()?;
        match timeout(self.inner.config.acquire_timeout, self.inner.permits.clone().acquire_owned())
            .await
        {
            Ok(Ok(permit)) => {
                if self.inner.closed.load(Ordering::SeqCst) {
                    drop(permit);
                    return Err(XError::cancelled("oss client 已关闭"));
                }
                Ok(permit)
            }
            Ok(Err(_)) => Err(XError::cancelled("oss in-flight 信号量已关闭")),
            Err(_) => Err(XError::deadline_exceeded(format!(
                "oss 获取 in-flight 许可超时（max={}）",
                self.inner.config.max_in_flight
            ))),
        }
    }

    fn validate_object_size(&self, size: usize) -> XResult<()> {
        if size > self.inner.config.max_object_bytes {
            return Err(XError::invalid(format!(
                "oss 对象大小 {size} 超过上限 {}",
                self.inner.config.max_object_bytes
            )));
        }
        if size > self.inner.config.max_buffer_bytes {
            return Err(XError::invalid(format!(
                "oss 缓冲大小 {size} 超过上限 {}",
                self.inner.config.max_buffer_bytes
            )));
        }
        Ok(())
    }

    /// 上传对象（含重试）。
    pub async fn put_object(&self, key: &str, data: Bytes) -> XResult<()> {
        self.validate_object_size(data.len())?;
        let key = key.to_string();
        let data = data;
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_deadline(&retry, "put_object", self.inner.config.operation_deadline, move || {
            let this = this.clone();
            let key = key.clone();
            let data = data.clone();
            async move { this.put_object_once(&key, data).await }
        })
        .await
    }

    async fn put_object_once(&self, key: &str, data: Bytes) -> XResult<()> {
        let _permit = self.acquire().await?;
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
            .body(data)
            .send()
            .await
            .map_err(|e| map_network("PUT", &e))?;

        map_status(
            "PUT",
            key.as_str(),
            resp.status(),
            resp,
            self.inner.config.max_error_body_bytes,
        )
        .await?;
        Ok(())
    }

    /// 下载对象（含重试）。
    pub async fn get_object(&self, key: &str) -> XResult<Bytes> {
        let key = key.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_deadline(&retry, "get_object", self.inner.config.operation_deadline, move || {
            let this = this.clone();
            let key = key.clone();
            async move { this.get_object_once(&key).await }
        })
        .await
    }

    async fn get_object_once(&self, key: &str) -> XResult<Bytes> {
        let _permit = self.acquire().await?;
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
            let body =
                read_limited_body(resp, self.inner.config.max_error_body_bytes, "GET error body")
                    .await?;
            return Err(status_error("GET", &key, status, &String::from_utf8_lossy(&body)));
        }
        let limit = self.inner.config.max_object_bytes.min(self.inner.config.max_buffer_bytes);
        read_limited_body(resp, limit, "GET object body").await
    }

    /// 删除对象（幂等：不存在亦视为成功；含重试）。
    pub async fn delete_object(&self, key: &str) -> XResult<()> {
        let key = key.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_deadline(
            &retry,
            "delete_object",
            self.inner.config.operation_deadline,
            move || {
                let this = this.clone();
                let key = key.clone();
                async move { this.delete_object_once(&key).await }
            },
        )
        .await
    }

    async fn delete_object_once(&self, key: &str) -> XResult<()> {
        let _permit = self.acquire().await?;
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
        let body =
            read_limited_body(resp, self.inner.config.max_error_body_bytes, "DELETE error body")
                .await?;
        Err(status_error("DELETE", &key, status, &String::from_utf8_lossy(&body)))
    }

    // ── Multipart ──────────────────────────────────────────────────────────

    /// 初始化分片上传，返回 `upload_id`（含重试）。
    pub async fn initiate_multipart(&self, key: &str) -> XResult<String> {
        self.initiate_multipart_with_deadline(key, self.inner.config.operation_deadline).await
    }

    async fn initiate_multipart_with_deadline(
        &self,
        key: &str,
        deadline: std::time::Duration,
    ) -> XResult<String> {
        let key = key.to_string();
        let this = self.clone();
        // 响应若在服务端成功后丢失，重试会制造不可关联的 orphan。
        let retry = RetryConfig::fixed(1, 0);
        with_retry_deadline(&retry, "initiate_multipart", deadline, move || {
            let this = this.clone();
            let key = key.clone();
            async move { this.initiate_multipart_once(&key).await }
        })
        .await
    }

    async fn initiate_multipart_once(&self, key: &str) -> XResult<String> {
        let _permit = self.acquire().await?;
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
        let body = read_limited_body(
            resp,
            self.inner.config.max_error_body_bytes,
            "InitiateMultipart XML",
        )
        .await?;
        let body = String::from_utf8(body.to_vec()).map_err(|error| {
            XError::invalid("InitiateMultipart XML 非 UTF-8").with_source(error)
        })?;
        if !status.is_success() {
            return Err(status_error("InitiateMultipart", &key, status, &body));
        }
        parse_upload_id(&body)
    }

    /// 上传单个分片，返回 ETag（含重试）。
    pub async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> XResult<String> {
        self.upload_part_with_deadline(
            key,
            upload_id,
            part_number,
            data,
            self.inner.config.operation_deadline,
        )
        .await
    }

    async fn upload_part_with_deadline(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
        deadline: std::time::Duration,
    ) -> XResult<String> {
        validate_part_number(part_number)?;
        validate_upload_id(upload_id)?;
        self.validate_object_size(data.len())?;
        if data.is_empty() || data.len() > MAX_MULTIPART_PART_BYTES {
            return Err(XError::invalid(format!(
                "multipart 分片大小必须在 1..={MAX_MULTIPART_PART_BYTES} 范围内"
            )));
        }
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_deadline(&retry, "upload_part", deadline, move || {
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
        let _permit = self.acquire().await?;
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
            .body(data)
            .send()
            .await
            .map_err(|e| map_network("UploadPart", &e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = read_limited_body(
                resp,
                self.inner.config.max_error_body_bytes,
                "UploadPart error body",
            )
            .await?;
            return Err(status_error("UploadPart", &key, status, &String::from_utf8_lossy(&body)));
        }
        let etag = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .ok_or_else(|| XError::internal("UploadPart 响应缺 ETag"))?;
        validate_etag(&etag)?;
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
        self.complete_multipart_with_deadline(
            key,
            upload_id,
            parts,
            self.inner.config.operation_deadline,
        )
        .await
    }

    async fn complete_multipart_with_deadline(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<(u32, String)>,
        deadline: std::time::Duration,
    ) -> XResult<()> {
        validate_upload_id(upload_id)?;
        validate_complete_parts(&parts)?;
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        // 响应不确定时自动重放会掩盖“对象已完成但响应丢失”的状态。
        let retry = RetryConfig::fixed(1, 0);
        with_retry_deadline(&retry, "complete_multipart", deadline, move || {
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
        let _permit = self.acquire().await?;
        let key = normalize_key(key)?;
        parts.sort_by_key(|(n, _)| *n);
        let body = build_complete_xml(&parts)?;
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

        map_status(
            "CompleteMultipart",
            key.as_str(),
            resp.status(),
            resp,
            self.inner.config.max_error_body_bytes,
        )
        .await?;
        Ok(())
    }

    /// 中止分片上传（含重试；幂等）。
    pub async fn abort_multipart(&self, key: &str, upload_id: &str) -> XResult<()> {
        let result = self
            .abort_multipart_with_deadline(key, upload_id, self.inner.config.operation_deadline)
            .await;
        if result.is_ok() {
            self.remove_orphan_audit(key, upload_id);
        }
        result
    }

    async fn abort_multipart_with_deadline(
        &self,
        key: &str,
        upload_id: &str,
        deadline: std::time::Duration,
    ) -> XResult<()> {
        validate_upload_id(upload_id)?;
        let key = key.to_string();
        let upload_id = upload_id.to_string();
        let this = self.clone();
        let retry = self.inner.retry;
        with_retry_deadline(&retry, "abort_multipart", deadline, move || {
            let this = this.clone();
            let key = key.clone();
            let upload_id = upload_id.clone();
            async move { this.abort_multipart_once(&key, &upload_id).await }
        })
        .await
    }

    async fn abort_multipart_once(&self, key: &str, upload_id: &str) -> XResult<()> {
        let _permit = self.acquire().await?;
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
        let body = read_limited_body(
            resp,
            self.inner.config.max_error_body_bytes,
            "AbortMultipart error body",
        )
        .await?;
        Err(status_error("AbortMultipart", &key, status, &String::from_utf8_lossy(&body)))
    }

    /// 高层：按 `part_size` 切分并完成 multipart 上传。
    ///
    /// - 数据为空 → `Invalid`
    /// - 单片（`data.len() <= part_size`）仍走 multipart 路径
    /// - 任一分片失败时尝试 `abort_multipart`
    /// - abort 失败会返回带 `orphan_risk=true` 的 `Conflict`，禁止静默丢失孤儿风险
    ///
    /// 整个状态机共享一个 operation deadline。调用方 drop future 时同步写入有界 orphan 审计
    /// 注册表，可通过 [`Self::multipart_orphan_audits`] 取得 key/UploadId 后显式补偿。注册表不替代
    /// 服务端 lifecycle；STS 与 lifecycle 仍为 OPEN。
    pub async fn put_object_multipart(
        &self,
        key: &str,
        data: Bytes,
        part_size: usize,
    ) -> XResult<()> {
        let started = Instant::now();
        let total_deadline = self.inner.config.operation_deadline;
        self.validate_object_size(data.len())?;
        let part_count = validate_multipart_plan(data.len(), part_size)?;
        if part_size > self.inner.config.max_buffer_bytes {
            return Err(XError::invalid(format!(
                "multipart part_size {part_size} 超过缓冲上限 {}",
                self.inner.config.max_buffer_bytes
            )));
        }
        let chunks = split_parts(&data, part_size);
        debug_assert_eq!(chunks.len(), part_count);

        let initiate_deadline = remaining_deadline(started, total_deadline, "multipart initiate")?;
        let upload_id = self
            .initiate_multipart_with_deadline(key, initiate_deadline)
            .await
            .map_err(mark_unknown_initiate_orphan_risk)?;
        let mut audit_guard = MultipartAuditGuard::new(self, key, &upload_id);
        let mut completed: Vec<(u32, String)> = Vec::with_capacity(chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let part_number =
                u32::try_from(i + 1).map_err(|_| XError::invalid("multipart part_number 溢出"))?;
            // 拷贝 chunk 为 Bytes（分片重试需要所有权）
            let part_data = Bytes::copy_from_slice(chunk);
            let remaining = match remaining_deadline(started, total_deadline, "multipart upload") {
                Ok(value) => value,
                Err(error) => {
                    return Err(self
                        .cleanup_multipart_failure(
                            key,
                            &upload_id,
                            error,
                            started,
                            total_deadline,
                            &mut audit_guard,
                        )
                        .await);
                }
            };
            match self
                .upload_part_with_deadline(key, &upload_id, part_number, part_data, remaining)
                .await
            {
                Ok(etag) => completed.push((part_number, etag)),
                Err(error) => {
                    return Err(self
                        .cleanup_multipart_failure(
                            key,
                            &upload_id,
                            error,
                            started,
                            total_deadline,
                            &mut audit_guard,
                        )
                        .await);
                }
            }
        }
        let remaining = match remaining_deadline(started, total_deadline, "multipart complete") {
            Ok(value) => value,
            Err(error) => {
                return Err(self
                    .cleanup_multipart_failure(
                        key,
                        &upload_id,
                        error,
                        started,
                        total_deadline,
                        &mut audit_guard,
                    )
                    .await);
            }
        };
        if let Err(error) =
            self.complete_multipart_with_deadline(key, &upload_id, completed, remaining).await
        {
            return Err(self
                .cleanup_multipart_failure(
                    key,
                    &upload_id,
                    error,
                    started,
                    total_deadline,
                    &mut audit_guard,
                )
                .await);
        }
        audit_guard.disarm();
        Ok(())
    }

    async fn cleanup_multipart_failure(
        &self,
        key: &str,
        upload_id: &str,
        primary: XError,
        started: Instant,
        total_deadline: std::time::Duration,
        audit_guard: &mut MultipartAuditGuard,
    ) -> XError {
        let Ok(remaining) = remaining_deadline(started, total_deadline, "multipart abort") else {
            return mark_known_orphan_risk(primary, upload_id);
        };
        let abort = self.abort_multipart_with_deadline(key, upload_id, remaining).await;
        if abort.is_ok() {
            audit_guard.disarm();
        }
        merge_abort_result(primary, abort, upload_id)
    }

    fn remove_orphan_audit(&self, key: &str, upload_id: &str) {
        let mut audits = match self.inner.orphan_audits.lock() {
            Ok(audits) => audits,
            Err(poisoned) => poisoned.into_inner(),
        };
        audits.retain(|audit| audit.key != key || audit.upload_id != upload_id);
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
    let mut ep = Url::parse(endpoint.trim_end_matches('/'))
        .map_err(|e| XError::invalid(format!("bad endpoint URL: {e}")))?;
    let host = ep.host_str().ok_or_else(|| XError::invalid("endpoint missing host"))?;
    let virtual_host = format!("{bucket}.{host}");
    ep.set_host(Some(&virtual_host)).map_err(|_| XError::invalid("virtual host URL 非法"))?;
    ep.set_path("/");
    Ok(ep)
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
    if k.len() > MAX_OBJECT_KEY_BYTES {
        return Err(XError::invalid(format!("object key 超过 {MAX_OBJECT_KEY_BYTES} 字节上限")));
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
    max_error_body_bytes: usize,
) -> XResult<()> {
    if status.is_success() {
        return Ok(());
    }
    let body = read_limited_body(resp, max_error_body_bytes, "OSS error body").await?;
    Err(status_error(op, key, status, &String::from_utf8_lossy(&body)))
}

async fn read_limited_body(
    mut response: reqwest::Response,
    limit: usize,
    label: &str,
) -> XResult<Bytes> {
    if let Some(length) = response.content_length() {
        let length = usize::try_from(length)
            .map_err(|_| XError::invalid(format!("{label} Content-Length 超出平台范围")))?;
        if length > limit {
            return Err(XError::invalid(format!("{label} 大小 {length} 超过上限 {limit}")));
        }
    }
    let mut body = BytesMut::with_capacity(limit.min(8 * 1024));
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| XError::transient(format!("oss {label} 读取失败: {error}")))?
    {
        append_limited(&mut body, &chunk, limit, label)?;
    }
    Ok(body.freeze())
}

fn append_limited(body: &mut BytesMut, chunk: &[u8], limit: usize, label: &str) -> XResult<()> {
    let next_len = body
        .len()
        .checked_add(chunk.len())
        .ok_or_else(|| XError::invalid(format!("{label} 大小溢出")))?;
    if next_len > limit {
        return Err(XError::invalid(format!("{label} 流式读取超过上限 {limit}")));
    }
    body.extend_from_slice(chunk);
    Ok(())
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

/// 从 InitiateMultipartUploadResult XML 中提取并校验 UploadId。
fn parse_upload_id(xml: &str) -> XResult<String> {
    const OPEN: &str = "<UploadId>";
    const CLOSE: &str = "</UploadId>";
    let start = xml
        .find(OPEN)
        .map(|index| index + OPEN.len())
        .ok_or_else(|| XError::internal("InitiateMultipart 响应缺 UploadId"))?;
    let end = xml[start..]
        .find(CLOSE)
        .map(|index| index + start)
        .ok_or_else(|| XError::internal("InitiateMultipart UploadId 未闭合"))?;
    let id = xml[start..end].trim();
    validate_upload_id(id)?;
    Ok(id.to_owned())
}

/// 构造 CompleteMultipartUpload XML。
fn build_complete_xml(parts: &[(u32, String)]) -> XResult<String> {
    validate_complete_parts(parts)?;
    let mut xml = String::from("<CompleteMultipartUpload>");
    for (n, etag) in parts {
        xml.push_str("<Part><PartNumber>");
        xml.push_str(&n.to_string());
        xml.push_str("</PartNumber><ETag>");
        xml.push_str(&escape_xml_text(etag));
        xml.push_str("</ETag></Part>");
    }
    xml.push_str("</CompleteMultipartUpload>");
    Ok(xml)
}

fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn validate_upload_id(upload_id: &str) -> XResult<()> {
    if upload_id.is_empty()
        || upload_id.len() > MAX_UPLOAD_ID_BYTES
        || upload_id.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | '&' | '\"' | '\'')
        })
    {
        return Err(XError::invalid("multipart upload_id 非法或超过上限"));
    }
    Ok(())
}

fn validate_etag(etag: &str) -> XResult<()> {
    if etag.is_empty() || etag.len() > MAX_ETAG_BYTES || etag.chars().any(char::is_control) {
        return Err(XError::invalid("multipart ETag 非法或超过上限"));
    }
    Ok(())
}

fn validate_part_number(part_number: u32) -> XResult<()> {
    let max = u32::try_from(MAX_MULTIPART_PARTS)
        .map_err(|error| XError::internal("multipart part 上限转换失败").with_source(error))?;
    if part_number == 0 || part_number > max {
        return Err(XError::invalid(format!("part_number 必须在 1..={max} 范围内")));
    }
    Ok(())
}

fn validate_complete_parts(parts: &[(u32, String)]) -> XResult<()> {
    if parts.is_empty() || parts.len() > MAX_MULTIPART_PARTS {
        return Err(XError::invalid(format!(
            "complete_multipart part 数必须在 1..={MAX_MULTIPART_PARTS} 范围内"
        )));
    }
    let mut seen = HashSet::with_capacity(parts.len());
    for (part_number, etag) in parts {
        validate_part_number(*part_number)?;
        validate_etag(etag)?;
        if !seen.insert(*part_number) {
            return Err(XError::invalid(format!(
                "complete_multipart 含重复 part_number={part_number}"
            )));
        }
    }
    Ok(())
}

fn validate_multipart_plan(object_size: usize, part_size: usize) -> XResult<usize> {
    if object_size == 0 {
        return Err(XError::invalid("multipart 数据不能为空"));
    }
    if part_size == 0 || part_size > MAX_MULTIPART_PART_BYTES {
        return Err(XError::invalid(format!(
            "multipart part_size 必须在 1..={MAX_MULTIPART_PART_BYTES} 范围内"
        )));
    }
    if object_size > part_size && part_size < MIN_MULTIPART_PART_BYTES {
        return Err(XError::invalid(format!(
            "multipart 非末片不得小于 {MIN_MULTIPART_PART_BYTES} 字节"
        )));
    }
    let part_count = object_size.div_ceil(part_size);
    if part_count > MAX_MULTIPART_PARTS {
        return Err(XError::invalid(format!(
            "multipart 分片数 {part_count} 超过上限 {MAX_MULTIPART_PARTS}"
        )));
    }
    Ok(part_count)
}

fn remaining_deadline(
    started: Instant,
    total: std::time::Duration,
    op: &str,
) -> XResult<std::time::Duration> {
    total.checked_sub(started.elapsed()).filter(|remaining| !remaining.is_zero()).ok_or_else(|| {
        XError::deadline_exceeded(format!(
            "oss {op} 超过 multipart 总 deadline {}ms",
            total.as_millis()
        ))
    })
}

fn mark_unknown_initiate_orphan_risk(error: XError) -> XError {
    if matches!(error.kind(), ErrorKind::Transient | ErrorKind::DeadlineExceeded) {
        return XError::conflict(format!(
            "multipart orphan_risk=true upload_id=unknown; initiate={error}"
        ));
    }
    error
}

fn mark_known_orphan_risk(primary: XError, upload_id: &str) -> XError {
    let context = format!(
        "multipart orphan_risk=true upload_id={upload_id}; cleanup deadline exhausted; primary={primary}"
    );
    match primary.kind() {
        ErrorKind::DeadlineExceeded => XError::deadline_exceeded(context),
        ErrorKind::Cancelled => XError::cancelled(context),
        _ => XError::conflict(context),
    }
}

fn merge_abort_result(primary: XError, abort: XResult<()>, upload_id: &str) -> XError {
    match abort {
        Ok(()) => primary,
        Err(abort_error) => XError::conflict(format!(
            "multipart orphan_risk=true upload_id={upload_id}; primary={primary}; abort={abort_error}"
        )),
    }
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
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::oneshot;

    async fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut request = Vec::new();
        let mut buffer = [0u8; 1024];
        let mut expected_len = None;
        loop {
            let read = stream.read(&mut buffer).await.expect("read request");
            assert!(read > 0, "request closed before complete");
            request.extend_from_slice(&buffer[..read]);
            if expected_len.is_none()
                && let Some(header_end) =
                    request.windows(4).position(|window| window == b"\r\n\r\n")
            {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                expected_len = Some(header_end + 4 + content_length);
            }
            if expected_len.is_some_and(|length| request.len() >= length) {
                return request;
            }
        }
    }

    async fn write_response(stream: &mut TcpStream, status: &str, headers: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.expect("write response");
    }

    async fn loopback_client(
        request_timeout: Duration,
        operation_deadline: Duration,
    ) -> (TcpListener, OssClient) {
        let listener = TcpListener::bind("[::1]:0").await.expect("bind loopback");
        let port = listener.local_addr().expect("local address").port();
        let config = OssConfig::builder()
            .endpoint(format!("http://localhost:{port}"))
            .bucket("bucket")
            .access_key_id("id")
            .access_key_secret("sec")
            .request_timeout(request_timeout)
            .operation_deadline(operation_deadline)
            .build()
            .expect("config");
        (listener, OssClient::connect(config).expect("client"))
    }

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
        assert!(normalize_key(&"x".repeat(MAX_OBJECT_KEY_BYTES)).is_ok());
        assert!(normalize_key(&"x".repeat(MAX_OBJECT_KEY_BYTES + 1)).is_err());
    }

    #[test]
    fn parse_upload_id_from_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<InitiateMultipartUploadResult>
  <Bucket>b</Bucket>
  <Key>k</Key>
  <UploadId>0004B9894A22E5B1888A1E29F8236E2D</UploadId>
</InitiateMultipartUploadResult>"#;
        assert_eq!(parse_upload_id(xml).expect("upload id"), "0004B9894A22E5B1888A1E29F8236E2D");
        assert!(parse_upload_id("<root/>").is_err());
        assert!(parse_upload_id("<UploadId>&xxe;</UploadId>").is_err());
    }

    #[test]
    fn complete_xml_orders_parts() {
        let xml = build_complete_xml(&[(1, "\"etag1\"".into()), (2, "\"etag2\"".into())])
            .expect("complete XML");
        assert!(xml.contains("<PartNumber>1</PartNumber>"));
        assert!(xml.contains("<ETag>&quot;etag1&quot;</ETag>"));
        assert!(xml.starts_with("<CompleteMultipartUpload>"));
        assert!(xml.ends_with("</CompleteMultipartUpload>"));
    }

    #[test]
    fn complete_xml_escapes_etag_and_rejects_duplicate_parts() {
        let xml = build_complete_xml(&[(1, "\"a&<b>\"".into())]).expect("合法 ETag");
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&gt;"));
        assert!(!xml.contains("<ETag>\"a&<b>\"</ETag>"));

        let error = build_complete_xml(&[(1, "a".into()), (1, "b".into())])
            .expect_err("重复 part_number 必须被拒绝");
        assert!(error.context().contains("重复"));
    }

    #[test]
    fn multipart_plan_enforces_part_size_and_count() {
        assert!(validate_multipart_plan(MIN_MULTIPART_PART_BYTES + 1, 0).is_err());
        assert!(validate_multipart_plan(MIN_MULTIPART_PART_BYTES + 1, 1).is_err());
        assert!(validate_multipart_plan(1, 1).is_ok());
        let oversized = (MAX_MULTIPART_PARTS + 1) * MIN_MULTIPART_PART_BYTES;
        assert!(validate_multipart_plan(oversized, MIN_MULTIPART_PART_BYTES).is_err());
    }

    #[test]
    fn chunked_body_buffer_stops_at_hard_limit() {
        let mut body = BytesMut::new();
        append_limited(&mut body, b"abcd", 5, "error body").expect("first chunk");
        let error = append_limited(&mut body, b"ef", 5, "error body")
            .expect_err("chunked body 必须在追加前拒绝超限");
        assert!(error.context().contains("上限 5"));
        assert_eq!(&body[..], b"abcd");
    }

    #[test]
    fn abort_failure_marks_orphan_risk() {
        let primary = XError::transient("upload failed");
        let abort = Err(XError::transient("abort failed"));
        let error = merge_abort_result(primary, abort, "upload-123");
        assert_eq!(error.kind(), kernel::ErrorKind::Conflict);
        assert!(error.context().contains("orphan_risk=true"));
        assert!(error.context().contains("upload-123"));
    }

    #[test]
    fn connect_validates_config() {
        let err = OssConfig::builder()
            .endpoint("")
            .bucket("b")
            .access_key_id("id")
            .access_key_secret("sec")
            .region("r")
            .build();
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn acquire_is_bounded_by_concurrency_and_timeout() {
        let config = OssConfig::builder()
            .endpoint("https://oss.example.com")
            .bucket("bucket")
            .access_key_id("id")
            .access_key_secret("sec")
            .max_in_flight(1)
            .acquire_timeout(Duration::from_millis(5))
            .build()
            .expect("config");
        let client = OssClient::connect(config).expect("client");
        let permit = client.acquire().await.expect("first permit");
        let error = client.acquire().await.expect_err("second permit must time out");
        assert_eq!(error.kind(), kernel::ErrorKind::DeadlineExceeded);
        drop(permit);
        let _permit = client.acquire().await.expect("permit released");
    }

    #[tokio::test]
    async fn close_is_a_cancelled_boundary() {
        let config = OssConfig::builder()
            .endpoint("https://oss.example.com")
            .bucket("bucket")
            .access_key_id("id")
            .access_key_secret("sec")
            .build()
            .expect("config");
        let client = OssClient::connect(config).expect("client");
        client.close();
        let error = client.acquire().await.expect_err("closed client must reject acquire");
        assert_eq!(error.kind(), kernel::ErrorKind::Cancelled);
    }

    #[test]
    fn orphan_registry_capacity_and_overflow_are_bounded() {
        let config = OssConfig::builder()
            .endpoint("https://oss.example.com")
            .bucket("bucket")
            .access_key_id("id")
            .access_key_secret("sec")
            .build()
            .expect("config");
        let client = OssClient::connect(config).expect("client");
        for index in 0..=ORPHAN_AUDIT_CAPACITY {
            drop(MultipartAuditGuard::new(&client, "object", &format!("upload-{index}")));
        }
        assert_eq!(client.multipart_orphan_audits().len(), ORPHAN_AUDIT_CAPACITY);
        assert_eq!(client.orphan_audit_overflow_count(), 1);
    }

    #[tokio::test]
    async fn chunked_error_body_is_bounded_over_real_http() {
        let listener = TcpListener::bind("[::1]:0").await.expect("bind loopback");
        let port = listener.local_addr().expect("local address").port();
        let config = OssConfig::builder()
            .endpoint(format!("http://localhost:{port}"))
            .bucket("bucket")
            .access_key_id("id")
            .access_key_secret("sec")
            .max_error_body_bytes(5)
            .build()
            .expect("config");
        let client = OssClient::connect(config).expect("client");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept GET");
            let _ = read_http_request(&mut stream).await;
            stream
                .write_all(
                    b"HTTP/1.1 500 Internal Server Error\r\nConnection: close\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nabcd\r\n2\r\nef\r\n0\r\n\r\n",
                )
                .await
                .expect("write chunked error");
        });

        let error = client.get_object("object").await.expect_err("body must exceed cap");
        assert_eq!(error.kind(), kernel::ErrorKind::Invalid);
        assert!(error.context().contains("上限 5"));
        server.await.expect("server");
    }

    #[tokio::test]
    async fn cancelling_after_initiate_records_recoverable_upload_id() {
        let (listener, client) =
            loopback_client(Duration::from_secs(5), Duration::from_secs(5)).await;
        let (part_started_tx, part_started_rx) = oneshot::channel();
        let (allow_cleanup_tx, allow_cleanup_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut initiate, _) = listener.accept().await.expect("accept initiate");
            let request = read_http_request(&mut initiate).await;
            assert!(String::from_utf8_lossy(&request).starts_with("POST /object?uploads HTTP/1.1"));
            let body = "<InitiateMultipartUploadResult><UploadId>recoverable-123</UploadId></InitiateMultipartUploadResult>";
            write_response(&mut initiate, "200 OK", "Content-Type: application/xml\r\n", body)
                .await;

            let (mut part, _) = listener.accept().await.expect("accept part");
            let request = read_http_request(&mut part).await;
            assert!(String::from_utf8_lossy(&request).starts_with("PUT /object?"));
            let _ = part_started_tx.send(());
            let _ = allow_cleanup_rx.await;
            drop(part);

            let (mut abort, _) = listener.accept().await.expect("accept abort");
            let request = read_http_request(&mut abort).await;
            let request = String::from_utf8_lossy(&request);
            assert!(request.starts_with("DELETE /object?"));
            assert!(request.contains("uploadId=recoverable-123"));
            write_response(&mut abort, "204 No Content", "", "").await;
        });

        let task_client = client.clone();
        let upload = tokio::spawn(async move {
            task_client.put_object_multipart("object", Bytes::from_static(b"x"), 1).await
        });
        part_started_rx.await.expect("part started");
        upload.abort();
        let join_error = upload.await.expect_err("upload task must be cancelled");
        assert!(join_error.is_cancelled());

        let audits = client.multipart_orphan_audits();
        assert_eq!(audits.len(), 1);
        assert_eq!(audits[0].key(), "object");
        assert_eq!(audits[0].upload_id(), "recoverable-123");
        assert!(!format!("{:?}", audits[0]).contains("recoverable-123"));
        assert_eq!(client.orphan_audit_overflow_count(), 0);
        let _ = allow_cleanup_tx.send(());
        client
            .abort_multipart(audits[0].key(), audits[0].upload_id())
            .await
            .expect("audit record must support compensating abort");
        assert!(client.multipart_orphan_audits().is_empty());
        server.await.expect("server");
    }

    #[tokio::test]
    async fn multipart_uses_one_total_deadline_across_parts() {
        let total_deadline = Duration::from_millis(100);
        let (listener, client) = loopback_client(Duration::from_millis(80), total_deadline).await;
        let server = tokio::spawn(async move {
            let (mut initiate, _) = listener.accept().await.expect("accept initiate");
            let _ = read_http_request(&mut initiate).await;
            let body = "<InitiateMultipartUploadResult><UploadId>deadline-123</UploadId></InitiateMultipartUploadResult>";
            write_response(&mut initiate, "200 OK", "Content-Type: application/xml\r\n", body)
                .await;

            let (mut first_part, _) = listener.accept().await.expect("accept first part");
            let _ = read_http_request(&mut first_part).await;
            tokio::time::sleep(Duration::from_millis(60)).await;
            write_response(&mut first_part, "200 OK", "ETag: etag-1\r\n", "").await;

            let (mut second_part, _) = listener.accept().await.expect("accept second part");
            let _ = read_http_request(&mut second_part).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        });

        let data = Bytes::from(vec![b'x'; MIN_MULTIPART_PART_BYTES + 1]);
        let started = Instant::now();
        let error = client
            .put_object_multipart("object", data, MIN_MULTIPART_PART_BYTES)
            .await
            .expect_err("whole multipart must respect one deadline");
        assert!(started.elapsed() < Duration::from_millis(300));
        assert!(error.context().contains("orphan_risk=true"));
        assert_eq!(client.multipart_orphan_audits().len(), 1);
        server.abort();
    }
}
