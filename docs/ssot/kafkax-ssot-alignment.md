# kafkax SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 实现 | `crates/adapters/storage/kafka` |
| 审计日期 | 2026-07-22 |
| version | `0.3.1` |
| 结论 | **生产 EventBus + 应用层 at-least-once/offset/EOS 已落地**；**未**宣称 package stable |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 生产默认面 | `KafkaPool / KafkaProducer / KafkaConsumer / KafkaEventBus` |
| offset commit | `OffsetCommitStore` + `MemoryOffsetStore` / `FileOffsetStore` |
| at-least-once | `AtLeastOnceConsumer` / `KafkaAtLeastOnceBus`（显式 ack） |
| EOS | `EosCoordinator` / `EosSession`（**应用级**；rskafka 无 transactional producer） |
| contracts | `EventBus`（默认 AMO；at-least-once 另面） |
| 环境变量 | `FOUNDATIONX_KAFKAX_{BROKERS,SASL_MECHANISM,SASL_USERNAME,SASL_PASSWORD,TLS}` |
| live | `tests/live_event_bus.rs`（`#[ignore]`） |
| 原 OBJECTIVE DEFER | **PASS**（offset commit / at-least-once / 应用级 EOS） |
| 仍 OPEN（非 OBJECTIVE） | schema registry / group coordinator 强依赖路径 / broker 事务 |

## 对齐矩阵

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| KAFKAX-1 | workspace member | PASS | metadata |
| KAFKAX-2 | 生产默认导出 | PASS | `src/lib.rs` |
| KAFKAX-3 | from_env | PASS | config |
| KAFKAX-4 | 离线测试 | PASS | 22 unit |
| KAFKAX-5 | live 入口 | PASS | live_event_bus |
| KAFKAX-6 | bench 有界 | PASS | benches/hot_path |
| KAFKAX-7–8 | docs / SSOT | PASS | — |
| KAFKAX-9 | package stable | OPEN | 禁止宣称 |
| KAFKAX-10 | offset commit | PASS | `src/offset.rs` |
| KAFKAX-11 | at-least-once | PASS | `src/at_least_once.rs` |
| KAFKAX-12 | 应用级 EOS | PASS | `src/eos.rs` |

## 诚实边界

- EOS 为 **produce 成功后才允许 commit** 的应用协调，**不是** Kafka 事务/幂等 producer 协议。
- 业务副作用写出但 commit 失败时会重投；调用方需幂等处理。

## 验证

```bash
cargo test -p kafkax --all-targets
```

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
