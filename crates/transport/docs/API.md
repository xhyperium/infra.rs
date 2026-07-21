# transportx 公开 API

**角色**：HTTP/WS 传输

## 公开消费面

- `HttpDriver` / `HttpRequest` / `HttpResponse` / `ReqwestHttpDriver`
- `WsConnector` / `WsConnection` / `TungsteniteWsConnector`
- `MockHttpTransport`（测试）
- `TransportError`

## 最小用法

```rust
use bytes::Bytes;
use transportx::{HttpDriver, HttpRequest, MockHttpTransport};

# async fn demo() {
let mock = MockHttpTransport::new();
mock.set_get("https://ex/ping", Bytes::from_static(b"ok"));
let r = mock.execute(HttpRequest {
    method: "GET".into(),
    url: "https://ex/ping".into(),
    headers: vec![],
    body: None,
}).await.unwrap();
assert_eq!(r.status, 200);
# }
```
