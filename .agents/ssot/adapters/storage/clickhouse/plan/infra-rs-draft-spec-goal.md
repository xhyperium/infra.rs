# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/clickhousex_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# clickhousex 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/clickhouse` → `clickhousex`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

把内存 `ClickHouseAdapter` 升级为基于官方纯 Rust `clickhouse` 客户端的生产级异步分析库：共享 HTTP 连接资源池、批量/流式 insert、类型与 schema 验证、流式 select、并发/内存背压、压缩、TLS、复制表写入语义、可观测性和集群故障测试。

### 边界

- 实现 `contracts::AnalyticsSink`，但其 event+Bytes 只作为受控 envelope 入口。
- 完整 API 应使用强类型 Row/Codec；禁止默认把不透明 Bytes 直接拼 SQL。
- ClickHouse 适合 OLAP/append-heavy，不提供 OLTP 事务语义。
- 不将 HTTP client 的内部复用伪装成一请求一连接 checkout；Pool 是共享 client + 并发/批处理资源管理器。

### 验收目标

高并发写入有界且批次可控；单次大查询以 stream 消费不 OOM；schema 不匹配启动/首请求快速失败；节点滚动/短时 5xx 可恢复；24h soak 无 batch/task/connection 泄漏；固定数据集输出 rows/s、bytes/s、p99、CPU/RSS。

## 2. SPEC

### 2.1 当前差距

当前 `sink` 只把 `(event, Bytes)` push 到 Mutex Vec，无 ClickHouse I/O、schema、batch、查询、压缩、池、超时、安全、故障和观测。

### 2.2 feature

`default=[runtime-tokio,tls-rustls,compression-lz4]`；可选 `compression-zstd`、`inserter`、`opentelemetry`、`metrics`、`test-util`、`scaffold`。底层官方 `clickhouse` crate 精确锁定；RowBinary validation 的性能/安全选择成为显式配置并有基准证据。

### 2.3 API

```rust
pub struct ClickHouseConfig;
pub struct ClickHouseConfigBuilder;
#[derive(Clone)] pub struct ClickHousePool;
#[derive(Clone)] pub struct ClickHouseClient;
pub struct InsertSink<R>;               // write/flush/end/abort
pub struct RowStream<R>;                // Stream<Item=XResult<R>>
pub trait AnalyticsRow: clickhouse::Row + Serialize + Send + Sync { const TABLE: &'static str; }

impl ClickHousePool {
    pub async fn connect(config: ClickHouseConfig) -> XResult<Self>;
    pub fn client(&self) -> ClickHouseClient;
    pub async fn inserter<R: AnalyticsRow>(&self, opts: InsertOptions) -> XResult<InsertSink<R>>;
    pub async fn query<R>(&self, query: QuerySpec) -> XResult<RowStream<R>>;
    pub async fn health(&self) -> XResult<ClickHouseHealth>;
    pub fn stats(&self) -> ClickHousePoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}
```

配置包括 URL 列表/logical cluster、database/user/secret、TLS、connect/request/acquire timeout、max in-flight、HTTP keepalive/pool、compression、insert rows/bytes/period、queue bytes、query max rows/bytes/execution time、validation、replica/failover policy、quota key（受控）。

### 2.4 Pool 与路由

官方 client/HTTP stack 负责连接复用；`ClickHousePool` 在启动构造并共享。外围包含每 endpoint client、健康状态、semaphore、batch registry、buffer-byte budget、路由与关停。禁止每条 row 创建 client/HTTP connection。

多 endpoint 路由必须明确：写入通常指向 Distributed table/负载均衡入口；客户端轮询只有在部署拓扑和重复写风险经过验证时启用。失败后换 endpoint 重试写入可能重复，默认禁止不透明重试。

### 2.5 AnalyticsSink 合同

