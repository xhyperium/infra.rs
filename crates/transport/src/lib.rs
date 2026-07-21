#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # transportx — L1 统一网络客户端抽象
//!
//! 提供驱动无关的 HTTP / WebSocket 传输边界，以及基于 reqwest / tokio-tungstenite
//! 的默认驱动。驱动私有类型（`reqwest::Client`、tungstenite stream）封装在 crate 内部。
//!
//! ## 职责
//!
//! - 统一 HTTP / WebSocket 客户端侧传输边界
//! - L1 实现，可被存储与交易所适配器依赖
//! - 只承载传输，不承载业务契约
//!
//! ## 非目标
//!
//! - 不实现重试 / 熔断 / 限流 / 调度（`resiliencx` / `schedulex`）
//! - 不成为组合根（`bootstrap`）
//! - 不依赖其他 L1 crate（R3）
//!
//! 实现合同：`.agents/ssot/infra/transport/spec/spec.md`

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use kernel::{XError, XResult};
use reqwest::{Client, Method};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Transport failures retain enough semantics for reconnect policy decisions.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// TCP / TLS 握手超时。
    #[error("connect timeout")]
    ConnectTimeout,
    /// 读响应 / 帧超时。
    #[error("read timeout")]
    ReadTimeout,
    /// 连接已关闭；`clean` 表示是否为协议层正常关闭。
    #[error("connection closed ({clean})")]
    ConnectionClosed {
        /// `true` 表示对端/本端已完成协议关闭握手。
        clean: bool,
    },
    /// HTTP 429；可选整数秒 `Retry-After`。
    #[error("rate limited{retry_after:?}")]
    RateLimited {
        /// 建议等待时长（来自整数秒 `Retry-After`）。
        retry_after: Option<Duration>,
    },
    /// 协议 / 方法 / URL 等不可恢复语义错误。
    #[error("protocol violation: {0}")]
    ProtocolViolation(String),
    /// 底层 I/O 或客户端构建失败。
    #[error("I/O error: {0}")]
    Io(#[source] Box<dyn std::error::Error + Send + Sync>),
}

// ---------------------------------------------------------------------------
// HTTP boundary
// ---------------------------------------------------------------------------

/// Request passed to an HTTP driver. Concrete HTTP clients stay behind this boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// HTTP method（如 `"GET"` / `"POST"`）。
    pub method: String,
    /// 完整请求 URL。
    pub url: String,
    /// 请求头（name, value）。
    pub headers: Vec<(String, String)>,
    /// 可选请求体。
    pub body: Option<Bytes>,
}

/// Transport-neutral HTTP response.
///
/// 当前只保留 status/body，不保留 response headers（SSOT §4.4）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// HTTP 状态码。
    pub status: u16,
    /// 响应体。
    pub body: Bytes,
}

/// Typed HTTP driver boundary.
#[async_trait]
pub trait HttpDriver: Send + Sync {
    /// 执行一次 HTTP 请求。
    ///
    /// - HTTP 429 → [`TransportError::RateLimited`]（整数秒 `Retry-After`）
    /// - 其他 4xx/5xx → [`Ok`]`(`[`HttpResponse`]`)`
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportError>;
}

// ---------------------------------------------------------------------------
// WebSocket boundary
// ---------------------------------------------------------------------------

/// WebSocket 连接器边界。
#[async_trait]
pub trait WsConnector: Send + Sync {
    /// 建立到 `url` 的 WebSocket 连接。
    async fn connect(&self, url: &str) -> Result<Box<dyn WsConnection>, TransportError>;
}

/// 已建立的 WebSocket 连接（帧级生命周期）。
#[async_trait]
pub trait WsConnection: Send + Sync {
    /// 读取下一帧。
    ///
    /// - text / binary → `Some(Bytes)`
    /// - Ping / Pong / Frame → 跳过
    /// - Close → `Ok(None)`（不保留 code/reason）
    /// - 流自然结束 → [`TransportError::ConnectionClosed`] `{ clean: false }`
    async fn next_frame(&mut self) -> Result<Option<Bytes>, TransportError>;
    /// 发送一帧 binary payload。
    async fn send_frame(&mut self, frame: Bytes) -> Result<(), TransportError>;
    /// 发送 Close 并结束发送侧。
    async fn close(&mut self) -> Result<(), TransportError>;
}

// ---------------------------------------------------------------------------
// Reqwest HTTP driver
// ---------------------------------------------------------------------------

/// reqwest-backed HTTP driver. The reqwest type remains private to this crate.
#[derive(Clone, Debug)]
pub struct ReqwestHttpDriver {
    client: Client,
}

