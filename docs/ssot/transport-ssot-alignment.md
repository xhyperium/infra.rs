# transportx SSOT 对齐

| 字段 | 值 |
|---|---|
| Baseline | `2299ff1f9c6d006d014c80d89a3082a01ba27c9a` |
| Active SSOT | `.agents/ssot/transport/spec/spec.md` ≡ 双镜像 |
| Crate | `crates/infra/transport` · package/lib `transportx` · 候选 `0.1.4` |
| 范围 | HTTP/WS 客户端传输、TLS/代理配置、进程内客户端池 |
| 本地证据 | [`manifest.json`](../../evidence/testkit/2026-07-23-infra-2d9.10/manifest.json) 绑定固定代码与本地门禁结果 |
| 状态 | 行为已实现，本地 workspace 门禁已运行；PR CI、独立终审、人工批准与 merge 均为 OPEN |

## 本轮三次收敛

| 轮次 | 发现 | 收敛 |
|---|---|---|
| R1 安全 | URL userinfo/query 可经 Debug 泄漏；SNI false 被静默忽略 | 统一 URL fail-closed 脱敏；SNI false 构造时拒绝 |
| R2 资源 | chunked 响应与 WS 聚合可能在完整缓冲后才判超限 | HTTP 逐 chunk 累计；WS decoder 前置 frame/message 上限 |
| R3 生命周期 | 池配置可无效，手动归还易泄漏；Retry-After 语义过窄 | `try_new` + RAII lease + 精确许可恢复；支持 HTTP-date |

## 合同矩阵

| 要求 | 实现证据 | 候选状态 |
|---|---|---|
| RFC 9110 Retry-After | `parse_retry_after_at` + parser 测试 | PASS |
| URL Debug fail-closed | `RedactedUrl` + request/proxy 测试 | PASS |
| chunked 累计上限 | `ReqwestHttpDriver::execute` + stall server 测试 | PASS |
| SNI 默认/拒绝 | `TlsConfig::default` + builder 测试 | PASS |
| 池校验/RAII | `PoolConfig::validate` / `HttpClientLease` + 公共测试 | PASS |
| WS 入站 decoder 上限 | tungstenite config + loopback 测试 | PASS |
| `httpdate` 依赖评估 | `Cargo.lock` / `cargo tree` / `cargo deny check` | PASS；锁定 `1.0.3`，仅 transportx 直接使用 |
| 企业 PKI/mTLS/M3 | 无相应实现或 live 证据 | NO-GO |

## 验证

```bash
cmp .agents/ssot/transport/spec/spec.md \
  .agents/ssot/transport/spec/xhyper-transportx-complete-spec.md
cargo test -p transportx --all-targets
cargo clippy -p transportx --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p transportx --no-deps
```

本地 PASS 不外推为公网业务 live、企业 TLS 或 package stable。
