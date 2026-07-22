# clickhousex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 实现 | `crates/adapters/storage/clickhouse` |
| 审计日期 | 2026-07-22 |
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
| 环境变量 | `FOUNDATIONX_CLICKHOUSEX_{HOST,HTTP_PORT,USER,PASSWORD,DATABASE,...}` |
| live | `tests/live_smoke.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（批量 insert / 池强度） |
| 仍 OPEN（非 OBJECTIVE） | native 9000 protocol / cluster / ReplicatedMergeTree 运维面 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| CLICKHOUSEX-1–8 | member…SSOT | PASS | — |
| CLICKHOUSEX-9 | package stable | OPEN | 禁止宣称 |
| CLICKHOUSEX-10 | 批量 insert | PASS | `insert_batch` / `chunk_ranges` |
| CLICKHOUSEX-11 | 池强度 | PASS | config + Semaphore |
| CLICKHOUSEX-12 | HTTPS/CA 与远程明文拒绝 | PASS | config/client + `https_conformance.rs` |

## 验证

```bash
cargo test -p clickhousex --all-targets
cargo clippy -p clickhousex --all-targets -- -D warnings
node scripts/clickhouse-https-conformance.mjs
```

HTTPS 实验验证客户端传输与 CA/主机名校验，**不**证明真实 ClickHouse 集群、复制或故障切换。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
