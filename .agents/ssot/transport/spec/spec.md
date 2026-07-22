# transportx `0.1.3` maintenance 实现合同

| 字段 | 值 |
|---|---|
| Status | Active maintenance spec；限定既有 HTTP/WS 客户端面，**未达 M3 / 非 package stable** |
| Package / lib | `transportx` / `transportx` |
| Path | `crates/transport` |
| Baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47`（2026-07-23 审计） |
| Target | `0.1.3`（`0.1.2` PATCH +1） |
| Mirror | `spec/spec.md` 与 `spec/xhyper-transportx-complete-spec.md` 必须 byte-identical |

本文只批准已有 `HttpDriver`、`WsConnector`、`HttpClientPool` 及其配置面的安全加固，不扩展为企业网络平台。

## 1. 责任与边界

- 提供 HTTP/WS 客户端传输、默认 reqwest/tungstenite 驱动与 Mock。
- 位于 L1，不依赖其他 L1，不承载重试、熔断、业务合同或组合根职责。
- `HttpDriver`、`WsConnector`、`HttpClientPool` 是本轮冻结的公共 TDD seam；兼容接口保留。
- M3 故障恢复、企业 PKI/mTLS、WS 企业 TLS、完整代理/认证矩阵与业务 live 证据保持 **OPEN / NO-GO**。

## 2. HTTP 合同

1. 请求体在发网前按 `max_request_body_bytes` fail-closed。
2. 响应若 `Content-Length` 已知且超限，可在读取前拒绝；无论是否有长度，正文必须按 chunk 流式累计，并在累计首次越界时立即停止读取，禁止先聚合整包再检查。
3. 429 的 `Retry-After` 支持 RFC 9110 `delay-seconds` 与 HTTP-date。解析使用显式 `now` 的确定性公共 seam；过去日期钳制为零，非法值为 `None`。
4. 其他 4xx/5xx 仍返回 `Ok(HttpResponse)`；`HttpResponse` 仍仅含 status/body。

## 3. WebSocket 合同

- `TungsteniteWsConnector` 在 handshake 时把 `max_frame_size` 与 `max_message_size` 下沉到 tungstenite 配置，使限制在帧解码/消息聚合前生效。
- 出站与解码后的防御性检查保留；入站超限必须终止为 `PayloadTooLarge` 或协议错误，不交付应用 payload。
- 连接超时、Close/异常关闭的既有公共语义保持兼容。

## 4. 安全配置合同

- `HttpRequest` 与 `ProxyConfig` 的 `Debug` 必须隐藏 URL userinfo 与**全部 query value**；scheme/host/path 与 query key 可保留用于定位。禁止依赖敏感 key 黑名单。
- URL 无法可靠解析时采用 fail-closed 输出，不回显原始值。
- `TlsConfig.sni == false` 当前没有真实接线，构建驱动必须明确拒绝；不得静默忽略。`sni == true` 保持既有系统根、自定义 CA、仅开发 insecure 行为。

## 5. 有界对象池合同

- `PoolConfig` 要求 `max_pool_size > 0` 且 `max_idle <= max_pool_size`；新增可失败构造执行校验，旧 `new` 保持兼容。
- 新增 RAII lease：checkout 成功后 lease 持有对象，`Drop` 自动归还对象并释放许可；显式取走对象时必须仍释放许可。
- 旧 `checkout_with` / `return_client` 保留；其手动借还风险写入文档，不宣称 RAII 可修复旧调用方遗忘归还。

## 6. 验收与 NO-GO

必须覆盖 chunked/无长度超限、入站 WS 超限、URL 脱敏、SNI 拒绝、lease drop 回收、两种 Retry-After 格式及失败路径。通过本地 loopback 只证明受控实现面，不证明公网、企业 PKI 或完整业务 live。

```bash
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets --all-features -- -D warnings
cargo doc -p transportx --no-deps
cargo test -p binancex --all-targets
cargo test -p okxx --all-targets
```
