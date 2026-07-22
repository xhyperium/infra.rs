# clickhousex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 实现 | `crates/adapters/storage/clickhouse` |
| 审计日期 | 2026-07-23 |
| version | `0.3.2` |
| 结论 | **HTTP(S) AnalyticsSink + PEM CA + 分片批量 insert + 有界池已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `ClickHousePool / ClickHouseClient` HTTP(S) |
| HTTPS / CA | rustls roots + 可选 PEM CA；远程 HTTP fail-closed；本地 TLS 协议实验 PASS |
| batch insert | `insert_batch` + `BatchInsertOptions.max_rows_per_chunk` + `chunk_ranges` |
| pool strength | `max_idle_per_host` / `max_in_flight` + Semaphore + `ClickHousePoolStats` |
| contracts | `AnalyticsSink` |
| 环境变量 | `HTTP_PORT` 优先、兼容 `PORT`；双设不同值 fail-closed |
| 错误安全 | 最多读取 4096 字节异常前缀；只暴露 HTTP 状态/数字错误码，不回显 SQL/payload/认证正文 |
| live | `tests/live_smoke.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（批量 insert / 池强度） |
| 仍 OPEN | 真实 ClickHouse TLS/auth/deadline/并发、native 9000、cluster/ReplicatedMergeTree |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| CLICKHOUSEX-1–8 | member…SSOT | PASS | — |
| CLICKHOUSEX-9 | package stable | OPEN | 禁止宣称 |
| CLICKHOUSEX-10 | 批量 insert | PASS | `insert_batch` / `chunk_ranges` |
| CLICKHOUSEX-11 | 池强度 | PASS | config + Semaphore |
| CLICKHOUSEX-12 | HTTPS/CA 与远程明文拒绝 | PASS | config/client + `https_conformance.rs` |
| CLICKHOUSEX-13 | PORT alias 一致性 | PASS | 主变量优先、同值兼容、异值拒绝单测 |
| CLICKHOUSEX-14 | 错误正文不泄漏 | PASS | `security_failures.rs` + 固定数字码映射 |
| CLICKHOUSEX-15 | 真实 TLS/auth/deadline/并发 | OPEN | 未运行真实 ClickHouse，不伪造 evidence |

## 验证

```bash
cargo test -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
node scripts/clickhouse-https-conformance.mjs
cmp .agents/ssot/adapters/storage/clickhouse/spec/spec.md \
  .agents/ssot/adapters/storage/clickhouse/spec/xhyper-clickhousex-complete-spec.md
```

HTTPS 实验验证客户端传输与 CA/主机名校验，**不**证明真实 ClickHouse 集群、复制或故障切换。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
