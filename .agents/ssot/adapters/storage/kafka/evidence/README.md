# adapters/storage/kafka — Evidence

> 模块战役证据落盘处（≠ `crates/evidence` 生产库）。

## 本仓证据索引

| 类型 | 位置 |
|------|------|
| 单元/集成（离线） | `cargo test -p kafkax --all-targets` 日志 |
| 单节点 broker harness | `scripts/kafka-broker-conformance.mjs` → `tests/broker_conformance.rs` |
| TLS/SASL 脱敏 harness | `scripts/kafka-tls-sasl-conformance.mjs` → `tests/tls_sasl_conformance.rs` |
| 受控 live | `crates/adapters/storage/kafka/tests/live_event_bus.rs` |
| bench | `crates/adapters/storage/kafka/benches/hot_path.rs` |
| landing | [../plan/infra-rs-landing.md](../plan/infra-rs-landing.md) |
| draft | [../plan/infra-rs-draft-spec-goal.md](../plan/infra-rs-draft-spec-goal.md) |
| 对齐 | `docs/ssot/kafkax-ssot-alignment.md` |

固定摘要只证明其明确场景；harness 存在不等于当前会话 PASS。本轮未运行 broker/TLS
环境时不得创建或改写 PASS evidence，且不得外推 group/rebalance/自动重连/native EOS。

2026-07-23 本轮 Code 输出记录：`kafka-broker-conformance.mjs` 为 `3/3` PASS，
`kafka-tls-sasl-conformance.mjs` 为 `2/2` PASS，原始退出码均为 0；runner 已清理容器与
临时凭据。未生成伪造日志文件，原始命令输出随本轮交付给 Test。
