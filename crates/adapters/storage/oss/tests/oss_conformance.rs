//! ossx 对抗性一致性测试：闭合负向验收缺口 + 边界回归防护。
//!
//! 覆盖范围（离线，loopback HTTP，不依赖真实 OSS）：
//! - 鉴权降级拒绝：真实 401/403 响应经完整重试链路后确认不重试（`status_error` ↔
//!   `is_oss_retryable` 的字符串耦合契约回归）。
//! - 边界回归：operation deadline 超时不残留请求；配置层无界资源被拒绝。

use std::time::Duration;

use bytes::Bytes;
use kernel::ErrorKind;
use ossx::{OssClient, OssConfig};
use resiliencx::RetryConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buffer = [0u8; 1024];
    let mut expected_len = None;
    loop {
        let read = stream.read(&mut buffer).await.expect("read request");
        assert!(read > 0, "request closed before complete");
        request.extend_from_slice(&buffer[..read]);
        if expected_len.is_none()
            && let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n")
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

/// 单次连接、多次重试预算的客户端：用于验证"是否真的只发一次请求"。
async fn retrying_client(max_attempts: u32) -> (TcpListener, OssClient) {
    let listener = TcpListener::bind("[::1]:0").await.expect("bind loopback");
    let port = listener.local_addr().expect("local addr").port();
    let config = OssConfig::builder()
        .endpoint(format!("http://localhost:{port}"))
        .bucket("bucket")
        .access_key_id("id")
        .access_key_secret("sec")
        .request_timeout(Duration::from_secs(5))
        .operation_deadline(Duration::from_secs(5))
        .build()
        .expect("config");
    let client =
        OssClient::connect_with_retry(config, RetryConfig::fixed(max_attempts, 0)).expect("client");
    (listener, client)
}

/// 401/403 必须映射为 `Unavailable` 且 `is_oss_retryable` 拒绝重试：
/// `status_error()` 的措辞与 `is_oss_retryable()` 的字符串匹配是跨模块隐性契约，
/// 之前只有直接构造 `XError` 的单测，未验证真实响应端到端链路。
#[tokio::test]
async fn forbidden_status_does_not_retry_on_get() {
    let (listener, client) = retrying_client(5).await;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let _ = read_http_request(&mut stream).await;
        write_response(&mut stream, "403 Forbidden", "", "AccessDenied").await;
        // 若客户端误判重试，第二次连接会在此处被 accept 到，从而暴露 bug。
        let second = tokio::time::timeout(Duration::from_millis(200), listener.accept()).await;
        assert!(second.is_err(), "403 必须只触发一次请求，不能重试");
    });

    let error = client.get_object("infra-draft/forbidden").await.expect_err("403 必须失败");
    assert_eq!(error.kind(), ErrorKind::Unavailable);
    assert!(error.context().to_ascii_lowercase().contains("forbidden"));
    server.await.expect("server task");
}

#[tokio::test]
async fn unauthorized_status_does_not_retry_on_put() {
    let (listener, client) = retrying_client(5).await;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let _ = read_http_request(&mut stream).await;
        write_response(&mut stream, "401 Unauthorized", "", "InvalidAccessKeyId").await;
        let second = tokio::time::timeout(Duration::from_millis(200), listener.accept()).await;
        assert!(second.is_err(), "401 必须只触发一次请求，不能重试");
    });

    let error = client
        .put_object("infra-draft/unauthorized", Bytes::from_static(b"x"))
        .await
        .expect_err("401 必须失败");
    assert_eq!(error.kind(), ErrorKind::Unavailable);
    assert!(error.context().to_ascii_lowercase().contains("auth"));
    server.await.expect("server task");
}

