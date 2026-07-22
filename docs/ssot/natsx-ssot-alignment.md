# natsx SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| package | `natsx` |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 实现 | `crates/adapters/storage/nats` |
| 审计日期 | 2026-07-22 |
| version | `0.3.2` |
| 结论 | **Core AMO 与 JetStream durable pull 有 broker 证据**；同客户端重启恢复连续三次失败，明确 **NO-GO**；未宣称 package stable |

## 语义矩阵

| 入口 | 状态 | 证据与边界 |
|------|------|------------|
| `NatsEventBus` | Core AMO PASS | 只接收订阅建立后的实时消息，无 ack/回放 |
| `JetStream::publish` | PASS | 等待 publish ack |
| `JetStreamConsumer` | 有条件 PASS | durable pull、Explicit ack、有限 fetch、稳定元数据 |
| ack/nak/progress/term | PASS | 每次错误可观察；`command_timeout` 有界；终结操作消费 delivery 句柄 |
| MaxDeliver/Term | 有条件 PASS | 停止重投；**不等于自动 DLQ** |
| 内部 deadline / capacity | PASS | Core 与 JetStream 命令有界；client/subscription capacity 与有限 reconnect 显式配置 |
| 连接事件 stats | PASS | connected/disconnected/slow-consumer 可观察 |
| 同客户端 broker 重启恢复 | **NO-GO** | 命令通道关闭，30 秒内持续 Unavailable；`infra-2d9.3.1` |
| Cluster/HA/跨账户/KV/ObjectStore | NO-GO | 无对应系统证据 |

## 验证

```bash
cargo test -p natsx --all-targets
cargo clippy -p natsx --all-targets -- -D warnings
node scripts/broker-conformance.mjs
```

broker conformance 已覆盖：Core 无回放、本地有界转发 slow-consumer 观测、JetStream 连接重建后的重投与 double ack、nak/progress、`max_ack_pending` 背压恢复、`max_deliver`/`term` 与 DLQ 探针负向边界。单节点容器结果不证明同客户端自动恢复、Cluster/HA 或自动 DLQ。

`node scripts/nats-reconnect-conformance.mjs` 是当前返回非零的缺陷重现实验，不是绿色发布门禁。

## 相关

- [adapters-ssot-alignment.md](./adapters-ssot-alignment.md)
- [gap-matrix.md](./gap-matrix.md)
