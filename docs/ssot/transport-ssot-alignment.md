# transportx SSOT 对齐矩阵

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| SSOT | `.agents/ssot/transport/spec/spec.md` |
| 本仓 crate | `crates/transport` → package `transportx` / lib `transportx` |
| 版本 | `0.1.0` |
| 覆盖率门禁 | `cargo llvm-cov -p transportx --fail-under-lines 100` |

> 镜像写 COMPLETE ≠ 本仓可宣称 ship。本表以 **members + 源码 + 本仓测试** 为准。

## §2 职责与非目标

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 2.1.1 | 统一 HTTP/WS 客户端侧传输边界 | **PASS** | `crates/transport/src/lib.rs`：`HttpDriver` / `WsConnector` |
| 2.1.2 | L1，可被适配器依赖 | **PASS** | `Cargo.toml` 仅依赖 `xhyper-kernel` + 网络信封 |
| 2.1.3 | 不承载业务契约 | **PASS** | 无业务 trait/type；仅传输 DTO |
| 2.1.4 | 驱动私有类型封装 | **PASS** | `reqwest::Client` / tungstenite stream 均为 private 字段 |
| 2.2.* | 非目标（重试/熔断/bootstrap/TLS 矩阵等） | **PASS** | 未实现；见 Non-goals |

## §3 依赖与公开表面

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 3.ver | 版本 `0.1.0` | **PASS** | `crates/transport/Cargo.toml` |
| 3.deps | kernel + async-trait + bytes + thiserror + reqwest + tokio(net) + tokio-tungstenite + futures-util | **PASS** | `Cargo.toml` + workspace deps |
| 3.R3 | 禁止其他 L1 | **PASS** | `cargo metadata` 生产图仅 `xhyper-kernel` |
| 3.err | `TransportError` 重连语义 | **PASS** | enum 变体 + `tests/mock_http.rs` |
| 3.http | `HttpRequest` / `HttpResponse` / `HttpDriver` | **PASS** | `src/lib.rs` |
| 3.ws | `WsConnector` / `WsConnection` | **PASS** | `src/lib.rs` |
| 3.drv | `ReqwestHttpDriver` / `TungsteniteWsConnector` | **PASS** | `src/lib.rs` + loopback 测试 |
| 3.mock | `MockHttpTransport` + `HttpDriver` | **PASS** | `tests/mock_http.rs` |
| 3.legacy | `HttpTransport` deprecated | **PASS** | `#[deprecated(note = "use HttpDriver")]` |
| 3.binance/okx | 适配器消费真实驱动 | **PASS（接线）** / 公共 time 解析 **PASS** / 业务协议 **DEFER** | `with_http` + mock；`server_time` JSON 已解析（#172）；下单/签名未做 |

## §4 公开 API 行为

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 4.1.execute | `HttpDriver::execute` | **PASS** | mock + reqwest tests |
| 4.1.ws | connect / next_frame / send_frame / close | **PASS** | `tests/websocket.rs` |
| 4.1.ctor | `ReqwestHttpDriver::new` / `with_timeout`；`TungsteniteWsConnector` | **PASS** | `tests/reqwest_driver.rs` |
| 4.1.mock | `set_get` / `set_post` | **PASS** | `tests/mock_http.rs` |
| 4.2 | Codec/RpcClient/RpcServer 非合同 | **PASS** | 未实现（正确） |
| 4.3 | M3 Unknown（TLS/池/代理/gRPC…） | **DEFER** | 生产矩阵；本目标 Non-goals |
| 4.4.429 | 整数秒 Retry-After → RateLimited | **PASS** | `reqwest_driver_429_*` |
| 4.4.4xx5xx | 其他 4xx/5xx → Ok(HttpResponse) | **PASS** | `reqwest_driver_4xx/5xx_*` |
| 4.4.headers | 不保留 response headers | **PASS** | `HttpResponse` 仅 status/body |
| 4.4.ws-frames | text/binary→Bytes；Ping/Pong 跳过；Close→None | **PASS** | `ws_text_binary_ping_pong_close_lifecycle` |
| 4.4.limits | payload 上限 / 敏感 Debug 等 | **PASS（P0 硬化）** | `PayloadTooLarge` + Debug 脱敏等（#166 / s9t.16）；完整 M3 矩阵仍 DEFER |

## §5 不变量

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 5.1 | L1 分层 | **PASS** | 路径 `crates/transport`，无业务依赖 |
| 5.2 | 适配器→transport；不反向 | **PASS** | 无 adapter 依赖 |
| 5.3 | 同层隔离 R3 | **PASS** | metadata 无其他 L1 |
| 5.4 | 可恢复失败不 panic | **PASS** | 错误映射 + 测试 |
| 5.5 | 资源边界 M3 | **部分** / 其余 **DEFER** | P0 上限已入；TLS/池/代理矩阵仍 DEFER |
| 5.6 | 实现可用 ≠ M2/M3 | **PASS** | README / AGENTS 明示 |

## §6 测试与验收

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 6.unit | mock + 驱动映射测试 | **PASS** | `tests/*`；多于上游 11 测 |
| 6.not-mock-only | 不得描述为 mock-only | **PASS** | 真实驱动 + loopback |
| 6.not-prod | 不得宣称生产 TLS 就绪 | **PASS** | 文档声明 |
| 6.cmd.binance/okx | adapter 验收命令 | **PASS（生产默认）** | `cargo test -p binancex -p okxx --all-targets`：签名/协议/WS 解析 mock 内容断言；live `server_time` ignore；全量私有流/OCO **DEFER** |
| 6.cmd.local | test/clippy/fmt | **PASS** | 本仓质量门禁 |

## 覆盖率

| 指标 | 目标 | 结果 |
|------|------|------|
| lines | 100% | `cargo llvm-cov -p transportx --fail-under-lines 100` |
| functions | 100%（报告） | 同上 summary |

## 本目标 FAIL 计数

| 范围 | FAIL |
|------|------|
| portable scope（本 workspace 可实现） | **0** |
| monorepo / M3 | DEFER（非 FAIL） |

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 扩大 SSOT DEFER 平台面 |

自验证：`cargo test -p transportx --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p transportx`；`cargo run -p transportx --example …`；`cargo bench -p transportx --bench hot_path -- --quick`。
