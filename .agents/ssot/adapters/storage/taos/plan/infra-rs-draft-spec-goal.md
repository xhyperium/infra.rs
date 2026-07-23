# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/taosx_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191
>
> **过时声明（2026-07-23）**：§2.1「仅 HashMap scaffold」已过时；当前默认路径为 REST
> 生产客户端 `TaosPool`/`TaosClient`（见 `docs/ssot/taosx-ssot-alignment.md` 与十轮审查）。
> 后半量化交易全底座、Native SQL、HA、package stable 仍为 OUT-OF-SCOPE / NO-GO。

---

# taosx 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/taos` → `taosx`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

将内存 `TaosAdapter` 升级为 TDengine 生产级异步时序库，支持 native 与 WebSocket/REST（以驱动实际能力为准）的明确 feature、真实 `TaosPool`、批量写入、参数绑定、流式查询、超级表/子表模型、无界基数防护、超时/背压、可观测性及集群故障验证。

### 产品边界

- 完整实现 `contracts::TimeSeriesStore`，保持时间范围单位为纳秒 epoch。
- 提供 TDengine 专属 API 表达 database/stable/subtable/tag、批量、schemaless、TMQ（独立 feature）。
- 不让用户输入直接成为未校验标识符；不自动创建无限子表。
- 不宣称 REST/native 在所有语义和性能上等价；分别测试与发布能力矩阵。

### 验收目标

批量写和查询全异步、内存有界；池耗尽有 deadline；节点重启/leader 迁移可恢复；时间精度转换无静默截断；24h soak 无连接/任务/内存增长；固定数据集报告吞吐和 p99。

## 2. SPEC

### 2.1 当前差距

当前实现只是 `HashMap<String, Vec<Tick>>`，范围查询为内存过滤，无 TDengine 连接、池、SQL、schema/tag、批量、精度校验、流式、认证、故障和观测。

### 2.2 feature

`default=[runtime-tokio,ws,tls-rustls]`；可选 `native`、`rest`（若驱动支持且单独验证）、`tmq`、`schemaless`、`metrics`、`test-util`、`scaffold`。native feature 需在 CI 验证动态库/ABI/目标平台；ws 路径不得声称拥有 native 未验证能力。

### 2.3 API

```rust
pub struct TaosConfig;
pub struct TaosConfigBuilder;
#[derive(Clone)] pub struct TaosPool;
pub struct TaosConnection;             // checkout guard
pub struct TaosQueryStream<T>;         // Stream<Item=XResult<T>>
pub struct BatchWriteReport { pub accepted: usize, pub failed: usize }
pub enum TaosTransport { WebSocket, Native, Rest }

impl TaosPool {
    pub async fn connect(config: TaosConfig) -> XResult<Self>;
    pub async fn acquire(&self) -> XResult<TaosConnection>;
    pub async fn write_ticks(&self, target: SeriesTarget, ticks: &[Tick]) -> XResult<BatchWriteReport>;
    pub async fn query_ticks(&self, query: SeriesQuery) -> XResult<TaosQueryStream<Tick>>;
    pub async fn health(&self) -> XResult<TaosHealth>;
    pub fn stats(&self) -> TaosPoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}
```

配置至少包括 DSN/hosts、transport、database、user/secret、TLS、connect/acquire/query timeout、min/max pool、idle/lifetime、write batch rows/bytes/flush interval、query row/byte limits、timezone/precision、max table cardinality。

### 2.4 池

优先复用官方 `taos`/`taos-query` 的 `TaosPool/Pool` 能力，不重复实现底层连接管理。外围 `TaosPool` 统一不同 transport、semaphore、deadline、健康、状态与指标。checkout 失败/超时必须分类；fatal connection 不归池；max lifetime 带 jitter；关闭时拒绝新请求并排空批处理器。

### 2.5 时间与 schema

