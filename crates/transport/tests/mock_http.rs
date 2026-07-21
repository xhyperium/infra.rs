//! MockHttpTransport + legacy HttpTransport + HttpDriver 行为测试。

#![allow(deprecated)]

use bytes::Bytes;
use kernel::ErrorKind;
use std::sync::Arc;
use std::time::Duration;
use transportx::{
    HttpDriver, HttpRequest, HttpResponse, HttpTransport, MockHttpTransport, TransportError,
};

#[tokio::test]
async fn get_returns_preset_response() {
    let t = MockHttpTransport::new();
    t.set_get("https://api/ping", Bytes::from_static(b"{}"));
    let r = t.get("https://api/ping").await.unwrap();
    assert_eq!(r, Bytes::from_static(b"{}"));
}

#[tokio::test]
async fn get_missing_url_returns_not_found() {
    let t = MockHttpTransport::new();
    let err = t.get("https://api/nope").await.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Missing);
    assert!(err.to_string().contains("mock GET"));
}

#[tokio::test]
async fn post_returns_preset_response_ignoring_body() {
    let t = MockHttpTransport::new();
    t.set_post("https://api/order", Bytes::from_static(b"ack"));
    let r = t.post("https://api/order", Bytes::from_static(b"body-ignored")).await.unwrap();
    assert_eq!(r, Bytes::from_static(b"ack"));
}

#[tokio::test]
async fn post_missing_url_returns_not_found() {
    let t = MockHttpTransport::new();
    let err = t.post("https://api/nope", Bytes::from_static(b"")).await.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Missing);
    assert!(err.to_string().contains("mock POST"));
}

#[tokio::test]
async fn set_overwrites_previous_response() {
    let t = MockHttpTransport::new();
    t.set_get("u", Bytes::from_static(b"a"));
    t.set_get("u", Bytes::from_static(b"b"));
    assert_eq!(t.get("u").await.unwrap(), Bytes::from_static(b"b"));
}

#[tokio::test]
async fn set_post_overwrites() {
    let t = MockHttpTransport::new();
    t.set_post("p", Bytes::from_static(b"1"));
    t.set_post("p", Bytes::from_static(b"2"));
    assert_eq!(t.post("p", Bytes::new()).await.unwrap().as_ref(), b"2");
}

#[tokio::test]
async fn get_and_post_are_isolated() {
    let t = MockHttpTransport::new();
    t.set_get("u", Bytes::from_static(b"g"));
    t.set_post("u", Bytes::from_static(b"p"));
    assert_eq!(t.get("u").await.unwrap(), Bytes::from_static(b"g"));
    assert_eq!(t.post("u", Bytes::new()).await.unwrap(), Bytes::from_static(b"p"));
}

#[tokio::test]
async fn as_trait_object() {
    let t = MockHttpTransport::new();
    t.set_get("u", Bytes::from_static(b"v"));
    let dyn_t: &dyn HttpTransport = &t;
    assert_eq!(dyn_t.get("u").await.unwrap(), Bytes::from_static(b"v"));
}

#[test]
fn transport_error_keeps_reconnect_semantics() {
    let err = TransportError::ConnectionClosed { clean: false };
    assert_eq!(err.to_string(), "connection closed (false)");
    let err = TransportError::RateLimited { retry_after: Some(Duration::from_secs(2)) };
    assert!(err.to_string().contains("2s"));
    assert_eq!(TransportError::ConnectTimeout.to_string(), "connect timeout");
    assert_eq!(TransportError::ReadTimeout.to_string(), "read timeout");
    assert!(TransportError::ProtocolViolation("bad".into()).to_string().contains("bad"));
    assert!(TransportError::Io(Box::new(std::io::Error::other("x"))).to_string().contains("x"));
}

#[test]
fn request_headers_are_part_of_the_boundary() {
    let request = HttpRequest {
        method: "POST".into(),
        url: "https://api/order".into(),
        headers: vec![("X-Token".into(), "redacted".into())],
        body: Some(Bytes::from_static(b"{}")),
    };
    assert_eq!(request.headers[0].0, "X-Token");
    assert_eq!(request.body.as_ref().unwrap().as_ref(), b"{}");
    let response = HttpResponse { status: 200, body: Bytes::from_static(b"x") };
    assert_eq!(response, response.clone());
    let _ = format!("{request:?}{response:?}");
}

#[tokio::test]
async fn mock_http_driver_returns_transport_response() {
    let driver = MockHttpTransport::new();
    driver.set_get("https://api/ping", Bytes::from_static(b"{}"));
    let response = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: "https://api/ping".into(),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, Bytes::from_static(b"{}"));
}

#[tokio::test]
async fn mock_http_driver_post_and_case_insensitive_method() {
    let driver = MockHttpTransport::new();
    driver.set_post("https://api/order", Bytes::from_static(b"ok"));
    let response = driver
        .execute(HttpRequest {
            method: "post".into(),
            url: "https://api/order".into(),
            headers: Vec::new(),
            body: Some(Bytes::from_static(b"ignored")),
        })
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, Bytes::from_static(b"ok"));
}

#[tokio::test]
async fn mock_http_driver_unsupported_method() {
    let driver = MockHttpTransport::new();
    let err = driver
        .execute(HttpRequest {
            method: "PUT".into(),
            url: "https://api/x".into(),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    match err {
        TransportError::ProtocolViolation(msg) => assert!(msg.contains("unsupported")),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn mock_http_driver_missing_response() {
    let driver = MockHttpTransport::new();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: "https://api/missing".into(),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    match err {
        TransportError::ProtocolViolation(msg) => assert!(msg.contains("missing")),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn mock_http_driver_poisoned_gets_lock() {
    let driver = MockHttpTransport::new();
    driver.__poison_gets();
    let err = driver
        .execute(HttpRequest {
            method: "GET".into(),
            url: "u".into(),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, TransportError::Io(_)));
}

#[tokio::test]
async fn mock_http_driver_poisoned_posts_lock() {
    let driver = MockHttpTransport::new();
    driver.__poison_posts();
    let err = driver
        .execute(HttpRequest {
            method: "POST".into(),
            url: "u".into(),
            headers: Vec::new(),
            body: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, TransportError::Io(_)));
}

#[tokio::test]
async fn mock_as_arc_dyn_http_driver() {
    let mock = Arc::new(MockHttpTransport::new());
    mock.set_get("u", Bytes::from_static(b"v"));
    let driver: Arc<dyn HttpDriver> = mock;
    let r = driver
        .execute(HttpRequest { method: "GET".into(), url: "u".into(), headers: vec![], body: None })
        .await
        .unwrap();
    assert_eq!(r.body.as_ref(), b"v");
}

#[test]
fn mock_debug() {
    let m = MockHttpTransport::default();
    let _ = format!("{m:?}");
}
