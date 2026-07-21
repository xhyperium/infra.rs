//! ReqwestHttpDriver + map_reqwest_error 本地 loopback 测试。

use bytes::Bytes;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use transportx::{
    __map_client_build_error, __map_reqwest_error, HttpDriver, HttpRequest, ReqwestHttpDriver,
    TransportError,
};

/// 极简阻塞 HTTP/1.1 响应服务端（单连接）。
fn spawn_http_server(
    status_line: &'static str,
    headers: &'static [(&'static str, &'static str)],
    body: &'static [u8],
) -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let mut response = format!("{status_line}\r\n");
        for (k, v) in headers {
            response.push_str(&format!("{k}: {v}\r\n"));
        }
        response.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
    });
    addr
}

#[test]
fn reqwest_driver_new_and_with_timeout() {
    let d = ReqwestHttpDriver::new().unwrap();
    let _ = format!("{d:?}");
    let _ = ReqwestHttpDriver::with_timeout(Some(Duration::from_secs(2))).unwrap();
    let _ = ReqwestHttpDriver::with_timeout(None).unwrap();
}

#[test]
fn reqwest_driver_rejects_invalid_method_without_network() {
    let driver = ReqwestHttpDriver::new().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let error = runtime
        .block_on(driver.execute(HttpRequest {
            method: "not a method".into(),
            url: "https://api.invalid".into(),
            headers: Vec::new(),
            body: None,
        }))
        .unwrap_err();
    assert!(matches!(error, TransportError::ProtocolViolation(_)));
}

#[tokio::test]
async fn reqwest_driver_ok_2xx_with_headers_and_body() {
    let addr =
        spawn_http_server("HTTP/1.1 200 OK", &[("Content-Type", "text/plain")], b"hello-body");
    let driver = ReqwestHttpDriver::with_timeout(Some(Duration::from_secs(5))).unwrap();
    let response = driver
        .execute(HttpRequest {
            method: "POST".into(),
            url: format!("http://{addr}/echo"),
            headers: vec![("X-Test".into(), "1".into())],
            body: Some(Bytes::from_static(b"payload")),
        })
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body.as_ref(), b"hello-body");
}

#[tokio::test]
async fn reqwest_driver_4xx_returns_ok_response() {
    let addr = spawn_http_server("HTTP/1.1 404 Not Found", &[], b"missing");
    let driver = ReqwestHttpDriver::new().unwrap();
    let response = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/nope"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap();
    assert_eq!(response.status, 404);
    assert_eq!(response.body.as_ref(), b"missing");
}

#[tokio::test]
async fn reqwest_driver_5xx_returns_ok_response() {
    let addr = spawn_http_server("HTTP/1.1 503 Service Unavailable", &[], b"down");
    let driver = ReqwestHttpDriver::new().unwrap();
    let response = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/down"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap();
    assert_eq!(response.status, 503);
}

#[tokio::test]
async fn reqwest_driver_429_with_retry_after_integer_seconds() {
    let addr = spawn_http_server("HTTP/1.1 429 Too Many Requests", &[("Retry-After", "7")], b"");
    let driver = ReqwestHttpDriver::new().unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/rl"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    match err {
        TransportError::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(Duration::from_secs(7)));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn reqwest_driver_429_without_or_invalid_retry_after() {
    let addr =
        spawn_http_server("HTTP/1.1 429 Too Many Requests", &[("Retry-After", "not-an-int")], b"");
    let driver = ReqwestHttpDriver::new().unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/rl2"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    match err {
        TransportError::RateLimited { retry_after } => assert_eq!(retry_after, None),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn reqwest_driver_429_no_retry_after_header() {
    let addr = spawn_http_server("HTTP/1.1 429 Too Many Requests", &[], b"");
    let driver = ReqwestHttpDriver::new().unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/rl3"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, TransportError::RateLimited { retry_after: None }));
}

#[tokio::test]
async fn reqwest_driver_connection_refused_maps_to_io() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let driver = ReqwestHttpDriver::with_timeout(Some(Duration::from_secs(2))).unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, TransportError::Io(_)), "got {err:?}");
}