- 输入 `Tick.ts` 是 i64 纳秒；目标 database precision 必须在启动验证。
- 转为 ms/us 时如不能整除，默认 `Invalid`，除非调用方显式选择 rounding policy。
- query 范围固定 `[start,end]` 以兼容当前合同；专属 API 额外支持 half-open，并在类型中体现。
- symbol→subtable 名称必须经稳定、碰撞可检测的编码；原 symbol 保存为 tag，禁止直接 SQL 拼接。
- stable schema 有版本/checksum；启动默认验证，不自动破坏性变更。

### 2.6 写入

- 优先参数绑定/官方批量 API；标识符由 allowlist/newtype 构造。
- batch 同时受 rows、bytes、flush interval 限制；队列有界，满时背压。
- 部分失败必须返回可定位的 batch report 或整体错误，禁止成功计数与实际提交不一致。
- 自动重试仅在可证明幂等时启用；推荐业务唯一键/时间+tag 去重策略由 schema 明确。
- async batcher 必须有显式 `flush` 与 `close`；drop 不保证远端写入，文档/类型不得暗示保证。

### 2.7 查询

专属查询返回 stream，限制最大行数、扫描时间和 bytes；慢 consumer 通过有界 buffer 背压。取消 stream 必须取消/释放 server query 与连接。禁止 `SELECT *` 作为库生成默认；显式列和解码 schema。`TimeSeriesStore::query_series` 可 collect，但必须受 config 上限，超过返回 `Invalid/DeadlineExceeded` 而非 OOM。

### 2.8 错误

非法表/时间/精度/配置 → `Invalid`；database/stable/table 不存在 → `Missing`；schema/tag/版本冲突 → `Conflict`；可恢复 leader/network/overload → `Transient`；集群/认证依赖不可用 → `Unavailable`；取消 → `Cancelled`；池/查询/写入超时 → `DeadlineExceeded`；状态/计数不变量 → `Invariant`；未知驱动错误 → `Internal`。必须基于驱动 code 分类，禁止字符串匹配。

### 2.9 安全与基数控制

TLS 默认验证；secret 不 Debug；runtime 与 DDL role 分离。database/table/stable 名称强类型验证。必须限制每 logical tenant 的新 subtable 速率/总量并暴露拒绝指标，避免高基数耗尽元数据。SQL 参数化；无法参数化的标识符只能由编码器产生。

### 2.10 可观测性

指标：pool size/in-use/waiters、write rows/bytes/batches/partial failures、flush latency/queue、query rows/bytes/latency/cancel、reconnect、driver error code class、table-create rate/cardinality。标签仅 transport、operation、outcome、logical db；symbol/table 仅 allowlist，默认禁止。

readiness 获取连接并做有 deadline 的轻量 query，且验证 precision/schema；liveness 只看本地任务；diagnostics 脱敏。

### 2.11 测试

时间精度/边界/标识符/codec 单元；TimeSeriesStore 合同；真实 TDengine 固定多版本，ws/native 分矩阵；超级表/tag/batch/partial failure/大查询；节点滚动、leader 变化、网络延迟、池耗尽、磁盘压力；stream cancel/drop、batch close race；1/32/256 并发和不同 batch/payload 基准；24h soak。native 路径加 ABI/打包检查。

### 2.12 里程碑与 DoD

P0 ws Pool + Tick 批量/查询 + telemetry；P1 schema/stable/subtable/stream；P2 native + 故障/soak；P3 TMQ/schemaless 可选。

DoD：默认无 scaffold；每个 transport 有真实证据和能力表；公开文档、示例、schema/精度迁移、运维手册齐全；安全/SBOM/许可证/基准/故障门禁通过；高基数保护和 OOM 防护经过压力验证。

## 3. 参考依据

- [infra.rs](https://github.com/xhyperium/infra.rs)
- [`taos` Rust connector 暴露异步查询与 `TaosPool`](https://docs.rs/taos/latest/taos/)