- `event` 必须通过静态 registry 映射到 table+codec，未知 event → `Invalid`。
- `payload` 由对应 versioned codec 解码，禁止拼接 SQL；解码失败 → `Invalid`。
- `sink` 成功仅在该 row 被远端 insert 确认后返回。为吞吐而异步缓冲的 API 必须另命名 `enqueue`，返回 receipt，并提供 `flush/await_committed`；不得改变合同成功语义。
- 生产默认强类型 row API；envelope schema 包含 event type/version/id/timestamp/payload。

### 2.6 Insert

- batch 同时受 rows、bytes、period 限制；全局 queue bytes 有硬上限。
- `InsertSink::end` 才确认最后 batch；drop 未 end 必须 abort/记录 dropped rows，不能宣称成功。
- schema validation 默认开启；高性能 RowBinary 无验证模式需显式 opt-in 和部署前 schema checksum。
- async insert/`wait_for_async_insert` 等 server setting 只能通过强类型 options；成功边界必须文档化。
- 去重依赖表引擎、insert dedup token 或业务 event id；库不得一般化声称 exactly-once。
- retry 只在响应明确“未接收”或 dedup token 有效时；响应丢失属于结果不确定，返回可识别错误。

### 2.7 Query

select 返回 row stream；有 row/byte/execution limits，慢消费者采用有界 buffer。stream drop 必须取消/释放 HTTP body。查询模板静态注册，值参数化；动态表/列仅 allowlist。禁止将原始 SQL 作为默认稳定 API；可提供 `unsafe_raw_sql` 命名的显式 escape hatch（不是 Rust unsafe），并限制生产使用。

### 2.8 错误

非法 event/payload/query/config → `Invalid`；table/database 不存在 → `Missing`；schema/type/dedup version 冲突 → `Conflict`；限流/可恢复 5xx/leader/network → `Transient`；所有 endpoint/认证依赖不可用 → `Unavailable`；取消 → `Cancelled`；排队/query/insert 超时 → `DeadlineExceeded`；batch 计数/schema checksum 不变量 → `Invariant`；未知 server code → `Internal`。维护 ClickHouse code→kind 的固定测试表，禁止字符串匹配。

### 2.9 安全

TLS 默认校验；secret 不 Debug；runtime role 只授予目标表 SELECT/INSERT，DDL role 隔离。query settings allowlist，禁止用户绕过资源限制。payload/SQL/credentials 不进日志。按 tenant 使用 server quota/row policy 时，tenant 标签必须低基数并防伪造。

### 2.10 可观测性

指标：insert rows/bytes/batches/latency/errors、queue rows/bytes/age、flush/drop/uncertain result、query rows/bytes/latency/cancel、in-flight/waiters、endpoint health/retry、compression ratio、schema mismatch。标签只含 logical cluster、operation/event allowlist、outcome；禁止 raw SQL/table 动态值/payload。

readiness 执行 `SELECT 1` 并验证目标表/schema checksum；liveness 看本地任务；diagnostics 脱敏 endpoint 与 pool/batcher 状态。

### 2.11 测试

codec/schema/配置/错误/批处理状态单元；AnalyticsSink 合同；真实单节点与复制/分布式 ClickHouse 固定版本；TLS/压缩/RowBinary validation；批次边界、partial/unknown response、节点滚动、网络抖动、readonly/配额、schema change；大查询取消与慢消费者；1/32/256 并发、不同 row/batch/压缩基准；24h ingest/query soak。验证已确认 rows 与 server count/checksum 一致。

### 2.12 里程碑与 DoD

P0 shared Pool + typed insert + TLS/error/telemetry；P1 batcher/stream query/schema validation；P2 replicated/distributed 故障、去重策略、soak；P3 高级 inserter/OTel 可选。

DoD：生产默认无 scaffold；sink 成功边界准确；所有缓存有字节上限；真实集群/故障/soak/性能/安全/SBOM/许可证门禁通过；API、schema 迁移、部署拓扑、运维与回滚文档完整。

## 3. 参考依据

- [infra.rs](https://github.com/xhyperium/infra.rs)
- [ClickHouse 官方纯 Rust typed client：select/insert/inserter、RowBinary 与校验](https://docs.rs/clickhouse/latest/clickhouse/)