#[tokio::test]
async fn reqwest_driver_read_timeout() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        thread::sleep(Duration::from_secs(5));
    });
    let driver = ReqwestHttpDriver::with_timeout(Some(Duration::from_millis(200))).unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/slow"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .expect_err("stalling server must not succeed");
    // 总超时在 connect 之后通常为 ReadTimeout；部分平台归为 Io/ConnectTimeout
    assert!(
        matches!(
            err,
            TransportError::ReadTimeout | TransportError::ConnectTimeout | TransportError::Io(_)
        ),
        "expected timeout/io TransportError, got {err:?}"
    );
}

#[tokio::test]
async fn map_reqwest_connect_timeout_branch() {
    // 专用 connect_timeout，不可达 TEST-NET 地址 → is_timeout && is_connect
    let client =
        reqwest::Client::builder().connect_timeout(Duration::from_millis(150)).build().unwrap();
    let err = client.get("http://192.0.2.1:9/").send().await.expect_err("should timeout");
    let mapped = __map_reqwest_error(err);
    // 部分环境可能直接 Io；优先验证 ConnectTimeout 语义
    match mapped {
        TransportError::ConnectTimeout => {}
        TransportError::ReadTimeout | TransportError::Io(_) => {
            // 若平台未标 is_connect，再造一次短总超时读路径验证 ReadTimeout
            let client =
                reqwest::Client::builder().timeout(Duration::from_millis(80)).build().unwrap();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            thread::spawn(move || {
                let (mut s, _) = listener.accept().unwrap();
                let mut buf = [0u8; 512];
                let _ = s.read(&mut buf);
                thread::sleep(Duration::from_secs(3));
            });
            let err2 =
                client.get(format!("http://{addr}/")).send().await.expect_err("read timeout");
            let m2 = __map_reqwest_error(err2);
            assert!(
                matches!(
                    m2,
                    TransportError::ReadTimeout
                        | TransportError::ConnectTimeout
                        | TransportError::Io(_)
                ),
                "m2={m2:?}"
            );
        }
        other => panic!("unexpected map: {other:?}"),
    }
}

#[test]
fn map_reqwest_error_builder_branch() {
    let client = reqwest::Client::new();
    let err = client
        .request(reqwest::Method::GET, "http://example.com")
        .header("X-Bad", "\n")
        .build()
        .unwrap_err();
    let mapped = __map_reqwest_error(err);
    assert!(
        matches!(mapped, TransportError::ProtocolViolation(_) | TransportError::Io(_)),
        "mapped={mapped:?}"
    );
}

#[test]
fn map_client_build_error_wraps_as_io() {
    // 任意 reqwest::Error 均可验证 build 失败映射为 Io
    let err = reqwest::Client::new()
        .request(reqwest::Method::GET, "http://example.com")
        .header("X-Bad", "\n")
        .build()
        .unwrap_err();
    let mapped = __map_client_build_error(err);
    assert!(matches!(mapped, TransportError::Io(_)), "mapped={mapped:?}");
}

#[tokio::test]
async fn reqwest_partial_response_maps_error() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 2048];
        let _ = stream.read(&mut buf).await;
        let headers = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial";
        let _ = stream.write_all(headers).await;
    });
    let driver = ReqwestHttpDriver::with_timeout(Some(Duration::from_secs(2))).unwrap();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: format!("http://{addr}/partial"),
            headers: Vec::new(),
            body: None,
        })
        .await
        .expect_err("truncated body must not yield Ok(HttpResponse)");
    // body 读取失败映射为 Io 或超时类错误，绝不能是成功响应
    assert!(
        matches!(
            err,
            TransportError::Io(_) | TransportError::ReadTimeout | TransportError::ConnectTimeout
        ),
        "expected body-read TransportError, got {err:?}"
    );
}
