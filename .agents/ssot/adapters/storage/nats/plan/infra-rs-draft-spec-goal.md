# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/natsx_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# natsx 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/nats` → `natsx`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

将内存 `NatsAdapter/MockNatsBus` 替换为基于 `async-nats` 的生产级异步库。目标覆盖 Core NATS 的低延迟 Pub/Sub 与 Request/Reply，以及作为明确扩展的 JetStream 可靠消息；提供 `NatsPool` 资源对象、自动重连、订阅生命周期、背压、TLS/NKey/JWT/creds、可观测性和故障测试。

### 边界

- `contracts::EventBus` 映射到 Core NATS 时只能承诺 at-most-once。
- at-least-once、durable、ack/nack、重投递、consumer state 必须使用 JetStream 扩展 API。
- 不把 Core NATS 的“客户端可克隆”误写成多物理连接池；按隔离与吞吐需求配置少量 lane。
- 不在稳定 API 泄露底层 crate 的全部配置和类型。

### 验收目标

10,000 并发 publish future 有界；慢订阅者不能导致无界内存；断线/集群节点滚动重启可恢复；订阅 drop 无任务泄漏；24h soak 稳定；所有交付声明均有真实 NATS 集群测试。

## 2. SPEC

### 2.1 当前差距

现实现为进程内 HashMap 快照回放，不是实时订阅，无 NATS I/O、reconnect、slow consumer、queue group、headers、JetStream、认证和观测。

### 2.2 feature

`default=[runtime-tokio,tls-rustls]`；可选 `jetstream`、`nkeys`、`websocket`、`metrics`、`test-util`、`scaffold`。同步 `nats` crate 已弃用，生产实现必须使用经审计的 `async-nats` 精确版本。

### 2.3 公共 API

```rust
pub struct NatsConfig;
pub struct NatsConfigBuilder;
#[derive(Clone)] pub struct NatsPool;
#[derive(Clone)] pub struct NatsClient;
pub struct NatsSubscription;       // Stream<Item=XResult<NatsMessage>>
pub struct NatsMessage { /* subject, reply, headers, payload, sequence? */ }
#[cfg(feature="jetstream")] pub struct JetStreamClient;

impl NatsPool {
    pub async fn connect(config: NatsConfig) -> XResult<Self>;
    pub fn client(&self) -> NatsClient;
    pub async fn subscribe(&self, subject: Subject) -> XResult<NatsSubscription>;
    pub async fn health(&self) -> XResult<NatsHealth>;
    pub fn stats(&self) -> NatsPoolStats;
    pub async fn drain_and_close(&self, deadline: Duration) -> XResult<()>;
}
```

配置必须覆盖 servers、name、TLS、auth provider、connect/request timeout、ping interval、max reconnects、reconnect backoff+jitter、publish/subscribe buffer 上限、lane 数、in-flight 上限、inbox prefix、drain deadline。subject 必须由验证过的 newtype 构造。

### 2.4 池与高并发

`async_nats::Client` 是 cloneable connection handle；`NatsPool` 默认持有一个共享 client/lane，由内部连接负责复用和重连。只有以下理由允许多 lane：流量隔离、不同凭据、独立故障域或基准证明单 lane 瓶颈。Pool 负责：

- lane 生命周期和 readiness；
- 全局 publish semaphore 与有界缓冲；
- subscription registry、取消和 drain；
- 重连事件、凭据轮换与统计；
- Core 与 JetStream 资源隔离。

排队时间计入 deadline；不得在重连期间无界积压 publish。离线缓冲策略必须可配置为 fail-fast 或有界等待，默认有界等待。

### 2.5 Core NATS 语义

- `EventBus::publish` 在客户端接受/flush 策略完成后返回；文档必须说明这不等价于 durable persistence。
- `EventBus::subscribe` 返回实时订阅，不回放历史；`BusMessage.id` 使用订阅会话内单调 ID，不能跨重连去重。
- 生产扩展 `NatsSubscription` 的 item 必须携带错误，使断线/slow consumer 可见。
- 支持 queue subscribe、headers、request/reply、flush；request 使用唯一 inbox 与总 deadline。
- Core NATS 不自动重试业务 publish，避免断线边界重复且不可判定。

