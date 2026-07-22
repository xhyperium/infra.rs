# infra.rs draft SPEC_GOAL 入库（只读快照）

> **来源**：`.cargo/draft/postgresx_SPEC_GOAL.md`（原 gitignored 草稿；现入库 SSOT）
> **入库原因**：draft 战役合同进入本仓 SSOT，供实现对照；**≠** package stable
> **本仓落地**：见 `docs/ssot/adapters-ssot-alignment.md` 与 PR #188–#191

---

# postgresx 生产级开发库：GOAL / SPEC

> Draft v1.0｜`crates/adapters/storage/postgres` → `postgresx`｜基线 HEAD `6b5f8bb`（2026-07-21）

## 1. GOAL

把内存 `PostgresAdapter` 升级为使用 `tokio-postgres + deadpool-postgres`（或评审通过的等价组合）的完整异步 PostgreSQL 基础库：真实 `PostgresPool`、连接 checkout、事务绑定查询、prepared statement、类型 codec、迁移接口、超时/取消、TLS、读写分离扩展、观测和故障验证。

### 范围

- 兼容 `contracts::{Repository, TxRunner}`，但生产查询优先使用 Postgres 专属 API。
- 强类型 query/execute、事务、批处理/COPY、游标/流式读取、statement cache。
- 池容量、排队、连接回收、连接寿命、健康检查、优雅关闭。
- 不构建 ORM；不解析任意 SQL；迁移执行器与业务查询分层。
- 不承诺跨数据库事务；不默认自动重试事务体。

### SLO

池等待有界；取消不泄漏连接；连接回收前验证可用性；数据库重启后恢复；10k 等待者不造成内存失控；24h soak 无 client/task/FD 泄漏。吞吐/延迟目标由固定 Postgres 配置基准给出并作为回归门禁。

## 2. SPEC

### 2.1 当前差距

当前 rows 位于 `Mutex<HashMap>`；`ScaffoldTxContext` 不绑定 rows，commit/rollback 不影响真实数据。现有 `Repository<Record,String>` 只是形状演示，绝不能迁移为通用生产 repository 而继续隐藏 SQL/schema/codec。

### 2.2 feature

`default=[runtime-tokio,tls-rustls]`；可选 `tls-native`、`migrations`、`copy`、`metrics`、`test-util`、`scaffold`。TLS backend 必须互斥或有明确优先级。依赖精确锁定并通过 MSRV、许可证、漏洞和 feature 图审计。

### 2.3 公共 API

```rust
pub struct PostgresConfig;
pub struct PostgresConfigBuilder;
#[derive(Clone)] pub struct PostgresPool;
pub struct PgConnection;               // checkout guard，drop 归还
pub struct PgTransaction<'c>;          // 借用连接，非 'static
pub struct QueryOptions { pub timeout: Duration, /*...*/ }
pub struct PostgresPoolStats { pub size: usize, pub available: usize, pub waiters: usize }

impl PostgresPool {
    pub async fn connect(config: PostgresConfig) -> XResult<Self>;
    pub async fn acquire(&self) -> XResult<PgConnection>;
    pub async fn acquire_with(&self, deadline: Duration) -> XResult<PgConnection>;
    pub async fn execute(&self, sql: &Statement, params: &[&(dyn ToSql + Sync)]) -> XResult<u64>;
    pub async fn with_transaction<T, F>(&self, options: TxOptions, f: F) -> XResult<T>;
    pub async fn health(&self) -> XResult<PostgresHealth>;
    pub fn stats(&self) -> PostgresPoolStats;
    pub async fn close(&self, deadline: Duration) -> XResult<()>;
}

impl PgConnection {
    pub async fn transaction(&mut self, options: TxOptions) -> XResult<PgTransaction<'_>>;
}
```

禁止公开接受未标注语义的 URL 字符串作为唯一配置。字段至少包括 hosts/ports/db/user/secret provider、TLS、application name、connect/acquire/query timeout、pool min/max、recycle method、max lifetime/idle、statement cache、TCP keepalive、target session attrs。

### 2.4 池规则

- `max_size > 0`；`min_idle <= max_size`；无效配置启动失败。
- acquire 排队必须公平且有界；排队时间计入调用总 deadline。
- 新连接执行 session 初始化（timezone、search_path、statement_timeout 等）并验证结果。
- 归还时若事务未结束、protocol desync、fatal SQLSTATE 或取消状态不明，连接必须丢弃，禁止复用。
- max lifetime 加 jitter，避免同刻连接风暴；预热受并发限制。
- 不允许把 checkout guard clone；连接上的 transaction 独占直到终结。

### 2.5 查询与 SQL 安全

