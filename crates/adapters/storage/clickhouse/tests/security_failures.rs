//! ClickHouse HTTP 失败路径安全合同。
//!
//! 测试只启动 loopback 临时 HTTP 服务，不访问真实 ClickHouse 或生产环境。

use std::time::Duration;

use clickhousex::{ClickHouseConfig, ClickHousePool};
use kernel::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn spawn_one_response(status: &str, body: String) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("绑定临时 HTTP 端口");
    let port = listener.local_addr().expect("读取临时 HTTP 地址").port();
    let status = status.to_owned();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("接受 HTTP 连接");
        tokio::time::timeout(Duration::from_secs(2), read_request(&mut stream))
            .await
            .expect("读取请求不得超时")
            .expect("读取 HTTP 请求");
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.expect("写 HTTP 响应");
        stream.shutdown().await.expect("关闭 HTTP 流");
    });
    (port, server)
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> std::io::Result<()> {
    let mut request = Vec::with_capacity(4096);
    loop {
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            return Ok(());
        }
        request.extend_from_slice(&chunk[..read]);
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let header_end = header_end + 4;
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("content-length:")
                    .or_else(|| line.strip_prefix("Content-Length:"))
            })
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        if request.len() >= header_end + content_length {
            return Ok(());
        }
    }
}

fn local_config(port: u16) -> ClickHouseConfig {
    ClickHouseConfig {
        host: "127.0.0.1".into(),
        http_port: port,
        timeout: Duration::from_secs(2),
        acquire_timeout: Duration::from_secs(2),
        ..ClickHouseConfig::default()
    }
}

fn assert_not_disclosed(error: &kernel::XError, secrets: &[&str]) {
    let display = error.to_string();
    let debug = format!("{error:?}");
    for secret in secrets {
        assert!(!display.contains(secret), "Display 泄漏敏感响应片段");
        assert!(!debug.contains(secret), "Debug 泄漏敏感响应片段");
    }
}

#[tokio::test]
async fn clickhouse_exception_omits_sql_and_payload_from_error() {
    let secret_sql = "SELECT secret_column FROM customer_private";
    let secret_payload = "payload-token-should-never-escape";
    let body = format!(
        "Code: 60. DB::Exception: Table missing; query={secret_sql}; payload={secret_payload}"
    );
    let (port, server) = spawn_one_response("400 Bad Request", body).await;

    let error = match ClickHousePool::connect(local_config(port)).await {
        Err(error) => error,
        Ok(_) => panic!("ClickHouse 异常必须失败"),
    };

    assert_eq!(error.kind(), ErrorKind::Missing, "context={}", error.context());
    assert!(error.context().contains("server_code=60"));
    assert_not_disclosed(&error, &[secret_sql, secret_payload]);
    server.await.expect("HTTP server task");
}

#[tokio::test]
async fn authentication_response_omits_server_body_from_error() {
    let secret = "credential-detail-must-not-escape";
    let (port, server) =
        spawn_one_response("401 Unauthorized", format!("认证失败：{secret}")).await;

    let error = match ClickHousePool::connect(local_config(port)).await {
        Err(error) => error,
        Ok(_) => panic!("认证失败必须拒绝"),
    };

    assert_eq!(error.kind(), ErrorKind::Unavailable);
    assert_not_disclosed(&error, &[secret]);
    server.await.expect("HTTP server task");
}

#[tokio::test]
async fn unexpected_ping_response_omits_response_body_from_error() {
    let secret = "unexpected-body-must-not-escape";
    let (port, server) = spawn_one_response("200 OK", secret.to_owned()).await;

    let error = match ClickHousePool::connect(local_config(port)).await {
        Err(error) => error,
        Ok(_) => panic!("异常 ping 响应必须失败"),
    };

    assert_eq!(error.kind(), ErrorKind::Unavailable);
    assert_not_disclosed(&error, &[secret]);
    server.await.expect("HTTP server task");
}
