# transportx 公开 API

**角色**：HTTP/WS 传输

## 公开消费面

- `HttpDriver` / `HttpRequest` / `HttpResponse` / `ReqwestHttpDriver`
- `WsConnector` / `WsConnection` / `TungsteniteWsConnector`
- `MockHttpTransport`（测试）
- `TransportError`
- `HttpClientPool` / `PoolConfig` / `HttpClientLease`（`try_new` + RAII checkout）
- `TlsConfig` / `TlsMode` / `ProxyConfig`
- `parse_retry_after_at`（确定性 RFC 9110 解析）

## 安全语义

- HTTP response 逐 chunk 累计并在越界时立即停止。
- WS 入站 frame/message 限额在 tungstenite 解码/聚合前配置。
- Request/Proxy Debug 隐藏 URL userinfo 与全部 query value；非法 URL 不回显。
- `TlsConfig.sni=false` 当前明确拒绝，不代表企业 TLS 已支持。

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
