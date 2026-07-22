# natsx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 实现 | `crates/adapters/storage/nats` |
| 审计日期 | 2026-07-22 |
| version | `0.3.1` |
| 结论 | **Core NATS + JetStream 薄封装 + TLS 默认策略已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `NatsPool / NatsEventBus / NatsSubscription` |
| JetStream | `JetStream`：publish / get_or_create_stream / create_pull_consumer |
| TLS 默认策略 | loopback → Prefer；非 loopback → Require（`require_tls(true)`） |
| contracts | `EventBus`（Core，at-most-once） |
| 环境变量 | `FOUNDATIONX_NATS_{URL,USER,PASSWORD,TLS,TLS_POLICY,JETSTREAM}` 或 `NATSX_*` |
| live | `tests/live_event_bus.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（JetStream / TLS 默认策略） |
| 仍 OPEN（非 OBJECTIVE） | NKey / JetStream KV·ObjectStore 全量 / Cluster 运维面 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| NATSX-1–8 | member/export/env/test/live/bench/docs/SSOT | PASS | — |
| NATSX-9 | package stable | OPEN | 禁止宣称 |
| NATSX-10 | JetStream | PASS | `src/jetstream.rs` |
| NATSX-11 | TLS 默认策略 | PASS | `TlsPolicy` + pool `require_tls` |

## 验证

```bash
cargo test -p natsx --all-targets
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
