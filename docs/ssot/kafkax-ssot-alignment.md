# kafkax SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `kafkax` |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 实现 | `crates/adapters/storage/kafka` |
| 审计日期 | 2026-07-22 |
| version | `0.3.2` |
| 结论 | **AMO、单 owner 应用 ALO、TLS+CA+SASL/PLAIN 有隔离 broker 固定摘要**；group/rebalance/自动重连/native EOS **NO-GO**；未宣称 package stable |

## 语义矩阵

| 入口 | 状态 | 证据与边界 |
|------|------|------------|
| `KafkaEventBus` | AMO PASS | `contracts::EventBus` 无 ack handle |
| `AtLeastOnceConsumer` | 有条件 PASS | 单分区、单 owner、应用自管 checkpoint；无 group/fencing |
| `ProduceThenCheckpointCoordinator` | 非原子 PASS | produce 成功/checkpoint 失败会重复；不称 EOS |
| `FileOffsetStore` | 有条件 PASS | 单进程同目录 temp+sync+rename+父目录 sync；不承诺多进程/绝对掉电持久 |
| TLS / CA | PASS | rskafka rustls transport；webpki roots + 可选 PEM CA；远程明文 fail-closed |
| SASL | 有条件 PASS | 仅 PLAIN；未知/不完整机制 fail-closed |
| deadline | PASS | connect（含 CA 装载）/metadata/topic/partition client/produce 均有界；delivery timeout 不放大 |
| backpressure / close | PASS | consumer/EventBus 有界队列；close 取消等待并按 deadline 等待在途操作；超时后仍关闭 |
| 错误边界 | PASS | 稳定 `ErrorKind`；驱动原文、SASL 用户名/密码不进入公开错误或 Debug |
| group/rebalance/自动重连/native EOS/DLQ | NO-GO | 没有对应实现或证据 |

旧 `EosCoordinator` / `EosSession` 仅为 deprecated 源码兼容别名。内置 checkpoint 单调前进，拒绝负 offset 与 `i64::MAX` 溢出。

## 验证

```bash
cargo test -p kafkax --all-targets
cargo clippy -p kafkax --all-targets -- -D warnings
node scripts/kafka-broker-conformance.mjs
node scripts/kafka-tls-sasl-conformance.mjs
```

broker conformance 已覆盖 AMO/ALO/重复窗口；固定摘要 SASL_SSL 实验覆盖受信 CA + PLAIN 成功以及错误 CA/密码 fail-closed。单节点结果不证明 group/rebalance/HA/native EOS。

本轮 Code 会话实际运行两个隔离 harness：Kafka AMO/ALO/重复窗口 `3/3` PASS，
TLS+SASL/PLAIN 正向与错误 CA/密码 fail-closed `2/2` PASS，命令原始退出码均为 0；
容器与临时凭据已清理。该结果仍不证明 group/rebalance/自动重连/HA/native EOS。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