impl ReqwestHttpDriver {
    /// Build a driver with reqwest's default timeout policy.
    pub fn new() -> Result<Self, TransportError> {
        Self::with_timeout(None)
    }

    /// Build a driver with an optional total request timeout.
    pub fn with_timeout(timeout: Option<Duration>) -> Result<Self, TransportError> {
        let mut builder = Client::builder();
        if let Some(timeout) = timeout {
            builder = builder.timeout(timeout);
        }
        let client = builder.build().map_err(map_client_build_error)?;
        Ok(Self { client })
    }
}

/// ClientBuilder::build 失败映射（构建期错误统一为 Io）。
pub(crate) fn map_client_build_error(error: reqwest::Error) -> TransportError {
    TransportError::Io(Box::new(error))
}

/// 将 reqwest 错误映射为 [`TransportError`]。
///
/// 公开为 crate 内测试可直接驱动的映射函数，保持与 execute 路径一致。
pub(crate) fn map_reqwest_error(error: reqwest::Error) -> TransportError {
    if error.is_timeout() {
        if error.is_connect() {
            TransportError::ConnectTimeout
        } else {
            TransportError::ReadTimeout
        }
    } else if error.is_builder() {
        TransportError::ProtocolViolation(error.to_string())
    } else {
        TransportError::Io(Box::new(error))
    }
}

#[async_trait]
impl HttpDriver for ReqwestHttpDriver {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        let method = Method::from_bytes(request.method.as_bytes())
            .map_err(|error| TransportError::ProtocolViolation(error.to_string()))?;
        let mut builder = self.client.request(method, &request.url);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let request = builder.build().map_err(map_reqwest_error)?;
        let response = self.client.execute(request).await.map_err(map_reqwest_error)?;
        let status = response.status();
        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(Duration::from_secs);
            return Err(TransportError::RateLimited { retry_after });
        }
        let body = response.bytes().await.map_err(map_reqwest_error)?;
        Ok(HttpResponse { status: status.as_u16(), body })
    }
}

// ---------------------------------------------------------------------------
// Tungstenite WebSocket driver
// ---------------------------------------------------------------------------

/// tokio-tungstenite-backed WebSocket connector. Driver-specific stream types
/// stay private behind [`WsConnection`].
#[derive(Debug, Default, Clone, Copy)]
pub struct TungsteniteWsConnector;

impl TungsteniteWsConnector {
    /// 创建默认连接器。
    pub const fn new() -> Self {
        Self
    }
}

type TungsteniteStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct TungsteniteWsConnection {
    stream: TungsteniteStream,
}

/// 将 tungstenite 错误映射为 [`TransportError`]。
pub(crate) fn map_tungstenite_error(
    error: tokio_tungstenite::tungstenite::Error,
) -> TransportError {
    use tokio_tungstenite::tungstenite::Error;
    match error {
        Error::ConnectionClosed | Error::AlreadyClosed => {
            TransportError::ConnectionClosed { clean: true }
        }
        Error::Io(error) => TransportError::Io(Box::new(error)),
        Error::Protocol(error) => TransportError::ProtocolViolation(error.to_string()),
        Error::Url(error) => TransportError::ProtocolViolation(error.to_string()),
        other => TransportError::ProtocolViolation(other.to_string()),
    }
}

#[async_trait]
impl WsConnector for TungsteniteWsConnector {
    async fn connect(&self, url: &str) -> Result<Box<dyn WsConnection>, TransportError> {
        let (stream, _) = connect_async(url).await.map_err(map_tungstenite_error)?;
        Ok(Box::new(TungsteniteWsConnection { stream }))
    }
}

#[async_trait]
impl WsConnection for TungsteniteWsConnection {
    async fn next_frame(&mut self) -> Result<Option<Bytes>, TransportError> {
        while let Some(message) = self.stream.next().await {
            match message.map_err(map_tungstenite_error)? {
                Message::Text(text) => return Ok(Some(Bytes::copy_from_slice(text.as_bytes()))),
                Message::Binary(bytes) => return Ok(Some(bytes)),
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
                Message::Close(_) => return Ok(None),
            }
        }
        Err(TransportError::ConnectionClosed { clean: false })
    }

    async fn send_frame(&mut self, frame: Bytes) -> Result<(), TransportError> {
        self.stream.send(Message::Binary(frame)).await.map_err(map_tungstenite_error)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.stream.send(Message::Close(None)).await.map_err(map_tungstenite_error)
    }
}

