# natsx 实现规范

状态：当前 `0.3.2` 实现合同（`async-nats 0.50` 默认真实路径；Core 与 JetStream 语义分层）。**未宣称 package stable。**

## 0. 权威、职责与范围

裁定顺序为 Constitution → 组织 Rust 规范 → 本文 → 代码与可复验证据。规格中的“完成”不等于 workspace Production Ready。

`natsx` 位于 `crates/adapters/storage/nats`。默认构建包含真实 Core NATS pool/EventBus 和 JetStream 包装；旧内存实现仅在 `scaffold` feature 下导出。

## 1. 交付语义矩阵

| 入口 | 当前语义 | 确认所有权 | 明确非目标 |
|---|---|---|---|
| `NatsEventBus` / `contracts::EventBus` | Core NATS at-most-once、仅实时订阅 | 无 ack/重放 | 不得称持久或 ALO |
| `JetStream::publish` | 等待 JetStream publish ack | 发布端等待服务端确认 | 跨账户/Cluster 运维合同 |
| `JetStreamConsumer` | durable pull、显式 ack、有限 fetch | 每次 `JetStreamDelivery` 由调用方终结 | 自动业务重试、自动 DLQ、exactly-once |

Core NATS 与 JetStream 是两个独立合同。Core 发布发生在订阅建立前时不会回放；需要持久消费的调用方必须显式选择 JetStream。

## 2. JetStream consumer 合同

`JetStreamConsumerConfig` 是 additive 新类型，不给旧公开 `PullConsumerConfig` 追加字段。配置固定 `AckPolicy::Explicit`，并要求：

- `ack_wait > 0`；
- `max_deliver > 0`；
- `max_ack_pending > 0`；
- `command_timeout > 0`，用于约束 ack/nak/progress/term 等 broker 指令；
- durable name 合法，filter subject 若存在则非空。

`JetStreamConsumer::next_timeout` 同时设置服务端 fetch expiry 与客户端外层超时。服务端有限批次正常到期为 `Ok(None)`；broker/协议错误保留 source 并返回 `Unavailable`；服务端未在 expiry 后结束则返回 `DeadlineExceeded`。

`JetStreamDelivery` 复制并公开稳定元数据：stream、consumer、stream sequence、consumer sequence、delivery attempts、pending。底层 raw message 私有，`Debug` 只输出 subject、payload 长度和元数据，不输出 payload。

终结操作：

- `ack(self)`：发送普通确认；
- `double_ack(self)`：等待服务端确认；
- `nak(self, delay)`：请求立即或延迟重投；
- `progress(&self)`：延长处理窗口；
- `term(self)`：停止该消息继续重投。

`term` 与 `max_deliver` 都不等于 DLQ：库不会自动把 payload 发布到隔离 subject。conformance 只对约定的 DLQ 探针作负向检查；其他业务路由由应用显式实现并验证。

## 3. 安全与运维边界

- loopback 默认 `TlsPolicy::Prefer`，非 loopback 默认 `Require`；凭据只从环境注入并在 `Debug` 中脱敏。
- 本版不承诺 NKey 全量、JetStream KV/ObjectStore、跨账户、Cluster/HA、queue group 运维或自动 DLQ。
- 单节点容器 conformance 不能作为上述能力证据。
- `get_pull_consumer` 保留为底层高级逃生口；普通调用方使用稳定 `consumer` 包装面。

## 4. 验证与证据

离线门禁：

```bash
cargo test -p natsx --all-targets
cargo clippy -p natsx --all-targets -- -D warnings
```

可复现 broker conformance：

```bash
node scripts/broker-conformance.mjs
```

NATS 场景必须证明：

1. Core NATS 不回放订阅前消息；
2. JetStream 连接重建后按相同 stream sequence 重投，double ack 后停止；
3. `nak` 触发重投，`progress` 延长 ack wait；
4. `max_ack_pending=1` 在未 ack 时背压、ack 后恢复；
5. `max_deliver` 与 `term` 停止重投且 DLQ 探针无消息；
6. 唯一 stream/subject/durable、cargo 外层硬超时、日志与清理。

受控外部环境仍可运行 `tests/live_event_bus.rs`，但 ignored 或单节点 PASS 不得升级为 Cluster/HA/TLS/exactly-once 结论。

追溯：`crates/adapters/storage/nats/{Cargo.toml,src,tests/broker_conformance.rs}`、`docs/ssot/natsx-ssot-alignment.md`。
