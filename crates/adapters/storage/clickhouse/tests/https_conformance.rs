//! ClickHouse HTTPS transport 的可复现 TLS 实验。
//!
//! 本测试使用最小 TLS HTTP 协议服务返回 ClickHouse `SELECT 1` 响应，验证真实 TLS
//! 握手、hostname/CA 校验和错误 CA fail-closed；不外推为 ClickHouse 集群证据。

use std::sync::Arc;
use std::time::Duration;

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("缺少测试环境变量 {name}"))
}

fn load_server_config() -> rustls::ServerConfig {
    let cert_path = required_env("INFRA_CLICKHOUSE_TLS_CERT_FILE");
    let key_path = required_env("INFRA_CLICKHOUSE_TLS_KEY_FILE");
    let cert_file = std::fs::File::open(cert_path).expect("读取服务端证书");
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certificates = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .expect("解析服务端证书");
    let key_file = std::fs::File::open(key_path).expect("读取服务端私钥");
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .expect("解析服务端私钥")
        .expect("服务端私钥存在");
    let _ = rustls::crypto::ring::default_provider().install_default();
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .expect("构造 TLS server config")
}

async fn serve_three_connections(listener: TcpListener, acceptor: TlsAcceptor) {
    for _ in 0..3 {
        let (stream, _) = listener.accept().await.expect("接受 TLS TCP");
        let Ok(mut stream) = acceptor.accept(stream).await else {
            // 错误 CA 的客户端会在握手阶段主动终止；这是预期负向路径。
            continue;
        };
        let mut request = vec![0u8; 16 * 1024];
        let _ = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut request))
            .await
            .expect("读取 HTTPS 请求不得超时")
            .expect("读取 HTTPS 请求");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\n1\n")
            .await
            .expect("写 HTTPS 响应");
        stream.shutdown().await.expect("关闭 HTTPS 流");
    }
}

#[tokio::test]
#[ignore = "需要脚本生成临时 CA/证书"]
async fn trusted_ca_succeeds_and_wrong_ca_fails_closed() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定本机 TLS 端口");
    let port = listener.local_addr().expect("TLS 地址").port();
    let acceptor = TlsAcceptor::from(Arc::new(load_server_config()));
    let server = tokio::spawn(serve_three_connections(listener, acceptor));

    let good = ClickHouseConfig {
        host: "localhost".into(),
        http_port: port,
        tls: true,
        tls_ca_file: Some(required_env("INFRA_CLICKHOUSE_TLS_CA_FILE").into()),
        timeout: Duration::from_secs(5),
        ..ClickHouseConfig::default()
    };
    let pool = ClickHousePool::connect(good).await.expect("可信 CA HTTPS 应成功");
    pool.ping().await.expect("HTTPS 复用路径应成功");

    let bad = ClickHouseConfig {
        host: "localhost".into(),
        http_port: port,
        tls: true,
        tls_ca_file: Some(required_env("INFRA_CLICKHOUSE_TLS_BAD_CA_FILE").into()),
        timeout: Duration::from_secs(5),
        ..ClickHouseConfig::default()
    };
    let error = match ClickHousePool::connect(bad).await {
        Err(error) => error,
        Ok(_) => panic!("错误 CA 必须拒绝"),
    };
    assert!(
        matches!(error.kind(), ErrorKind::Unavailable | ErrorKind::DeadlineExceeded),
        "kind={:?}",
        error.kind()
    );
    assert!(std::error::Error::source(&error).is_some());
    server.await.expect("TLS server task");
}
