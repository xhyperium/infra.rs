# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/kafkax_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# kafkax 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/kafka` → `kafkax`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

把当前内存 `KafkaAdapter/MockKafkaBus` 升级为基于 `rust-rdkafka` 的生产级异步 Kafka 库：共享生产者资源池、隔离的消费者会话、端到端背压、交付确认、消费组再均衡、显式 offset/ack、幂等生产、事务扩展、TLS/SASL、可观测性和故障验证。

### 范围与非目标

- 必须实现现有 `contracts::EventBus`，但明确它只能表达最小 at-most-once 面。
- 必须提供 Kafka 专属可靠消费 API；禁止用无 ack 的 `EventBus` 宣称 at-least-once/exactly-once。
- 支持 Tokio；payload 二进制安全；支持 headers/key/partition/timestamp 的扩展消息。
- 不保证跨 Kafka 与外部数据库的 exactly-once；Kafka EOS 仅覆盖同一事务内的 Kafka records/offsets。
- 不内置 schema registry 作为硬依赖；通过 codec/schema provider 扩展。

### 发布 SLO

| 项目 | 门槛 |
|---|---|
| 并发生产 | 10,000 并发 send future，有界内存、无死锁 |
| 背压 | 本地队列满时等待或明确失败，禁止无界缓存 |
| 交付 | `publish` 仅在 broker delivery report 成功后返回成功 |
| 消费 | rebalance、取消、stream drop 不重复运行后台任务或泄漏消费者 |
| 稳定性 | broker 滚动重启、leader 迁移与短时网络分区可恢复 |
| 证据 | 24h soak + 多分区吞吐/p99/CPU/内存报告 |

## 2. SPEC

### 2.1 当前差距

现实现只在 `Mutex<HashMap>` 中追加并回放消息，无 broker I/O、partition、delivery report、consumer group、offset、rebalance、安全和资源生命周期，不能作为生产实现。

### 2.2 feature 与依赖

```toml
[features]
default = ["runtime-tokio", "tls"]
runtime-tokio = []
tls = []
sasl = []
transactions = []
schema = []
metrics = []
test-util = []
scaffold = []
```

底层首选经过依赖/许可证/构建链评审的 `rust-rdkafka` 精确版本；CI 必须覆盖动态/静态链接策略和目标平台。不得把 librdkafka 的全量配置字符串直接当稳定公共 API；常用配置强类型化，escape hatch 明确标记不稳定。

### 2.3 公共对象

```rust
pub struct KafkaConfig;
pub struct KafkaConfigBuilder;
#[derive(Clone)] pub struct KafkaPool;       // 资源与生命周期容器
#[derive(Clone)] pub struct KafkaProducer;   // 共享 FutureProducer
pub struct KafkaConsumer;                   // 每消费组/会话独占
pub struct KafkaMessage { /* id, topic, partition, offset, key, headers, payload */ }
pub struct Delivery { pub partition: i32, pub offset: i64 }
pub struct ConsumerStream;                   // Item = XResult<ReceivedMessage>
pub struct ReceivedMessage;                  // 含显式 ack/nack/commit handle

impl KafkaPool {
    pub async fn connect(config: KafkaConfig) -> XResult<Self>;
    pub fn producer(&self) -> KafkaProducer;
    pub async fn consumer(&self, cfg: ConsumerConfig) -> XResult<KafkaConsumer>;
    pub async fn health(&self) -> XResult<KafkaHealth>;
    pub fn stats(&self) -> KafkaPoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}
```

配置至少包括 bootstrap servers、client id、security protocol、TLS/SASL secret provider、acks、compression、linger、batch size、message timeout、本地队列上限、最大 in-flight、idempotence、consumer group、auto offset reset、session/max-poll timeout、fetch limits、rebalance drain deadline。

### 2.4 连接池语义

Kafka client 自己维护 broker 连接与元数据；禁止人为创建“每请求 checkout 一个 producer 连接”的伪池。`KafkaPool` 必须是：

- 一个或少量按 QoS/事务 ID 隔离的共享 `FutureProducer`；
- 有界 send permits 与本地队列容量；
- consumer registry 与生命周期/关停管理；
- metadata/admin 健康句柄；
- 统计、凭据刷新和后台回调所有权。

消费者不能在任意请求间池化：每个 `group.id + subscription + assignment strategy` 创建独立会话。事务生产者按 `transactional.id` 独占，不能并发共享一个进行中的事务。

### 2.5 `EventBus` 合同

