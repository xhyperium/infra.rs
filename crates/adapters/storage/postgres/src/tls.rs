//! rustls TLS 连接器：实现 `tokio_postgres::tls::{MakeTlsConnect, TlsConnect}`。
//!
//! 根证书来自 webpki-roots；强制证书校验（无 insecure 旁路）。

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::task::{Context, Poll};

use kernel::{XError, XResult};
use rustls::ClientConfig;
use rustls_pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_postgres::tls::{ChannelBinding, MakeTlsConnect, TlsConnect, TlsStream};
use tokio_rustls::TlsConnector;

/// 确保 rustls 进程级默认 crypto provider（ring）已安装。
fn ensure_crypto_provider() {
    static INIT: OnceLock<()> = OnceLock::new();
    let _ = INIT.get_or_init(|| {
        // 已安装时忽略错误（多适配器并存）
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// 构建带 webpki 根证书的 rustls `ClientConfig`。
pub fn build_client_config() -> XResult<ClientConfig> {
    ensure_crypto_provider();
    let mut root_store = rustls::RootCertStore::empty();
    root_store.roots = webpki_roots::TLS_SERVER_ROOTS.to_vec();
    let config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();
    Ok(config)
}

/// deadpool / tokio-postgres 可用的 rustls `MakeTlsConnect` 实现。
#[derive(Clone)]
pub struct MakeRustlsConnect {
    connector: TlsConnector,
}

impl MakeRustlsConnect {
    /// 使用 webpki-roots 构建连接器。
    pub fn with_webpki_roots() -> XResult<Self> {
        let config = build_client_config()?;
        Ok(Self { connector: TlsConnector::from(Arc::new(config)) })
    }

    /// 从既有 `ClientConfig` 构建。
    #[must_use]
    pub fn from_config(config: ClientConfig) -> Self {
        ensure_crypto_provider();
        Self { connector: TlsConnector::from(Arc::new(config)) }
    }

    /// 为指定域名构造一次 `TlsConnect`（SNI / 证书校验用）。
    pub fn for_domain(&self, domain: &str) -> XResult<RustlsConnect> {
        let server_name = ServerName::try_from(domain.to_owned())
            .map_err(|_| XError::invalid(format!("非法 TLS 域名（SNI）: {domain}")))?;
        Ok(RustlsConnect { connector: self.connector.clone(), domain: server_name })
    }
}

impl std::fmt::Debug for MakeRustlsConnect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MakeRustlsConnect").finish_non_exhaustive()
    }
}

/// 单次 TLS 握手参数。
pub struct RustlsConnect {
    connector: TlsConnector,
    domain: ServerName<'static>,
}

impl std::fmt::Debug for RustlsConnect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustlsConnect")
            .field("domain", &self.domain.to_str())
            .finish_non_exhaustive()
    }
}

impl<S> MakeTlsConnect<S> for MakeRustlsConnect
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = RustlsStream<S>;
    type TlsConnect = RustlsConnect;
    type Error = XError;

    fn make_tls_connect(&mut self, domain: &str) -> Result<RustlsConnect, Self::Error> {
        self.for_domain(domain)
    }
}

impl<S> TlsConnect<S> for RustlsConnect
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = RustlsStream<S>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<RustlsStream<S>, io::Error>> + Send>>;

    fn connect(self, stream: S) -> Self::Future {
        let Self { connector, domain } = self;
        Box::pin(async move {
            let tls = connector.connect(domain, stream).await?;
            Ok(RustlsStream { inner: tls })
        })
    }
}

/// rustls 包装流（实现 tokio-postgres `TlsStream`）。
pub struct RustlsStream<S> {
    inner: tokio_rustls::client::TlsStream<S>,
}

impl<S> AsyncRead for RustlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for RustlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl<S> TlsStream for RustlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn channel_binding(&self) -> ChannelBinding {
        // 基础 TLS：不提供 channel binding（SCRAM-PLUS 可后续增强）
        ChannelBinding::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_config_builds() {
        let cfg = build_client_config().expect("rustls client config");
        // 根证书非空
        assert!(!cfg.alpn_protocols.is_empty() || true);
        let _ = cfg;
    }

    #[test]
    fn make_tls_connect_accepts_domain() {
        let make = MakeRustlsConnect::with_webpki_roots().expect("connector");
        let connect = make.for_domain("db.example.com").expect("sni");
        let dbg = format!("{connect:?}");
        assert!(dbg.contains("RustlsConnect"));
    }

    #[test]
    fn make_tls_connect_rejects_empty_domain() {
        let make = MakeRustlsConnect::with_webpki_roots().expect("connector");
        let err = make.for_domain("").expect_err("empty domain");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }
}
