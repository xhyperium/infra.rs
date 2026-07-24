# adapters/storage/postgres — Evidence

> 模块战役证据落盘处（≠ `crates/infra/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 单元/集成（离线） | `cargo test -p postgresx --all-targets` 日志 |
| live | `crates/adapters/storage/postgres/tests/live_postgres.rs` |
| deadline | `scripts/postgres-deadline-conformance.mjs` + `tests/deadline_conformance.rs` |
| bench | `crates/adapters/storage/postgres/benches/query_hot_path.rs` |
| 10 轮 draft 审查 | [2026-07-23/postgresx-10x-review.md](./2026-07-23/postgresx-10x-review.md) |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/postgresx-ssot-alignment.md` |

## 2026-07-23 快照

| 项 | 结果 |
|----|------|
| offline unit | 43 passed |
| live (`--ignored`) | 9 passed（dev secrets 注入，loopback disable TLS） |
| deadline conformance | passed（固定镜像 Postgres 17） |
| bench `query_hot_path` | 200 iters 完成，有界 |
| package stable | **未宣称** |
| 远程 TLS live | **OPEN** |
