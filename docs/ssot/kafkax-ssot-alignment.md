# kafkax SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 实现 | `crates/adapters/storage/kafka` |
| 审计日期 | 2026-07-22 |
| version | `0.3.2` |
| 结论 | **AMO 与单 owner 应用 ALO 有 broker 证据**；TLS/group/rebalance/native EOS **NO-GO**；未宣称 package stable |

## 语义矩阵

| 入口 | 状态 | 证据与边界 |
|------|------|------------|
| `KafkaEventBus` | AMO PASS | `contracts::EventBus` 无 ack handle |
| `AtLeastOnceConsumer` | 有条件 PASS | 单分区、单 owner、应用自管 checkpoint；无 group/fencing |
| `ProduceThenCheckpointCoordinator` | 非原子 PASS | produce 成功/checkpoint 失败会重复；不称 EOS |
| `FileOffsetStore` | 有条件 PASS | 单进程同目录 temp+sync+rename+父目录 sync；不承诺多进程/绝对掉电持久 |
| TLS | NO-GO | 当前 rskafka 构建未接入；配置 fail-closed |
| group/rebalance/native EOS/DLQ | NO-GO | 没有对应实现或证据 |

旧 `EosCoordinator` / `EosSession` 仅为 deprecated 源码兼容别名。内置 checkpoint 单调前进，拒绝负 offset 与 `i64::MAX` 溢出。

## 验证

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
./scripts/broker-conformance.sh
```

broker conformance 已覆盖：未 ack 重建同 offset、ack 后推进、produce 成功/checkpoint 失败后的可观察重复。单节点容器结果不证明 group/rebalance/HA/TLS/native EOS。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
