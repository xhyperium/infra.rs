# SPEC-INFRA-TRANSPORTX-003

状态：IMPLEMENTED CANDIDATE；等待 PR CI、独立审查与人工审批。

| 字段 | 值 |
|---|---|
| Baseline | `2299ff1f9c6d006d014c80d89a3082a01ba27c9a` |
| Package / lib | `transportx` / `transportx` |
| Path / layer | `crates/transport` / L1 |
| 当前候选版本 | `0.1.4` |
| 消费者 | binance / okx 等 adapter |
| Authority | 本文件与双镜像共同定义当前声明面 |

## 1. 定位

`transportx` 统一客户端侧 HTTP/WebSocket 传输边界，封装 reqwest 与
tokio-tungstenite 私有类型。它不定义交易所、存储或领域业务语义，也不实现重试、
熔断、限流或调度策略。

本 crate 提供真实驱动、可注入 trait、测试 Mock、TLS/代理配置和进程内 HTTP 客户端池。
这些实现事实不等于生产 M3、完整 mTLS、服务发现、gRPC 或跨进程连接治理已经闭合。

## 2. 公开接口

- HTTP：`HttpRequest`、`HttpResponse`、`HttpDriver`、`ReqwestHttpDriver`。
- WebSocket：`WsConnector`、`WsConnection`、`TungsteniteWsConnector`。
- 配置：`TlsConfig` / `TlsMode`、`ProxyConfig`。
- 池：`PoolConfig`、`HttpClientPool<T>`、`HttpClientLease<'_, T>`。
- 测试：`MockHttpTransport`；遗留 `HttpTransport` 仅作 deprecated 兼容。

`HttpResponse` 当前只保存 status/body，不保存响应 headers。WS text/binary 转换为
`Bytes`；控制帧由驱动内部处理，Close 结束流但不冻结 code/reason 合同。

禁止用 `#[doc(hidden)] pub` 暴露 reqwest/tungstenite 错误、锁投毒或其它测试钩子；
`doc(hidden)` 不改变可见性，私有映射与故障注入必须留在 crate 内单元测试边界。

## 3. HTTP 合同

### 3.1 请求与响应上限

- 请求体在发起网络请求前校验；超限返回
  `PayloadTooLarge { kind: "request_body", ... }`。
- 有 `Content-Length` 时先做整数安全转换并 fail-closed 校验。
- chunked/未知长度响应按 chunk 累加，每次扩容前检查累计大小；第一次越界立即返回
  `PayloadTooLarge { kind: "response_body", ... }`，不得等待响应结束后再整体判定。
- 上限为零表示该方向不设字节上限；调用方必须显式承担资源风险。

### 3.2 429 与 `Retry-After`

HTTP 429 映射为 `TransportError::RateLimited`。解析同时支持 RFC 9110 的：

- delay-seconds；
- IMF-fixdate HTTP-date。

`parse_retry_after_at(value, now)` 显式接收当前时间以支持确定性测试；过去日期钳制为
`Duration::ZERO`，非法值返回 `None`。其他 4xx/5xx 仍返回 `Ok(HttpResponse)`，
由上层决定业务错误策略。

### 3.3 Debug 脱敏

- 敏感 header 值与 body 内容不得出现在 Debug；body 只显示长度。
- URL Debug 只允许保留 scheme、host 与显式 port；path、query 名和值、userinfo、
  fragment 必须整体隐藏，避免路径 token、对象标识与无值 query key 泄漏。
- URL 解析失败或为不能安全重写的不透明 URL 时输出固定
  `<invalid-url-redacted>`，不得回显原文。
- `ProxyConfig` 的 URL 使用同一脱敏规则，password 始终脱敏。

## 4. TLS 与代理合同

- `TlsConfig::default()` 等价于 `system_roots()`，且 `sni = true`。
- `CustomCa` 读取 PEM 并追加根证书；读取、解析或 client build 失败必须返回错误。
- `InsecureDevOnly` 只用于显式开发场景，不得成为默认值。
- 当前 reqwest 接线不能兑现 `sni = false`；该值必须在构造阶段 fail-closed，
  不得静默忽略。
- 代理 URL 无效时返回 `ProtocolViolation`；用户名和密码同时存在时接入 basic auth。

## 5. HTTP 客户端池合同

- `PoolConfig::validate` 要求 `max_pool_size > 0` 且
  `max_idle <= max_pool_size`；`new` 与 `try_new` 都必须先校验。
- `new` 保留既有返回 `Self` 的签名，配置无效时以包含校验原因的中文消息 fail-fast
  panic；需要可恢复错误的生产接线必须使用 `try_new`。
- `checkout_with` 达到上限时 fail-fast；`checkout_idle_timeout` 只等待已有 idle
  对象。两者返回裸对象且要求手动归还，仅为兼容入口；新代码使用 RAII lease。
- checkout 优先复用 idle，否则在总许可上限内调用 factory。
- factory 返回错误或 panic 展开、显式归还、lease Drop 和 `into_inner` 都必须准确释放一次许可。
- `HttpClientLease` Drop 自动归还对象；`into_inner` 取走对象且不加入 idle。
- 锁中毒时使用已持有状态恢复许可，不能令 size=1 的池永久耗尽。

## 6. WebSocket 合同

- 出站 frame 在发送前执行字节上限校验。
- 连接时把同一上限下推到 tungstenite decoder 的 frame/message 配置，确保入站聚合
  message 在交付给调用方前被拒绝。
- decoder 的 `MessageTooLong` 映射为
  `PayloadTooLarge { kind: "ws_message", ... }`。
- connect timeout、I/O、协议错误和 clean/unclean close 保持类型化映射。
- 本合同不承诺应用层背压、重连、订阅恢复或业务级消息大小策略。

## 7. 依赖与分层

第三方依赖必须由根 `[workspace.dependencies]` 集中声明。当前依赖包括
`async-trait`、`bytes`、`futures-util`、`httpdate`、`reqwest`、`thiserror`、
`tokio`、`tokio-tungstenite` 与 workspace 内 `kernel`。

transportx 不得反向依赖 adapter，也不得直接依赖 configx、observex、resiliencx、
schedulex 或 bootstrap。

## 8. 非目标 / NO-GO

- 完整 mTLS、证书轮换、服务发现、负载均衡与跨进程池治理；
- gRPC、统一 absolute deadline/cancellation budget；
- 自动重试、熔断、限流、业务认证或交易所签名；
- 仅凭本地单测宣称生产 live、长稳、M3 或 package stable。

## 9. 验收

公共测试必须覆盖 URL fail-closed 脱敏、RFC 9110 Retry-After、chunked 累计越界、
SNI 拒绝、池配置/RAII/许可恢复、入站 decoder 上限及既有 HTTP/WS 生命周期。

```bash
cmp .agents/ssot/transport/spec/spec.md \
  .agents/ssot/transport/spec/xhyper-transportx-complete-spec.md
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p transportx --no-deps
cargo fmt --all --check
node scripts/quality-gates/check-workspace-deps.mjs
```

本地通过不能替代 PR CI、独立 reviewer 与人工审批。
