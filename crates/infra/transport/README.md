# transportx

L1 统一网络客户端抽象（SSOT `infra/transport`，ADR-007）。提供 `HttpDriver` / `WsConnector` 边界、内存 mock，以及由本 crate 私有化的 reqwest / tokio-tungstenite 默认驱动。

## 主要内容

- `HttpDriver`：带 headers/body 的 typed HTTP 请求
- `WsConnector` / `WsConnection`：帧级 WebSocket 生命周期边界
- `ReqwestHttpDriver` / `TungsteniteWsConnector`：真实驱动（客户端类型 crate-private）
- `HttpClientPool` / `HttpClientLease`：有界对象池与 Drop 自动归还
- `MockHttpTransport`：`set_get` / `set_post` 预置响应，并实现 `HttpDriver`
- 遗留 `HttpTransport`：`#[deprecated(note = "use HttpDriver")]`

## 最小用法

```rust
use bytes::Bytes;
use transportx::{HttpDriver, HttpRequest, MockHttpTransport};

# async fn demo() {
let mock = MockHttpTransport::new();
mock.set_get("https://api/ping", Bytes::from_static(b"{}"));
let response = mock
    .execute(HttpRequest {
        method: "GET".into(),
        url: "https://api/ping".into(),
        headers: vec![],
        body: None,
    })
    .await
    .unwrap();
assert_eq!(response.status, 200);
# }
```

## 定位

L1 基础设施层。**R3 禁止依赖其他 L1 crate**，保持网络抽象独立。

## 非职责

- 不解析具体交易所业务协议（归 adapter）
- 不实现重试 / 熔断 / 调度（`resiliencx` / `schedulex`）
- 不成为 bootstrap 组合根
- 不承诺企业 PKI/mTLS、WS 企业 TLS、gRPC 或 M3 故障恢复矩阵

## 限制与安全

- 所有外部 I/O 须支持 timeout
- 禁止在日志中输出完整 Authorization 头或私钥材料
- 真实外网测试须 `#[ignore]` 或显式环境门控；本仓默认用 loopback

## 版本

`0.1.4`（见 `Cargo.toml`）。实现合同：`.agents/ssot/infra/transport/spec/spec.md`。

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| 宣称企业 PKI/WS TLS/平台矩阵完成 | M3 与企业 TLS 仍 NO-GO |
| 把 `sni=false` 当作已支持 | 当前明确 fail-closed；未接线 |
| 把 loopback 测试当公网/业务 live | 只证明受控传输实现面 |

示例：`cargo run -p transportx --example mock_ping`

## 默认超时与资源上限（infra-s9t.16）

| 项 | 默认 | 逃生口 |
|----|------|--------|
| HTTP 总超时 | 30s（`ReqwestHttpDriver::new`） | `with_timeout(None)` 显式关闭 |
| HTTP 请求/响应体 | 16 MiB | `with_limits(..., 0, 0)` |
| WS 连接超时 | 30s | `TungsteniteWsConnector::with_limits` |
| WS 入站 frame/message | 4 MiB，解码/聚合前下沉 | `max_frame_bytes = 0` |
| Debug | header/body 脱敏；URL 仅保留 scheme/host/port | — |

HTTP 响应按 chunk 累计，首次越界立即中止；`Retry-After` 支持 delay-seconds 与 HTTP-date。
Pool 新代码应使用可恢复的 `try_new` + `checkout_lease_with`；兼容构造 `new` 对无效
`PoolConfig` fail-fast panic，旧手动 checkout/return 仅为兼容保留。

超限 → `TransportError::PayloadTooLarge`（中文 Display）。
