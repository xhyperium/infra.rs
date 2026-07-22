# natsx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 实现 | `crates/adapters/storage/nats` |
| 审计日期 | 2026-07-22 |
| version | `0.3.2` |
| 结论 | **Core AMO 与 JetStream durable pull 有 broker 证据**；Cluster/HA/自动 DLQ 未承诺；未宣称 package stable |

## 语义矩阵

| 入口 | 状态 | 证据与边界 |
|------|------|------------|
| `NatsEventBus` | Core AMO PASS | 只接收订阅建立后的实时消息，无 ack/回放 |
| `JetStream::publish` | PASS | 等待 publish ack |
| `JetStreamConsumer` | 有条件 PASS | durable pull、Explicit ack、有限 fetch、稳定元数据 |
| ack/nak/progress/term | PASS | 每次错误可观察；`command_timeout` 有界；终结操作消费 delivery 句柄 |
| MaxDeliver/Term | 有条件 PASS | 停止重投；**不等于自动 DLQ** |
| Cluster/HA/跨账户/KV/ObjectStore | NO-GO | 无对应系统证据 |

## 验证

```bash
cargo test -p natsx --all-targets
cargo clippy -p natsx --all-targets -- -D warnings
node scripts/broker-conformance.mjs
```

broker conformance 已覆盖：Core 无回放、JetStream 连接重建后的重投与 double ack、nak/progress、`max_ack_pending` 背压恢复、`max_deliver`/`term` 与 DLQ 探针负向边界。单节点容器结果不证明 Cluster/HA 或自动 DLQ。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
