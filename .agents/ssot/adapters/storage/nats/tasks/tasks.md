# adapters/storage/nats — Tasks

> 对齐审计：`docs/ssot/natsx-ssot-alignment.md` (v0.3.3) | `matrix/matrix.md` S1–S10 | 三轮证据：`docs/report/2026-07-23/storage-round3-evidence.md`

## P0（已关闭 · #188–#191）

- [x] 生产默认客户端与配置
- [x] unit / lib 测试
- [x] live `#[ignore]` 测试：`tests/live_event_bus.rs`
- [x] bench：`benches/hot_path.rs（3s 有界）`
- [x] crate docs：usage / config / operations
- [x] SSOT landing + draft 快照
- [x] docs/ssot/natsx-ssot-alignment.md

## P0 后补充（已关闭 · v0.3.1–v0.3.3）

- [x] **TLS 策略** (v0.3.1)：`TlsPolicy { Disable, Prefer, Require }` + loopback/remote 自动判定 + 环境变量 `FOUNDATIONX_NATS_TLS{,_POLICY}` + Remote 明文 fail-closed
- [x] **JetStream 薄封装** (v0.3.1)：`JetStream { publish, get_or_create_stream, create_pull_consumer }` + `StreamConfig` / `PullConsumerConfig` + `validate_stream_name`
- [x] **JetStream durable pull 显式确认** (v0.3.2)：`JetStreamConsumerConfig` / `JetStreamConsumer` / `JetStreamDelivery` + `ack` / `double_ack` / `nak` / `progress` / `term` + `JetStreamDeliveryMetadata`
- [x] **连接事件 stats** (v0.3.2)：`NatsPoolStats { connected, disconnected, slow_consumers }` + 订阅任务注册表管�� + panic fail-closed
- [x] **internal deadline 全覆盖** (v0.3.2)：Core 与 JetStream 全部命令配置 `operation_timeout` / `command_timeout`
- [x] **broker conformance 7 场景** (v0.3.2)：Core 无回放、慢消费者、JetStream 重投/double_ack、Nak/Progress、max_ack_pending 背压、max_deliver/term DLQ 负向
- [x] **reconnect conformance 3/3** (v0.3.2–v0.3.3)：固定镜像同 client 重启恢复发布 + 原 Core subscription + 慢消费者观测
- [x] **validate fail-closed 补齐** (v0.3.3)：`validate_operation_timeout` 零值拒绝、`validate_consumer_name` 通配/空白专项单测
- [x] **live negative paths** (v0.3.3)：`get_pull_consumer` 非法名/缺失目标、`with_operation_timeout(0)` 不污染上下文

## P1（当前高优先级）

- [ ] **P1-1** package stable 证据包
  - 24h soak 测试（无内存/任务/FD 泄漏）
  - 10k 并发 publish 有界验证
  - 生产级 TLS live 证据（非 lab loopback）
  - 跨版本 API 兼容性声明
  - crates.io 发布流程就绪（当前 `publish = false`）
- [x] **P1-2** JetStream API 补齐（管理面）
  - [x] `delete_stream` / `get_stream_info` / `purge_stream` 公开 API（purge_stream 为 delete+recreate 占位，async-nats 0.50 `&mut self` 限制）
  - [x] consumer info / pending 查询
  - [x] 批次 fetch（`next_batch`，`max_messages > 1`）
- [x] **P1-3** JetStream ephemeral consumer
  - [x] `JetStreamConsumerConfig::ephemeral()` + `durable_name: Option<String>`
- [x] **P1-4** NKey 认证基础路径
  - [x] `NatsConfig` 增加 NKey seed/jwt 字段
  - [x] 环境变量注入（`FOUNDATIONX_NATS_NKEY_SEED` / `FOUNDATIONX_NATS_JWT`）
  - [x] Debug 脱敏 + validate 互斥/路径校验
  - [ ] connect() 端到端连接集成（需 async-nats `nkeys` feature 启用）
  - consumer info / pending 查询
  - 批次 fetch（`max_messages > 1`）
- [ ] **P1-3** JetStream ephemeral consumer
  - 非 durable 临时 consumer 构造路径
- [ ] **P1-4** NKey 认证基础路径
  - `NatsConfig` 增加 NKey seed/jwt 字段
  - 环境变量注入（`FOUNDATIONX_NATS_NKEY_SEED` / `FOUNDATIONX_NATS_JWT`）
  - Debug 脱敏 + broker conformance 验证

## P2（后续）

- [ ] **P2-1** Core NATS Request-Reply
  - `NatsPool::request(subject, payload, deadline)` — 唯一 inbox + 总 deadline
- [ ] **P2-2** Headers 传递
  - `NatsMessage` 增加 `headers` 字段
  - publish / subscribe / JetStream publish 传递 headers
- [ ] **P2-3** JetStream push consumer
  - push subscribe 模式封装
- [ ] **P2-4** TLS 证书配置（进阶）
  - CA bundle 路径注入
  - mTLS client certificate 配置
  - 证书轮换 callback
- [ ] **P2-5** Graceful drain
  - shutdown 序列：先排空 inflight → 再 close
  - 超时后强制 cancel
- [ ] **P2-6** JetStream direct get / mirror
  - `get_message(stream, seq)` / `get_last_message(stream, subject)`
  - stream mirror/source 配置暴露

## NO-GO（明确排除）

以下项在 v0.3.x 范围内**不承诺**，不得从单节点 conformance 外推为"已支持"：

