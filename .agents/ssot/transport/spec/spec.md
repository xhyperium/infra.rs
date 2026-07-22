# transportx L1 规范

状态：当前 `0.1.2` active 实现合同（HTTP/WS 真实驱动 + Mock 已落地；**未达**生产 M3）
权威来源：`CONSTITUTION.md`、`docs/architecture/spec.md`、ADR-007、当前 crate 源码
crate 路径：`crates/transport`

- Package / lib：`transportx` / `transportx`（别名 `xhyper-transportx` 仅作废弃兼容标签 / dual-mirror 文件名）
- Implementation snapshot：`b0934baa`（2026-07-15）
- Document commit：`e0b98df4`
- Verified at：`e0b98df4`（相关实现路径未变化）
- Candidate：[SPEC-INFRA-TRANSPORTX-002](../../../draft/transportx-complete-spec.md)（Draft，非权威，不覆盖本文）

> **证据优先**：本文描述当前代码事实，不是生产就绪证明。TLS 策略、连接池、认证、gRPC、生产级故障矩阵仍为 Unknown / 未闭环。

## 1. 目的与证据等级

`transportx` 提供可供存储与交易所适配器复用的 HTTP/WebSocket 传输基础设施。

- **证据（Evidence）**：XLib spec、Approved ADR、当前 `Cargo.toml` / `src/lib.rs`。
- **推论（Inference）**：满足证据所需的最小实现后果。
- **未知（Unknown）**：权威材料尚未裁定，实施不得静默选择。

权威顺序：`CONSTITUTION.md` → `docs/architecture/spec.md` → Approved ADR → 当前代码。

## 2. 职责与非目标

### 2.1 职责

1. 统一 HTTP / WebSocket 客户端侧传输边界（spec §4.4）。
2. 位于 L1，可被存储与交易所适配器依赖（ADR-007；R2/R2.1）。
3. 只承载传输实现，不承载业务契约；业务 trait/type 属于 `contracts` / `canonical`。
4. 将驱动私有类型（`reqwest::Client`、tungstenite stream）封装在 crate 内部。

### 2.2 非目标

- 不定义存储、交易所或领域业务语义。
- 不实现重试、熔断、限流或调度（`resiliencx` / `schedulex`）。
- 不成为组合根（`bootstrap`）。
- 不承诺完整服务发现、负载均衡、生产 TLS/认证矩阵、gRPC 服务端生命周期（均为 Unknown / 未闭环）。

## 3. 当前代码与依赖契约

| 项 | 事实 |
|----|------|
| 版本 | `0.1.2` |
| 依赖 | `kernel`、`async-trait`、`bytes`、`thiserror`、`reqwest`、`tokio`（`net`）、`tokio-tungstenite`、`futures-util` |
| R3 | **禁止**依赖其他 L1（configx/observex/resiliencx/schedulex/bootstrap） |
| 公开错误 | `TransportError`（timeout / closed / rate-limited / protocol / I/O） |
| HTTP 边界 | `HttpRequest` / `HttpResponse` / `HttpDriver` |
| WS 边界 | `WsConnector` / `WsConnection` |
| 真实驱动 | `ReqwestHttpDriver`、`TungsteniteWsConnector` |
| Mock | `MockHttpTransport`（并实现 `HttpDriver`） |
| 遗留 | `HttpTransport` trait 已 `#[deprecated(note = "use HttpDriver")]` |

binance/okx 的 REST 路径消费 `HttpDriver`，WS 路径消费 `WsConnector`；默认构造使用真实 reqwest/tungstenite，测试使用 `MockHttpTransport`。

第三方网络依赖须通过 `cargo-deny`；版本走 workspace dependencies。

## 4. 公开 API 状态

### 4.1 已实现（代码事实）

- `HttpDriver::execute`
- `WsConnector::connect` + `WsConnection::{next_frame,send_frame,close}`
- 默认驱动构造：`ReqwestHttpDriver::new` / `with_timeout`；`TungsteniteWsConnector`
- Mock 预置响应：`MockHttpTransport::{set_get,set_post}`

### 4.2 提案而非合同

implementation-plan 中的 `Codec` / `RpcClient` / `RpcServer` **仍未批准**，不得据此固化跨层 SPI。若形成稳定多实现方契约，优先走 `contracts` additive 提案。

### 4.3 未闭环 / Unknown

协议 feature 切分、生产 TLS/mTLS、代理、连接池、压缩、背压、统一取消/超时预算、gRPC 面、完整错误到 `XError` 映射策略。

### 4.4 当前 HTTP/WS 特例

- HTTP 429 当前读取整数秒 `Retry-After` 后返回 `TransportError::RateLimited`；其他 4xx/5xx 返回 `Ok(HttpResponse)`。
- `HttpResponse` 当前只保留 status/body，不保留 response headers。
- WS text/binary 转为 `Bytes`；Ping/Pong/Frame 被跳过，Close 返回 `None` 且不保留 code/reason。
- 当前没有 request/response/frame size limit、absolute deadline 或 cancellation API。

## 5. 行为、不变量与错误

1. **分层**：L1 实现，不迁入业务合同（ADR-007）。
2. **复用方向**：适配器可依赖 transportx；transportx 不反向依赖适配器。
3. **同层隔离**：R3/R3.1；不直接依赖其他 L1。
4. **错误边界**：可恢复网络失败不 panic；驱动错误映射到 `TransportError`。
5. **资源边界**：失败/取消后不得遗留不可回收连接/任务（生产矩阵仍待 M3 Evidence）。
6. **成熟度**：实现级可用 ≠ M2/M3 真实集成/故障恢复证据。

## 6. 测试与验收

- 当前 11 个单元测试覆盖 mock 与有限驱动映射（见 `src/lib.rs` `#[cfg(test)]`）。
- **不得**将本 crate 描述为 mock-only 或“不引入 reqwest/tungstenite”。
- **不得**将本地成功构建宣称为生产 TLS/认证就绪。

```bash
cargo test -p transportx（别名 xhyper-transportx 已废弃，不可用于 -p）
cargo check -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
cargo test -p binancex（别名 xhyper-binance 已废弃）
cargo test -p okxx（别名 xhyper-okx 已废弃）
cargo xtl lint-deps
cargo fmt -- --check
```

## 7. 变更日志（文档）

| 日期 | 变更 |
|------|------|
| 2026-07-14 | 由“骨架/空依赖”改为与源码一致的实现合同；明确真实驱动与未达 M3 |