### 2.6 JetStream 语义

- stream/consumer 配置采用强类型并支持“验证现存配置”与“显式 reconcile”，禁止启动时静默破坏性修改。
- durable consumer 默认 explicit ack；`AckHandle` 单次终结，支持 ack/nak/term/in_progress。
- ack wait、max deliver、backoff、max ack pending、filter subjects 必须显式。
- consumer stream 使用有界并发；只有业务完成后 ack。超过重投次数由策略决定 advisories/DLQ，库不静默吞弃。
- publish 可启用 message-id 去重；成功返回 stream sequence。所谓 exactly-once 只能在 NATS 官方能力和测试覆盖范围内描述。

### 2.7 取消、重连、关停

- reconnect callback 不能阻塞网络任务；事件写入有界 channel。
- subscription drop 必须 unsubscribe/cancel；后台 task 有 owner 并可 join。
- drain：拒绝新请求 → flush publish → drain subscriptions → 等待 callbacks → close；deadline 超时返回错误并强制取消剩余本地任务。
- 关停期间错误为 `Cancelled`；服务器全部不可达为 `Unavailable`；请求/排队超时为 `DeadlineExceeded`。

### 2.8 错误映射

非法 subject/config → `Invalid`；stream/consumer 不存在 → `Missing`；配置或 expected sequence 冲突 → `Conflict`；可恢复 I/O/leader 变化 → `Transient`；无 server/认证依赖失败 → `Unavailable`；取消 → `Cancelled`；超时 → `DeadlineExceeded`；状态机破坏 → `Invariant`；未知驱动错误 → `Internal` 并计数。

### 2.9 安全

生产默认 TLS 验证；支持 user/pass、token、NKey、JWT/creds，但每实例只允许一个明确 auth 策略。seed/token/creds 不得 Debug/日志/指标；签名 callback 使用受控 secret provider。account/subject permissions 最小化；管理 JetStream 与数据面凭据隔离。

### 2.10 可观测性与健康

指标：publish/receive/request/ack、延迟、in-flight/buffer、slow consumer、dropped、reconnect/disconnect、server changes、subscription 数、JetStream pending/redelivery。标签限制为 logical cluster、operation、outcome、subject allowlist；禁止动态 inbox、payload、凭据。

liveness 检查内部事件循环；readiness 要求至少一 lane connected 且有 deadline 的 flush/ping 成功；JetStream readiness 另验证目标 stream/consumer。

### 2.11 测试

- 单元：subject、脱敏、错误、deadline、lane 选择、drain 状态机、AckHandle 单次终结。
- 合同：EventBus at-most-once 能力面。
- 集成：3 节点 NATS、TLS/NKey/JWT、queue group、request/reply、JetStream durable。
- 故障：节点滚动、leader 切换、网络分区、慢消费者、权限撤销、凭据轮换、磁盘/存储限制。
- 并发与取消：10k publish、1k subscriptions、drop/reconnect race；loom 检查自研状态。
- 基准/soak：100B/1KiB/1MiB，吞吐、p99、CPU、RSS、任务/FD；24h 无持续增长。

### 2.12 里程碑与 DoD

P0 Core publish/subscribe + Pool + TLS/telemetry；P1 queue/request/reconnect/drain；P2 JetStream publish/consumer/ack；P3 运维与性能加固。

完成条件：生产默认不导出 scaffold；公开 API 文档、示例和迁移指南完整；真实集群/故障/soak/安全/SBOM/许可证/基准门禁全部通过；每项可靠性声明可追溯到测试证据。

## 3. 参考依据

- [infra.rs](https://github.com/xhyperium/infra.rs)
- [`async-nats` 官方异步客户端](https://docs.rs/async-nats/latest/async_nats/)
- [`Client` 是可克隆连接句柄](https://docs.rs/async-nats/latest/async_nats/client/struct.Client.html)

