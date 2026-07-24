# adapters/storage/redis — Evidence

> 模块战役证据落盘处（≠ `crates/infra/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 单元/集成（离线） | `cargo test -p redisx --all-targets --features pubsub` |
| live | `crates/adapters/storage/redis/tests/live_*.rs`（`#[ignore]`） |
| bench | `crates/adapters/storage/redis/benches/kv_hot_path.rs` |
| gap + 10 轮审查（2026-07-23） | [2026-07-23/](./2026-07-23/) |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft 入库 | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md` |

## 2026-07-23 本轮摘要

| 项 | 结果 |
|----|------|
| version | `0.3.15` |
| offline default | 90 passed + live ignored |
| offline pubsub | 96 passed + live ignored |
| live Standalone | KV 5 + conformance 2 + pubsub 1 passed（真实 Redis） |
| Cluster/Sentinel/TLS live | OPEN |
| package stable | 禁止宣称 |

有新的验证输出时按 `YYYY-MM-DD/` 建日目录归档。
