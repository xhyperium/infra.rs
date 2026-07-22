# transportx SSOT 对齐矩阵

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| SSOT | `.agents/ssot/transport/spec/spec.md` |
| 本仓 crate | `crates/transport` → package `transportx` / lib `transportx` |
| 覆盖率门禁 | `cargo llvm-cov -p transportx --fail-under-lines 100` |

> 镜像写 COMPLETE ≠ 本仓可宣称 ship。本表以 **members + 源码 + 本仓测试** 为准。

## §2 职责与非目标

| ID | 要求 | 状态 | 证据 |
|----|------|------|------|
| 2.1.1 | 统一 HTTP/WS 客户端侧传输边界 | **PASS** | `HttpDriver` / `WsConnector` |
| 2.1.2 | L1，可被适配器依赖 | **PASS** | 仅 kernel + 网络信封 |
| 2.1.3 | 不承载业务契约 | **PASS** | 无业务 trait |
| 2.1.4 | 驱动私有类型封装 | **PASS** | reqwest / tungstenite 私有字段 |
| 2.2.* | 非目标（重试/熔断/bootstrap…） | **PASS** | 未实现；见 resiliencx / bootstrap |

## §3–§4 公开表面（摘要）

| 项 | 状态 | 证据 |
|----|------|------|
| Http/Ws 驱动 + Mock | **PASS** | `src/lib.rs` + tests |
| 429 → RateLimited | **PASS** | reqwest 驱动测 |
| payload 上限 / Debug 脱敏 | **PASS** | #166 |
| **TLS 配置面** | **PASS** | `src/tls.rs` · `TlsMode` / `TlsConfig` |
| **连接/客户端池** | **PASS** | `src/pool.rs` · `HttpClientPool` / `PoolConfig` |
| **代理配置** | **PASS** | `src/proxy.rs` · `ProxyConfig` / `build_reqwest_proxy` |
| 完整生产 TLS 合规矩阵 / mTLS 产品 | **OPEN** | 声明层配置 ≠ 企业 PKI 产品 |
| exchange 业务协议 | **NO-GO**（adapters） | 仅 transport 边界 |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| TLS 矩阵 | DEFER | **PASS（配置面）** | `crates/transport/src/tls.rs` |
| 池 | DEFER | **PASS** | `crates/transport/src/pool.rs` |
| 代理 | DEFER | **PASS** | `crates/transport/src/proxy.rs` |

## 本目标 FAIL 计数

| 范围 | FAIL |
|------|------|
| portable scope（本 workspace 可实现 OBJECTIVE） | **0** |
| 企业 TLS 产品 / exchange 业务 | OPEN / 外域 NO-GO |

## 验证

```bash
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p transportx
```

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%** |
| 声明面生产硬化 | 公共 API 集成测 + bench + docs 红线 |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 企业 PKI 完成 |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：tls/pool/proxy 声明层 PASS |