- 值必须参数化；表/列标识符不能作为参数，动态标识符必须 allowlist + 正确 quote。
- `Statement`/query object 应携带稳定 operation name，用于指标；禁止用完整 SQL 作为 metric label。
- 支持 prepared statement cache，但 DDL/schema 变更导致 invalid plan 时最多执行一次受控 reprepare。
- 大结果使用 stream/cursor/COPY，禁止默认 `collect` 无上限。
- 每请求必须设置总 deadline；取消查询后等待 cancel/连接状态确认，无法确认则销毁连接。

### 2.6 事务

- `PgTransaction` 明确状态 Active/Committed/RolledBack/Failed；commit/rollback 只能终结一次。
- 借用式 transaction 必须由调用方持有的 `PgConnection` 创建；pool 不得返回借用临时 checkout 的自引用对象。闭包式 `with_transaction` 在内部持有 checkout，并把事务借用限制在闭包 future 的作用域。
- drop Active transaction 必须触发同步可保证的回滚路径或把连接标记不可复用；不能依赖“后台最终会回滚”而立即归池。
- 支持 isolation、read_only、deferrable；非法组合为 `Invalid`。
- serialization failure/deadlock 可以由显式 `transaction_retry` helper 重试整个闭包；仅当调用方声明幂等，次数/总 deadline 有界。
- savepoint 为扩展 API，name 必须安全生成。

现有 `TxRunner::begin_tx -> Box<dyn TxContext>` 无法让 repository 在同一事务连接上执行 SQL。实现它只提供事务生命周期合同；完整事务业务必须使用 `PgTransaction` 或未来 additive contract，文档不得暗示二者等价。

### 2.7 Repository 设计

`Record` scaffold 保留在 `scaffold/test-util`。生产 repository 采用用户提供 codec/query plan：

```rust
pub trait PgEntity: Sized + Send + Sync {
    type Id: Send + Sync;
    fn find_statement() -> &'static str;
    fn decode(row: &tokio_postgres::Row) -> XResult<Self>;
    // bind_find / bind_save 由受审计 API 提供
}
```

也可以只提供组合原语，让领域 crate 实现 `Repository`。`save` 必须明确 INSERT/UPSERT/UPDATE 及并发控制；推荐支持 version column/expected version，冲突返回 `Conflict`。

### 2.8 错误映射

- SQLSTATE `22/42`、非法配置 → `Invalid`（42 类也可能是部署错误，需细分表）。
- no rows（要求存在的 API）/undefined object → `Missing`。
- unique/FK/check、serialization/version mismatch → `Conflict`（serialization/deadlock 可映射 `Transient`，需固定映射表）。
- connection reset、too many connections、admin shutdown 等可恢复场景 → `Transient/Unavailable`。
- caller cancel → `Cancelled`；acquire/query timeout → `DeadlineExceeded`；状态机破坏 → `Invariant`；未知 → `Internal`。

必须维护 SQLSTATE→ErrorKind 测试表；禁止字符串匹配。

### 2.9 安全与迁移

TLS hostname/CA 验证默认开启；密码不可 Debug；支持外部 secret provider/短期 token。数据库 role 最小权限，migration role 与 runtime role 分离。migration 必须持 advisory lock、有 checksum、禁止自动修改已应用 migration；启动时默认仅验证，不在每个应用副本自动跑 DDL。

### 2.10 可观测性

指标：acquire wait、pool size/idle/in-use/waiters、connect/recycle/discard、query/tx 延迟与 outcome、timeouts/cancels、SQLSTATE class、statement cache。标签只含 logical db、operation、outcome；禁止 SQL、参数、DSN。slow query 记录 operation 与阈值，不记录敏感参数。

readiness：从池获取连接并执行 `SELECT 1`（总 deadline）；liveness 不访问数据库。diagnostics 提供脱敏 host 状态和 pool stats。

### 2.11 测试

单元（配置/SQLSTATE/状态机/脱敏）；合同（Repository/TxRunner 限定语义）；真实 Postgres 多版本/TLS；事务隔离、deadlock、serialization、constraint；重启/failover/连接拒绝/网络延迟/池耗尽；取消 query/transaction/drop race；COPY/大结果内存上限；1/32/256/1024 并发基准与 24h soak。loom 检查自研池外围状态，底层池不重复造轮子。

### 2.12 里程碑与 DoD

P0 Pool/query/TLS/error/telemetry；P1 transaction/statement cache/cancel；P2 repository codec/COPY/migration；P3 failover/soak/performance。

DoD：生产默认无 scaffold；每次 checkout/transaction 都有生命周期测试；真实 DB 矩阵通过；API 文档/示例/迁移与运维手册齐全；安全、SBOM、许可证、SemVer、基准回归和故障演练通过。

## 3. 参考依据

- [infra.rs](https://github.com/xhyperium/infra.rs)
- [`deadpool-postgres`：tokio-postgres 的异步对象池与 statement cache](https://docs.rs/deadpool-postgres/latest/deadpool_postgres/)
