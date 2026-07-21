//! TungsteniteWsConnector / WsConnection 本地 loopback 测试。

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use transportx::{__map_tungstenite_error, TransportError, TungsteniteWsConnector, WsConnector};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ws_text_binary_ping_pong_close_lifecycle() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ready_tx.send(());
        ws.send(Message::Ping(Bytes::from_static(b"p"))).await.unwrap();
        ws.send(Message::Text("hello-text".into())).await.unwrap();
        ws.send(Message::Pong(Bytes::from_static(b"p"))).await.unwrap();
        ws.send(Message::Binary(Bytes::from_static(b"bin-payload"))).await.unwrap();
        loop {
            match ws.next().await {
                Some(Ok(Message::Binary(b))) => {
                    assert_eq!(b.as_ref(), b"from-client");
                    break;
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => continue,
                Some(Err(_)) => break,
            }
        }
        ws.send(Message::Close(None)).await.unwrap();
    });

    let connector = TungsteniteWsConnector::new();
    let mut conn = connector.connect(&format!("ws://{addr}/")).await.expect("connect");
    let _ = ready_rx.await;

    let frame = conn.next_frame().await.unwrap().expect("text frame");
    assert_eq!(frame.as_ref(), b"hello-text");

    let frame = conn.next_frame().await.unwrap().expect("binary frame");
    assert_eq!(frame.as_ref(), b"bin-payload");

    conn.send_frame(Bytes::from_static(b"from-client")).await.unwrap();

    let end = conn.next_frame().await.unwrap();
    assert!(end.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ws_close_then_send_maps_connection_closed() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        while let Some(msg) = ws.next().await {
            if let Ok(Message::Close(_)) = msg {
                break;
            }
        }
    });
    let connector = TungsteniteWsConnector::new();
    let mut conn = connector.connect(&format!("ws://{addr}/")).await.unwrap();
    conn.close().await.unwrap();
    let err = conn.send_frame(Bytes::from_static(b"x")).await.unwrap_err();
    assert!(
        matches!(
            err,
            TransportError::ConnectionClosed { .. } | TransportError::ProtocolViolation(_)
        ),
        "got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ws_abrupt_tcp_drop_maps_unclean_or_io() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let _ws = accept_async(stream).await.unwrap();
    });
    let connector = TungsteniteWsConnector::new();
    let mut conn = connector.connect(&format!("ws://{addr}/")).await.unwrap();
    let err = conn.next_frame().await.unwrap_err();
    assert!(
        matches!(
            err,
            TransportError::ConnectionClosed { .. }
                | TransportError::Io(_)
                | TransportError::ProtocolViolation(_)
        ),
        "got {err:?}"
    );
}

#[tokio::test]
async fn ws_connect_invalid_url_protocol_violation() {
    let connector = TungsteniteWsConnector;
    let err = match connector.connect("not-a-url").await {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert!(matches!(err, TransportError::ProtocolViolation(_)), "got {err:?}");
}

#[tokio::test]
async fn ws_connect_refused_maps_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let connector = TungsteniteWsConnector::new();
    let err = match connector.connect(&format!("ws://{addr}/")).await {
        Ok(_) => panic!("expected error"),
        Err(e) => e,
    };
    assert!(
        matches!(err, TransportError::Io(_) | TransportError::ProtocolViolation(_)),
        "got {err:?}"
    );
}

#[test]
fn map_tungstenite_error_variants() {
    use tokio_tungstenite::tungstenite::error::{Error, ProtocolError, UrlError};
    assert!(matches!(
        __map_tungstenite_error(Error::ConnectionClosed),
        TransportError::ConnectionClosed { clean: true }
    ));
    assert!(matches!(
        __map_tungstenite_error(Error::AlreadyClosed),
        TransportError::ConnectionClosed { clean: true }
    ));
    assert!(matches!(
        __map_tungstenite_error(Error::Io(std::io::Error::other("e"))),
        TransportError::Io(_)
    ));
    assert!(matches!(
        __map_tungstenite_error(Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)),
        TransportError::ProtocolViolation(_)
    ));
    assert!(matches!(
        __map_tungstenite_error(Error::Url(UrlError::UnableToConnect("x".into()))),
        TransportError::ProtocolViolation(_)
    ));
    assert!(matches!(__map_tungstenite_error(Error::Utf8), TransportError::ProtocolViolation(_)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ws_server_sends_only_ping_then_text_skips_control() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ready_tx.send(());
        // 控制帧后跟应用文本；客户端 next_frame 应跳过 Ping
        let _ = ws.send(Message::Ping(Bytes::from_static(b"x"))).await;
        let _ = ws.send(Message::Ping(Bytes::from_static(b"y"))).await;
        let _ = ws.send(Message::Text("after-pings".into())).await;
        while let Some(msg) = ws.next().await {
            if matches!(msg, Ok(Message::Close(_)) | Err(_)) {
                break;
            }
        }
    });
    let mut conn = TungsteniteWsConnector::new().connect(&format!("ws://{addr}/")).await.unwrap();
    let _ = ready_rx.await;
    let frame = conn.next_frame().await.expect("next_frame ok").expect("application frame");
    assert_eq!(frame.as_ref(), b"after-pings");
    let _ = conn.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ws_stream_end_without_close_frame() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let ws = accept_async(stream).await.unwrap();
        drop(ws);
    });
    let mut conn = TungsteniteWsConnector::new().connect(&format!("ws://{addr}/")).await.unwrap();
    let result = conn.next_frame().await;
    assert!(result.is_err() || matches!(result, Ok(None)), "{result:?}");
}

#[test]
fn connector_debug_default() {
    let c = TungsteniteWsConnector;
    let _ = format!("{c:?}");
    let _ = TungsteniteWsConnector::new();
}
