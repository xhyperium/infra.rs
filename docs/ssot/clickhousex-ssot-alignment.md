# clickhousex SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `clickhousex` |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 实现 | `crates/adapters/storage/clickhouse` |
| 审计日期 | 2026-07-22 |
| version | `0.3.1` |
| 结论 | **HTTP AnalyticsSink + 分片批量 insert + 有界池已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `ClickHousePool / ClickHouseClient` HTTP |
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

## 验证

```bash
cargo test -p clickhousex --all-targets
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
