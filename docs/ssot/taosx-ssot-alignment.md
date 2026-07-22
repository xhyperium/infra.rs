# taosx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `taosx` |
| SSOT | `.agents/ssot/adapters/storage/taos/` |
| 实现 | `crates/adapters/storage/taos` |
| 审计日期 | 2026-07-22 |
| version | `0.3.1` |
| 结论 | **REST TimeSeriesStore + batch write + Native WS 探测 + 有界池已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `TaosPool / TaosClient` REST（6041） |
| batch write | `write_batch` / `write_batch_chunked` / `build_insert_sql_chunks` |
| native | `TransportMode::NativeWs` + `connect_native_ws`（连通性探测；SQL 仍 REST） |
| pool | `max_in_flight` Semaphore + `TaosPoolStats` |
| contracts | `TimeSeriesStore`（ts 纳秒 epoch） |
| 环境变量 | `FOUNDATIONX_TAOSX_{HOST,PORT,USER,PASSWORD,DATABASE,TLS,PRECISION,TRANSPORT,...}` |
| live | `tests/live_smoke.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（batch / native 路径 / pool） |
| 仍 OPEN（非 OBJECTIVE） | 完整 WS SQL 会话 / 超表治理运维 / 集群 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| TAOSX-1–8 | member…SSOT | PASS | — |
| TAOSX-9 | package stable | OPEN | 禁止宣称 |
| TAOSX-10 | 批量写 | PASS | `write_batch*` / `build_insert_sql_chunks` |
| TAOSX-11 | native 路径 | PASS | `src/native.rs` + `TransportMode` |
| TAOSX-12 | 池强度 | PASS | max_in_flight + stats |

## 诚实边界

- Native WS 当前为 **握手连通性探测**；时序 SQL 执行默认仍走 REST。完整 WS SQL 会话为 follow-up。

## 验证

```bash
cargo test -p taosx --all-targets
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
