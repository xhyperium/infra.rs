# adapters/storage/redis — Evidence

> 模块战役证据落盘处（≠ `crates/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 单元/集成（离线） | `cargo test -p redisx --all-targets --features pubsub` 日志 |
| live | `crates/adapters/storage/redis/tests/live_kv.rs` 等 |
| bench | `crates/adapters/storage/redis/benches/kv_hot_path.rs` |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md` |

当前最终本地测试为 51 passed + 8 ignored；ignored live 项需要外部 Redis。候选曾冻结；治理修正后
最终 SHA 待重冻，当前没有最终 SHA 的 reviewer/verifier 或 CI artifact。

有新的验证输出时按 `YYYY-MM-DD/` 建日目录归档。
