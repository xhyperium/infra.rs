# transportx SSOT 对齐矩阵

| 字段 | 值 |
|---|---|
| 审计日期 | 2026-07-23 |
| baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| Active Spec | `.agents/ssot/transport/spec/spec.md`（与 xhyper 镜像 byte-identical） |
| crate | `crates/transport` · package/lib `transportx` |
| version | `0.1.3`（非 package stable） |
| 声明边界 | 既有 HTTP/WS 客户端与配置面；未达 M3 |

## Maintenance 对齐

| ID | 要求 | 状态 | 代码/测试证据 |
|---|---|---|---|
| TR-HTTP-1 | chunk 流式累计、首次越界中止 | IMPLEMENTED | `ReqwestHttpDriver::execute`；stalling chunked loopback |
| TR-WS-1 | 解码/聚合前 frame/message 上限 | IMPLEMENTED | `connect_async_with_config`；入站 32>8 loopback |
| TR-DBG-1 | URL userinfo/query fail-closed 脱敏 | IMPLEMENTED | `RedactedUrl`；Request/Proxy/非法 URL 测试 |
| TR-TLS-1 | 未接线 `sni=false` 不静默 | IMPLEMENTED | builder ProtocolViolation；default `sni=true` 测试 |
| TR-POOL-1 | 有界配置 + RAII lease | IMPLEMENTED | `try_new` / `HttpClientLease`；drop/into_inner 回收 |
| TR-RATE-1 | RFC 9110 Retry-After | IMPLEMENTED | `parse_retry_after_at` seconds/date/past/invalid |

旧 `checkout_with`/`return_client`、`HttpDriver`、`WsConnector` 与错误变体保留。新增 `httpdate` 按 workspace dependency 管理。binancex/okxx 仅同步 transportx path version，不 bump 消费者。

## 成熟度与 NO-GO

| 项 | 裁定 |
|---|---|
| HTTP/WS 受控实现面 | 有本地回归证据 |
| 代理/TLS 配置面 | 仅既有 reqwest 配置；`sni=false` 明确拒绝 |
| 企业 PKI/mTLS、WS 企业 TLS | **OPEN / NO-GO** |
| M3 故障恢复、长稳、完整认证矩阵 | **OPEN / NO-GO** |
| binance/okx 完整业务 live | adapter 外域 **NO-GO**；本轮不宣称 |

## 验证

最终命令与退出码写入 `.agents/ssot/transport/evidence/README.md`。本地 loopback、测试和 coverage 通过都不能升级为公网或业务 readiness。maintainer/独立 reviewer 未签署前 release 保持 BLOCKED。
