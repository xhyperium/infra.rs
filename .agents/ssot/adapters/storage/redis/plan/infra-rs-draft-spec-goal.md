# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/redisx_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# redisx 生产级开发库：GOAL / SPEC

> 状态：Draft v1.0｜目标仓库：`xhyperium/infra.rs`｜目标 crate：`crates/adapters/storage/redis` → `redisx`｜基线：仓库 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

### 1.1 产品目标

将当前“内存 `RedisAdapter` + 有限 `live` 验证入口”升级为可长期维护的异步 Redis 开发库。生产入口必须提供显式的 `RedisPool`、可克隆 `RedisClient`、独立 Pub/Sub 会话、集群/哨兵能力、超时与背压、结构化错误、可观测性、优雅关停和可复现的真实服务测试。

成功不是“能够连 Redis”，而是：在高并发、连接抖动、节点切换、慢请求、池耗尽和服务关停时行为有界且可诊断。

### 1.2 范围

- 完整实现 `contracts::{KeyValueStore, PubSub}`，严格保持合同语义。
- 提供 Redis 专属扩展：删除、TTL、CAS、批量、pipeline、Lua、分布式锁原语、流式 Pub/Sub、Cluster/Sentinel。
- Tokio 异步；所有公共 I/O 都支持 deadline/cancellation；不得在 async 路径执行阻塞 DNS、文件或锁等待。
- 支持 standalone；Cluster、Sentinel 作为独立 feature/里程碑。
- TLS、ACL、凭据轮换、连接预热、健康/就绪、指标和 tracing。

### 1.3 非目标

- 不承诺跨 Redis 主从/分片的强一致事务。
- 不把 Pub/Sub 描述为可靠消息队列；它天然可能丢消息。
- 不默认提供“锁即正确”的业务保证；锁需 fencing token 才能用于关键写入。
- 不在 `contracts` 中泄露 Redis 类型。

### 1.4 SLO 与容量目标

以下是验收基线，不是对所有部署环境的无条件保证：

| 项目 | 发布门槛 |
|---|---|
| 并发安全 | 10,000 个并发 future 下无 panic、死锁、数据竞争 |
| 池等待 | 必须有上限；耗尽返回 `DeadlineExceeded`/`Unavailable`，不得无限等待 |
| 连接恢复 | 单节点短暂断开后自动恢复；恢复过程有状态与指标 |
| 泄漏 | 24h soak 后连接数、任务数、内存无持续单调增长 |
| 延迟证据 | CI/基准报告 p50/p95/p99；回归阈值由基准环境锁定 |
| 可用性 | readiness 只在至少一个命令通道可用且 `PING` 成功时为 true |

## 2. SPEC（规范性要求）

本文的“必须/禁止/应该/可以”分别对应 MUST/MUST NOT/SHOULD/MAY。

### 2.1 当前差距

- 默认 `RedisAdapter` 是 `std::sync::Mutex<HashMap>`，忽略 TTL；不是 Redis。
- `RedisLiveKv` 仅验证基础 KV；缺少完整池生命周期、Pub/Sub、集群、指标和故障测试。
- 当前 `redis = 0.27` 可以作为迁移起点，但生产实现必须通过依赖评审后精确锁定版本，不以本文硬编码“最新版”。

### 2.2 crate 与 feature

```toml
[features]
default = ["runtime-tokio", "tls-rustls"]
runtime-tokio = []
tls-rustls = []
cluster = []
sentinel = []
pubsub = []
metrics = []
test-util = []       # 禁止进入生产默认依赖图
scaffold = []        # 旧内存实现，仅迁移/测试
```

生产类型不得被 `live` 这种含糊 feature 隐藏；`scaffold` 类型必须改名为 `InMemoryRedis` 或明确 deprecated，防止误用。

### 2.3 公共 API

```rust
pub struct RedisConfig { /* 私有字段，Builder 构造 */ }
pub struct RedisConfigBuilder;
#[derive(Clone)] pub struct RedisPool { /* Arc<Inner> */ }
#[derive(Clone)] pub struct RedisClient { /* 共享复用句柄 */ }
pub struct RedisPubSub { /* 独占连接/后台任务，drop 可取消 */ }
pub struct RedisPoolStats { pub open: usize, pub in_flight: usize, pub waiters: usize }
pub enum RedisMode { Standalone, Cluster, Sentinel }

impl RedisPool {
    pub async fn connect(config: RedisConfig) -> XResult<Self>;
    pub fn client(&self) -> RedisClient;
    pub async fn subscribe(&self, channels: impl IntoIterator<Item=String>) -> XResult<RedisPubSub>;
    pub async fn ping(&self) -> XResult<Duration>;
    pub fn stats(&self) -> RedisPoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}
```

- `RedisPool`、`RedisClient` 必须 `Send + Sync + Clone`，clone 只增加共享引用。
- `RedisPubSub` 必须 `Send`；是否 `Sync` 由底层实现决定，不能为满足签名加全局互斥锁。
- 配置字段必须至少包含：端点列表、mode、用户名/secret provider、database、TLS、connect/command/acquire timeout、最大并发、预热数、重连 backoff、TCP keepalive、client name。
- `Debug`、日志和错误上下文必须脱敏 URL、密码、token、证书私钥。

### 2.4 “连接池”的真实语义

Redis async `MultiplexedConnection` 可跨线程并发复用，`ConnectionManager` 可自动重连；因此禁止简单照搬同步池“一请求一物理连接”。`RedisPool` 是资源池对象，但内部应为：