// ---------------------------------------------------------------------------
// Legacy HttpTransport + Mock
// ---------------------------------------------------------------------------

/// Legacy HTTP 传输抽象（仅为 staged compatibility 保留）。
#[async_trait]
#[deprecated(note = "use HttpDriver")]
pub trait HttpTransport: Send + Sync {
    /// 发起 GET 请求，返回响应体。
    async fn get(&self, url: &str) -> XResult<Bytes>;
    /// 发起 POST 请求，返回响应体。
    async fn post(&self, url: &str, body: Bytes) -> XResult<Bytes>;
}

/// 内存模拟 HTTP 传输（不依赖任何 HTTP 驱动）。
///
/// 通过 [`MockHttpTransport::set_get`] / [`MockHttpTransport::set_post`]
/// 预置 `(url, response)` 映射；未预置的 url 在 legacy API 返回
/// [`XError::missing`]，在 [`HttpDriver`] 返回
/// [`TransportError::ProtocolViolation`]。
/// POST 的 `body` 参数在 mock 中被忽略（不校验请求体）。
#[derive(Debug, Default)]
pub struct MockHttpTransport {
    gets: RwLock<HashMap<String, Bytes>>,
    posts: RwLock<HashMap<String, Bytes>>,
}

impl MockHttpTransport {
    /// 创建空 mock。
    pub fn new() -> Self {
        Self::default()
    }

    /// 预置 GET `url` 的响应。
    pub fn set_get(&self, url: &str, response: Bytes) {
        self.gets.write().expect("mock gets lock").insert(url.to_string(), response);
    }

    /// 预置 POST `url` 的响应。
    pub fn set_post(&self, url: &str, response: Bytes) {
        self.posts.write().expect("mock posts lock").insert(url.to_string(), response);
    }
}

#[async_trait]
#[allow(deprecated)]
impl HttpTransport for MockHttpTransport {
    async fn get(&self, url: &str) -> XResult<Bytes> {
        self.gets
            .read()
            .expect("mock gets lock")
            .get(url)
            .cloned()
            .ok_or_else(|| XError::missing(format!("mock GET {url}")))
    }

    async fn post(&self, url: &str, _body: Bytes) -> XResult<Bytes> {
        self.posts
            .read()
            .expect("mock posts lock")
            .get(url)
            .cloned()
            .ok_or_else(|| XError::missing(format!("mock POST {url}")))
    }
}

#[async_trait]
impl HttpDriver for MockHttpTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        let response = match request.method.to_ascii_uppercase().as_str() {
            "GET" => self
                .gets
                .read()
                .map_err(|_| TransportError::Io(Box::new(std::io::Error::other("mock lock"))))?
                .get(&request.url)
                .cloned(),
            "POST" => self
                .posts
                .read()
                .map_err(|_| TransportError::Io(Box::new(std::io::Error::other("mock lock"))))?
                .get(&request.url)
                .cloned(),
            method => {
                return Err(TransportError::ProtocolViolation(format!(
                    "mock method unsupported: {method}"
                )));
            }
        };
        response.map(|body| HttpResponse { status: 200, body }).ok_or_else(|| {
            TransportError::ProtocolViolation(format!(
                "mock response missing for {} {}",
                request.method, request.url
            ))
        })
    }
}

// ---------------------------------------------------------------------------
// Test hooks（doc(hidden)；供集成测试驱动错误映射与锁投毒）
// ---------------------------------------------------------------------------

/// 映射 reqwest 错误（测试钩子）。
#[doc(hidden)]
pub fn __map_reqwest_error(error: reqwest::Error) -> TransportError {
    map_reqwest_error(error)
}

/// 映射 Client::build 错误（测试钩子）。
#[doc(hidden)]
pub fn __map_client_build_error(error: reqwest::Error) -> TransportError {
    map_client_build_error(error)
}

/// 映射 tungstenite 错误（测试钩子）。
#[doc(hidden)]
pub fn __map_tungstenite_error(error: tokio_tungstenite::tungstenite::Error) -> TransportError {
    map_tungstenite_error(error)
}

impl MockHttpTransport {
    /// 投毒 GET 锁（测试钩子）。
    #[doc(hidden)]
    pub fn __poison_gets(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = self.gets.write().expect("lock");
            panic!("poison gets");
        }));
    }

    /// 投毒 POST 锁（测试钩子）。
    #[doc(hidden)]
    pub fn __poison_posts(&self) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = self.posts.write().expect("lock");
            panic!("poison posts");
        }));
    }
}