/// 直接验证 `is_oss_retryable` 对真实 `status_error` 产物（而非手工构造字符串）的判定，
/// 防止未来重构 `status_error` 文案时悄悄破坏这条跨模块契约却无测试报警。
#[tokio::test]
async fn is_oss_retryable_rejects_real_forbidden_status_error() {
    let (listener, client) = retrying_client(1).await;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let _ = read_http_request(&mut stream).await;
        write_response(&mut stream, "403 Forbidden", "", "").await;
    });

    let error = client.get_object("infra-draft/x").await.expect_err("403 必须失败");
    assert!(
        !ossx::is_oss_retryable(&error),
        "真实 403 status_error 产物必须被 is_oss_retryable 拒绝"
    );
    server.await.expect("server task");
}

/// 服务端 5xx 仍必须走可重试路径（与鉴权拒绝形成对照，避免"一刀切禁止重试"的回归）。
#[tokio::test]
async fn server_error_still_retries_until_success() {
    let (listener, client) = retrying_client(3).await;
    let server = tokio::spawn(async move {
        let (mut first, _) = listener.accept().await.expect("accept first");
        let _ = read_http_request(&mut first).await;
        write_response(&mut first, "500 Internal Server Error", "", "boom").await;

        let (mut second, _) = listener.accept().await.expect("accept second");
        let _ = read_http_request(&mut second).await;
        write_response(&mut second, "200 OK", "", "ossx-conformance-payload").await;
    });

    let got = client.get_object("infra-draft/retry-ok").await.expect("must succeed after retry");
    assert_eq!(got, Bytes::from_static(b"ossx-conformance-payload"));
    server.await.expect("server task");
}

/// operation deadline 到期必须立即终止整个重试过程，不残留悬挂请求（边界回归）。
#[tokio::test]
async fn operation_deadline_bounds_whole_retry_and_leaves_no_pending_request() {
    let listener = TcpListener::bind("[::1]:0").await.expect("bind loopback");
    let port = listener.local_addr().expect("local addr").port();
    let config = OssConfig::builder()
        .endpoint(format!("http://localhost:{port}"))
        .bucket("bucket")
        .access_key_id("id")
        .access_key_secret("sec")
        .request_timeout(Duration::from_millis(80))
        .operation_deadline(Duration::from_millis(100))
        .build()
        .expect("config");
    let client = OssClient::connect_with_retry(config, RetryConfig::fixed(5, 0)).expect("client");

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let _ = read_http_request(&mut stream).await;
        // 故意不回应，制造"服务端挂起"场景，模拟慢下游。
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = stream.shutdown().await;
    });

    let started = std::time::Instant::now();
    let error = client.get_object("infra-draft/hang").await.expect_err("deadline 必须终止挂起请求");
    assert_eq!(error.kind(), ErrorKind::DeadlineExceeded);
    assert!(
        started.elapsed() < Duration::from_millis(300),
        "deadline 必须终止整个重试过程，不能靠 request_timeout 逐次耗尽后仍继续重试放大"
    );
    server.abort();
}

/// 配置层拒绝超过 crate 导出 `HARD_MAX_*` 的资源上界（无界资源硬拒绝）。
#[test]
fn config_rejects_object_bytes_above_hard_max() {
    let error = OssConfig::builder()
        .endpoint("https://oss.example.com")
        .bucket("bucket")
        .access_key_id("id")
        .access_key_secret("sec")
        .max_object_bytes(ossx::HARD_MAX_OBJECT_BYTES + 1)
        .build()
        .expect_err("超过硬上界必须 fail-closed");
    assert_eq!(error.kind(), ErrorKind::Invalid);
}

/// 配置层拒绝零值资源上界（防止"配置为无界"这一类退化）。
#[test]
fn config_rejects_zero_max_in_flight() {
    let error = OssConfig::builder()
        .endpoint("https://oss.example.com")
        .bucket("bucket")
        .access_key_id("id")
        .access_key_secret("sec")
        .max_in_flight(0)
        .build()
        .expect_err("零值 in-flight 上界必须 fail-closed");
    assert_eq!(error.kind(), ErrorKind::Invalid);
}