1. N 个可配置的 multiplexed command lane（默认由压测决定，通常很小）；
2. 每 lane 一个自动重连 manager；
3. 全局/每 lane `Semaphore` 限制 in-flight，形成背压；
4. Pub/Sub 使用专用连接或专用任务，不占普通命令 lane；
5. 阻塞命令（BLPOP/XREAD BLOCK 等）使用独立 blocking lane；
6. Cluster 按 slot 路由并处理 MOVED/ASK，但重定向次数必须有界。

不得把等待 semaphore 的时间排除在总 deadline 之外。命令总耗时 = 排队 + 获取资源 + 网络 + 解码。

### 2.5 合同实现与扩展能力

- `KeyValueStore::get`：不存在返回 `Ok(None)`；二进制安全；空值不是不存在。
- `set`：TTL 为 `Some(0)` 时必须在文档中固定为 Invalid 或立即过期，不得随驱动版本漂移；推荐 Invalid。
- `PubSub::sub_channel` 返回实时 `'static` stream；断连事件不得被静默吞掉。由于合同 stream item 无 `Result`，生产扩展必须另提供 `RedisMessageStream<Item=XResult<BusMessage>>`。
- `BusMessage.id` 对 Redis Pub/Sub 无原生消息 ID，必须使用进程内单调序号并明确“只在当前订阅会话内唯一”；需要可靠 ID 时使用 Redis Streams 扩展。

扩展接口至少包括 `get_bytes/set_bytes/delete/exists/expire/ttl/mget/mset/pipeline/script`。锁扩展必须返回随机 ownership token；释放用 compare-and-delete Lua，续租用 compare-and-expire；关键业务另返回 fencing token。

### 2.6 并发、超时、重试

- 所有 I/O 必须异步；禁止持有 `std::sync::MutexGuard` 跨 `.await`。
- acquire 与 command timeout 独立配置，同时受调用级总 deadline 限制。
- 只对已知幂等操作自动重试；`INCR`、Lua、事务和不透明写入默认不自动重试。
- 退避为指数 + full jitter，次数与总时间有上限；不得在驱动自动重连之外形成重试风暴。
- `close` 后新请求立即 `Cancelled/Unavailable`；已接收请求在 deadline 前排空。

### 2.7 错误映射

| 场景 | `ErrorKind` |
|---|---|
| 非法 key/TTL/配置 | `Invalid` |
| key 不存在（仅要求存在的扩展 API） | `Missing` |
| CAS/锁 owner 不匹配 | `Conflict` |
| LOADING/TRYAGAIN/可恢复 I/O | `Transient` |
| 无可用节点/认证依赖不可用 | `Unavailable` |
| 调用取消/关停 | `Cancelled` |
| 排队或命令超时 | `DeadlineExceeded` |
| slot/状态机不变量破坏 | `Invariant` |
| 未分类驱动错误 | `Internal`（须计数并持续归零） |

### 2.8 安全

- 生产默认 TLS；明文连接需显式 opt-in。
- secret 通过 provider 注入，禁止 serde 序列化、Clone 到非必要对象或输出 Debug。
- ACL 最小权限；不同用途（KV/PubSub/Streams/管理）推荐不同用户。
- Lua 脚本固定内容 + SHA；禁止把不可信输入拼接进命令/脚本。

### 2.9 可观测性

必须暴露低基数指标：请求总数/失败数/延迟、排队延迟、in-flight、waiters、lane 状态、重连、重定向、超时、Pub/Sub lag/断线。标签仅允许 operation、outcome、mode、logical_instance；禁止 key、channel、完整 endpoint。Tracing span 要传播调用上下文但不记录 payload。

健康分三级：`liveness` 只检查内部任务；`readiness` 执行有 deadline 的 PING；`diagnostics` 返回脱敏节点/lane 状态，不用于高频探针。

### 2.10 测试与验证

- 单元：配置、脱敏、错误映射、deadline 预算、重试判定、关闭状态机。
- 合同：`contract-testkit` 的 KV/PubSub suite；TTL 边界与二进制值。
- 集成：真实 standalone、ACL、TLS、Cluster、Sentinel；测试容器固定服务版本。
- 故障：kill/restart、主从切换、网络丢包/延迟、连接拒绝、DNS 变化、池耗尽。
- 并发：loom 检查自研状态机；取消 acquire、取消命令、stream drop 不泄漏任务。
- 基准：1/32/256/1024 并发的 GET/SET/pipeline，1KiB/64KiB/1MiB payload；报告吞吐、p99、CPU、分配和连接数。
- Miri/ASan（适用处）、clippy、rustdoc、`cargo deny`、最小版本/feature 组合均为发布门禁。

### 2.11 交付阶段

1. P0：配置、`RedisPool`、单机 KV、错误/超时/指标、真实集成测试。
2. P1：专用 Pub/Sub、优雅关闭、故障注入、24h soak。
3. P2：pipeline/Lua/锁与 Streams 扩展。
4. P3：Cluster；P4：Sentinel。每阶段独立发布，未完成能力不得出现在稳定承诺中。

### 2.12 Definition of Done

- 默认生产入口无 scaffold；README 有最小、安全、故障处理示例。
- 公共 API 文档完整且 `#![deny(missing_docs)]`。
- 全部测试矩阵、基准阈值、安全审计、依赖许可证门禁通过。
- 有升级/回滚指南、CHANGELOG、SemVer 政策和运行手册。
- 至少一次故障演练证明：断线恢复、池耗尽、关停不丢失已确认完成的写入。

## 3. 参考依据

- 仓库与现状：[xhyperium/infra.rs](https://github.com/xhyperium/infra.rs)
- Redis Rust 客户端说明：[`MultiplexedConnection` 可安全并发复用，`ConnectionManager` 提供自动重连](https://docs.rs/redis/latest/redis/)