- [ ] Cluster / HA（多节点协调、故障转移）
- [ ] 跨账户 (cross-account)
- [ ] JetStream KV（Key-Value Store）
- [ ] JetStream ObjectStore
- [ ] 自动 DLQ（Dead-Letter Queue）—— `term` / `max_deliver` 停止重投，不自动发布到隔离 subject
- [ ] Core 断线窗口消息回放
- [ ] 无限自愈（超过 `max_reconnects` 后必须调用方重建 client）

## 验证矩阵

### 一、Conformance 覆盖（已实现）

> 来源：`tests/broker_conformance.rs` / `tests/reconnect_conformance.rs` / `tests/live_event_bus.rs`
> 默认 `#[ignore]`，由 `scripts/broker-conformance.mjs` / `scripts/nats-reconnect-conformance.mjs` 启动隔离环境。

| 分组 | 测试用例 | 覆盖点 |
|------|---------|--------|
| Core | `core_nats_does_not_replay_messages_published_before_subscribe` | 订阅建立前发布的消息不被回放 |
| Core | `bounded_subscription_reports_local_slow_consumer` | 本地有界转发超时 → `slow_consumers` 计数字段递增 |
| JetStream | `jetstream_redelivers_until_double_ack` | 连接重建后按相同 `stream_sequence` 重投；`double_ack` 停止后续投递 |
| JetStream | `max_ack_pending_applies_backpressure_until_ack` | `max_ack_pending=1` 未确认时背压不投递第二条；`ack` 后恢复 |
| JetStream | `nak_redelivers_and_progress_extends_ack_wait` | `nak` 触发重投、`progress` 延长 ack 等待窗口 |
| JetStream | `max_deliver_and_term_do_not_publish_conventional_dlq_subject` | `max_deliver` 达到上限停止重投；`term` 停止后续投递；DLQ 探针无消息 |
| JetStream | `with_operation_timeout_zero_fails_closed_without_mutating_context` | 零 deadline 被拒绝、原始上下文仍可用 |
| JetStream | `get_pull_consumer_rejects_invalid_names_and_missing_targets` | 非法 stream/consumer 名返回 `Invalid`；不存在目标返回 `Unavailable`；已有 durable consumer 可获取 |
| Reconnect | `reconnect_restores_subscription_and_slow_consumer_is_observable` | 同一 client 重启 broker 进程后发布 + 原 Core subscription 恢复 + 连接/断开/慢消费者事件统计连续 3/3 |
| Live | `live_pub_sub_content` | Core pub-sub 内容往返 + health 检查 |
| Live | `live_event_bus_roundtrip` | `contracts::EventBus` facade 完整往返 |

### 二、Conformance 缺口（缺失）

> 对应 P1/P2 任务项，当前无 `#[ignore]` broker 证据。

| 分组 | 缺失场景 | 关联任务 | 优先级 |
|------|---------|---------|:------:|
| Core | Request-Reply 超时/重试 | P2-1 | P2 |
| Core | Headers 传递端到端 | P2-2 | P2 |
| Core | Queue group 多订阅者分配 | — | NO-GO |
| JetStream | 批次 fetch（`max_messages > 1`）并发正确性 | P1-2 | P1 |
| JetStream | Ephemeral consumer 创建/销毁 | P1-3 | P1 |
| JetStream | Stream 管理操作（delete / purge / get_info） | P1-2 | P1 |
| JetStream | Consumer info / pending 查询 | P1-2 | P1 |
| JetStream | Push consumer 投递语义 | P2-3 | P2 |
| JetStream | Direct get（by sequence / last per subject） | P2-6 | P2 |
| Auth | NKey 认证 broker conformance | P1-4 | P1 |
| Auth | TLS 真实证书 live 验证 | P1-1 | P1 |
| Auth | TLS 证书配置 live 验证（CA/mTLS） | P2-4 | P2 |
| Drain | Graceful drain 不丢 in-flight 消息 | P2-5 | P2 |
| Reconnect | 超过 `max_reconnects` 后 channel 关闭行为 | — | P2 |
| Soak | 24h 无内存/任务/FD 泄漏 | P1-1 | P1 |
| Soak | 10k 并发 publish 有界 | P1-1 | P1 |

### 三、能力维度验证（已实现 / 未实现）

| 项 | 状态 | 证据 |
|----|:----:|------|
| Core AMO EventBus | ✓ | `tests/live_event_bus.rs` / broker conformance #1 |
| JetStream durable pull + explicit ack | ✓ | broker conformance #3–#8 |
| TLS policy (3-level) | ✓ | `config.rs` 单测：loopback/remote 判定、明文拒绝 |
| Stats (connected/disconnected/slow_consumer) | ✓ | broker conformance #2 / reconnect #1 |
| Internal deadline 全覆盖 | ✓ | `pool.rs` + `jetstream.rs` 模块级 timeout |
| Validate fail-closed | ✓ | `jetstream.rs` 单测：零 timeout / 非法名拒绝 |
| package stable | ○ | P1-1：待 soak + 10k 并发 + TLS 真实证据 |
| NKey 认证 | ○ | P1-4：配置 + conformance 补齐 |
| Request-Reply / Headers | ○ | P2-1, P2-2 |
| Graceful drain | ○ | P2-5 |
| TLS 证书进阶（CA/mTLS） | ○ | P2-4 |
| Cluster/HA / KV / ObjectStore | ✗ | NO-GO |

> 对齐审计日期：2026-07-22（v0.3.2）→ 2026-07-23（v0.3.3 加固）。
> conformance 来源：`crates/adapters/storage/nats/tests/{broker_conformance,reconnect_conformance,live_event_bus}.rs`