- `publish(topic,payload)` 必须等待 delivery report；topic 非法为 `Invalid`。
- `BusMessage.id` 固定格式 `topic/partition/offset`，可解析但调用方不得依赖字符串排序。
- `subscribe(topic)` 必须使用明确、稳定的默认消费组策略；推荐禁止生产环境调用无 group 配置的该 facade，并在 README 引导专属 API。
- 因 `BoxStream<Item=BusMessage>` 不能表达流错误和 ack，adapter 只能在成功取出后产出消息；任何错误必须结束 stream 并通过指标/日志暴露。可靠业务必须使用 `ConsumerStream<Item=XResult<ReceivedMessage>>`。

### 2.6 可靠生产

- 默认 `acks=all`；幂等生产默认开启（若 broker/driver 支持并验证）。
- send 的排队、broker ack 和 delivery report 共用总 deadline。
- 本地队列满必须背压；可配置 fail-fast，但不得静默丢弃。
- 自动重试仅由经审计的 librdkafka 机制执行；应用层不能叠加无限重试。
- headers/key 不能记录到低基数指标；payload 永不记录。
- 事务 API 使用 typestate 或运行时状态机，顺序为 init → begin → send → send_offsets → commit/abort；drop 未提交事务必须 abort/标记不可复用。

### 2.7 可靠消费

- 默认关闭 auto-commit 或明确规定其语义；生产扩展使用处理成功后显式 ack/批量 commit。
- ack 必须验证 generation/assignment；过期 handle 返回 `Conflict`，不得提交新一代分区 offset。
- 每 partition 保序；跨 partition 不承诺顺序。
- rebalance revoke：停止拉取 → 有界排空 → 提交已连续 ack 的最高 offset → 释放分区；超时则放弃未完成工作并记录。
- 并发处理采用 per-partition 有界队列；不得因某分区慢而无限占内存。
- poison message、DLQ 是策略层扩展；库提供 hook，不默认吞消息。

### 2.8 错误与重试映射

非法配置/topic → `Invalid`；未知 topic（且禁自动创建）→ `Missing`；generation/事务冲突 → `Conflict`；leader 迁移、可恢复 broker/network → `Transient`；全 broker/认证服务不可用 → `Unavailable`；取消/关停 → `Cancelled`；queue/delivery/max-poll 超时 → `DeadlineExceeded`；状态机破坏 → `Invariant`；其余 → `Internal` 并计数。

生产错误必须保留 source，但展示内容要去除凭据。Kafka error code 应作为受控 diagnostic 字段，不通过字符串判断重试。

### 2.9 安全与配置

- 支持 TLS hostname/CA 校验，禁止默认 accept-invalid-certs。
- SASL/PLAIN、SCRAM、OAuth 等按 feature/平台验证；secret 支持轮换且不 Debug。
- ACL 最小化到 topic/group/transactional-id；admin 能力与数据面客户端隔离。
- 禁止不受控 broker 自动建 topic；生产 topic 的 partition/replication/min ISR 由部署检查验证。

### 2.10 可观测性

指标至少包括 send/receive/ack/commit/rebalance、delivery latency、queue time/size、in-flight、consumer lag（有界采集）、assigned partitions、errors by class/code、retries、timeouts。标签可含 logical cluster、topic（必须 allowlist）、operation、outcome；禁止 key/header/payload/group 动态值。

健康：liveness 检查内部 poll loop；readiness 获取有 deadline 的 metadata 且目标 topic 可用；consumer readiness 还需有效 assignment（启动宽限期除外）。

### 2.11 测试矩阵

- 单元：配置校验、错误映射、ID 编码、连续 ack watermark、事务/关停状态机。
- 合同：EventBus suite，明确 at-most-once 证据。
- 集成：真实多 broker KRaft 集群、TLS/SASL、多个 partition/consumer。
- 故障：leader kill、滚动重启、ISR 缩减、网络抖动、磁盘满模拟、rebalance storm。
- 语义：重复 delivery、乱序 completion、过期 ack、commit 失败、事务 fencing。
- 基准：payload 100B/1KiB/1MiB，1/12/96 partition；吞吐、p99、CPU、RSS、queue depth、lag。
- 24h soak；任务、native allocation、FD、broker connection 不持续增长。

### 2.12 里程碑与 DoD

1. P0：producer + `KafkaPool` + delivery report + TLS + telemetry。
2. P1：consumer group、显式 ack、rebalance 正确性。
3. P2：幂等/事务、故障与 soak；P3：schema/admin 可选扩展。

完成条件：默认 API 无内存 scaffold；公开项文档完整；真实集群矩阵通过；可靠性声明与测试一一对应；安全/许可证/SBOM/基准门禁通过；README、迁移、运维、升级/回滚和兼容策略齐全。

## 3. 参考依据

- [infra.rs 仓库](https://github.com/xhyperium/infra.rs)
- [`rust-rdkafka` 的 `StreamConsumer`/`FutureProducer` 异步模型](https://docs.rs/rdkafka/latest/rdkafka/)

