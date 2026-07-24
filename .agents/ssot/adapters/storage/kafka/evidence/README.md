# adapters/storage/kafka — Evidence

> 模块战役证据落盘处（≠ `crates/infra/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 十轮条款矩阵 | [kafkax-10pass-matrix.md](./kafkax-10pass-matrix.md) |
| 单元/集成（离线） | `cargo test -p kafkax --all-targets` |
| 单节点 broker harness | `scripts/kafka-broker-conformance.mjs` → `tests/broker_conformance.rs` |
| TLS/SASL 脱敏 harness | `scripts/kafka-tls-sasl-conformance.mjs` → `tests/tls_sasl_conformance.rs` |
| 受控 live | `crates/adapters/storage/kafka/tests/live_event_bus.rs` |
| bench | `crates/adapters/storage/kafka/benches/hot_path.rs` |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/kafkax-ssot-alignment.md` |

## 2026-07-23 会话证据（脱敏）

| 项 | 结果 |
|----|------|
| 离线测试 | PASS |
| 真 secrets live | **3/3 PASS**（FOUNDATIONX_KAFKAX_*；密钥未入库） |
| broker conformance | **3/3 PASS** |
| TLS/SASL isolation | **2/2 PASS** |
| 十轮矩阵 | 已收敛写入 `kafkax-10pass-matrix.md` |

固定摘要只证明其明确场景；不得外推 group/rebalance/自动重连/HA/native EOS。
失败日志必须先脱敏再展示；secrets 仅环境变量注入。
## 2026-07-23 生产测试矩阵
- `tests/prod_offline.rs` + `tests/prod_reliability.rs`
- `node scripts/kafka-prod-matrix.mjs --fault-restart` PASS

