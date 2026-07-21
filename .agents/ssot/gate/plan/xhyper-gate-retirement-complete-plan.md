# `gate` 完整退役与替换执行方案

```text
Plan ID:        PLAN-GATE-RETIRE-001
Title:          Retire Runtime Gate and Replace with Typed Composition
Repository:     xhyperium/infra.rs
Target Package: gate
Current Path:   crates/gate
Decision:       Retire and delete
Replacement:    bootstrap + typed AppContext / bounded contexts
Status:         Implemented（2026-07-15 · A12 物理删除完成；防回流 no-new-gate）
Risk Level:     Architecture / Breaking API
Method:         Strangler Fig + small-batch migration
```

> **执行计划包**（agent-team 可执行台账）：[`.agents/ssot/infra/gate/plan/`](.)  
> 10x 完备性：[`plan/gate-plan-10x-verdict.md`](./gate-plan-10x-verdict.md) · `fail_rounds=0`（**≠** 退役 DONE）  
> 工作台账：[`.worktree/gate-todo.md`](../../../.worktree/gate-todo.md)  
> 对齐：[docs/audits/gate-plan-alignment-2026-07-15.md](../../../../../docs/audits/gate-plan-alignment-2026-07-15.md)  
> 人审：[approval-packet.md](./approval-packet.md) · RFC/ADR-016 **Accepted** · A12 Keep-OPEN

---

## 0. 最终裁定

当前 `gate` 不应继续补齐为生产模块，也不应简单改名或移动目录。

正确处理方式是：

```text
冻结新增使用
→ 批准退役 ADR
→ 在 bootstrap 建立强类型 AppContext
→ 迁移所有消费者
→ 封禁新依赖和字符串能力查找
→ 从 AppContext 移除 Gate
→ 删除 gate crate
→ 清理活动规范、架构登记和 workspace
→ 保留历史 ADR/Evidence
```

只删除：

```text
crates/gate
package: gate
runtime Gate / Capability / register / resolve
```

必须保留：

```text
.agent/gates/
tools/archgate/
CI gate jobs
release gates
xlibgate / policy gate 概念
```

“运行时 gate crate”与“CI/架构门禁 gate”是两个不同概念。退役前者，不得误删后者。

---

# 1. 为什么必须退役，而不是继续优化

## 1.1 当前对象不是有效能力接口

当前模型：

```rust
pub trait Capability: Send + Sync {
    fn name(&self) -> &str;
}

pub struct Gate {
    capabilities: RwLock<HashMap<String, Arc<dyn Capability>>>,
}
```

`resolve()` 返回 `Arc<dyn Capability>`，但 `Capability` 除了 `name()` 没有任何业务方法。

因此调用方获取对象后只能再次读取名字，无法调用：

```text
MarketDataSource
ExecutionVenue
KeyValueStore
EvidenceAppender
Instrumentation
Clock
```

它本质上不是“能力发现”，而是：

```text
字符串名称存在性表
```

## 1.2 它把编译期依赖退化为运行时字符串

当前系统是静态编译的 Rust workspace。

已知依赖应该由：

```text
Cargo dependency
+ contracts trait
+ typed struct field
+ bootstrap construction
```

表达。

把这些依赖改为：

```rust
gate.resolve("market-data")
```

会失去：

```text
- 编译期完整性；
- 类型检查；
- IDE 重构；
- 缺失依赖的构建期反馈；
- 明确依赖图；
- API 兼容检查；
- 可追踪消费者关系。
```

## 1.3 当前注册不是事务性的

当前顺序：

```text
1. 写入 HashMap；
2. 释放写锁；
3. 写 evidence；
4. evidence 失败则 register 返回 Err。
```

结果可能是：

```text
调用方看到 Err
但 capability 实际已经可见
```

这破坏：

```text
返回值语义
状态原子性
审计一致性
重试安全
```

## 1.4 默认 bootstrap 绕过 evidence

`Gate::new()` 使用：

```text
evidence = None
```

而 `Bootstrap::new()` 默认调用 `Gate::new()`。

所以默认组装路径中：

```text
register_capability
```

不会产生 evidence。

当前文档中“注册事件不得绕过 evidence”与真实默认路径不一致。

## 1.5 build 后仍可修改 registry

`AppContext::gate()` 返回：

```rust
&Gate
```

而：

```rust
Gate::register(&self, ...)
```

只要求共享引用。

因此 build 完成后，任意持有 `AppContext` 的调用方仍可注册新能力。

这意味着：

```text
Bootstrap::build()
```

没有真正冻结组合结果。

## 1.6 错误语义不正确

当前行为：

```text
重复注册 → Invalid
能力缺失 → not_found → Invalid
时钟失败 → Invalid
```

正确分类应该分别是：

```text
重复注册 → Conflict
能力缺失 → Missing
时钟失败 → Unavailable
```

继续维护 gate 会迫使它跟随 kernel 错误模型大规模改造，但该模块本身没有值得保留的运行时价值。

## 1.7 物理位置与逻辑分层冲突

当前：

```text
path  = crates/gate
layer = L0
```

目录表达 L1 Infra，文档却表达 L0。

这不是“移动到 crates/gate”即可解决的问题，因为核心问题是它不属于 L0。

## 1.8 与 bootstrap 职责重复

架构已经裁定：

```text
bootstrap 是唯一组合根和依赖注入层
```

但 gate 同时承担：

```text
注册
发现
运行时对象保存
```

导致两个组合中心：

```text
bootstrap
gate
```

系统必须只保留一个组合根：

```text
bootstrap
```

---

# 2. 目标架构

## 2.1 分层终态

```text
L0
├── kernel
└── evidence core

Types
└── canonical / decimalx

Contracts
└── typed ports / traits

L1 Infra
├── configx
├── observex
├── resiliencx
├── schedulex
├── transportx
└── bootstrap

Adapters
├── storage/*
├── exchange/*
└── evidence/*

Services / Apps
└── consume typed contexts
```

终态不存在：

```text
runtime generic registry
string capability lookup
Gate
Capability name-only trait
```

## 2.2 组合原则

```text
contracts 定义“能做什么”
adapters 定义“谁来实现”
bootstrap 决定“本次进程使用哪个实现”
AppContext 只读暴露“已经组装完成的依赖”
```

## 2.3 不允许 AppContext 变成新 Service Locator

错误替代：

```rust
pub struct AppContext {
    values: HashMap<TypeId, Box<dyn Any>>,
}
```

这只是把字符串 Service Locator 改成 TypeId Service Locator，仍然拒绝。

正确替代：

```rust
pub struct AppContext {
    platform: PlatformContext,
    // 后续按真实需求添加 bounded context
}
```

每个字段都有确定类型和职责。

---

# 3. 目标 API

## 3.1 PlatformContext

第一阶段只放已经被 bootstrap 真实拥有的横切能力：

```rust
pub struct PlatformContext {
    instrumentation: Arc<dyn Instrumentation>,
    shutdown_signal: ShutdownSignal,
}
```

提供只读访问：

```rust
impl PlatformContext {
    pub fn instrumentation(&self) -> &dyn Instrumentation;
    pub fn shutdown_signal(&self) -> &ShutdownSignal;
}
```

在新版 evidence 系统落地后追加：

```rust
evidence: Arc<dyn EvidenceAppender>
```

不要把旧 `EvidenceSink` 接入新结构。

## 3.2 AppContext

```rust
pub struct AppContext {
    platform: PlatformContext,
}

impl AppContext {
    pub fn platform(&self) -> &PlatformContext;
}
```

为兼容当前调用方，可保留窄访问器：

```rust
pub fn instrumentation(&self) -> &dyn Instrumentation;
pub fn shutdown_signal(&self) -> &ShutdownSignal;
```

但禁止：

```text
get(name)
resolve(name)
register(...)
insert(...)
Any/downcast
HashMap registry
```

## 3.3 BootstrappedApp

当前 bootstrap 要求调用方在 build 前手工 `take_shutdown_guard()`，容易误丢 trigger owner。

目标：

```rust
pub struct BootstrappedApp {
    context: AppContext,
    shutdown: ShutdownController,
}
```

```rust
pub struct ShutdownController {
    guard: Option<ShutdownGuard>,
}
```

API：

```rust
impl BootstrappedApp {
    pub fn context(&self) -> &AppContext;
    pub fn into_parts(
        self,
    ) -> (AppContext, ShutdownController);
}

impl ShutdownController {
    pub fn trigger(mut self);
}
```

`trigger()`：

```text
take guard
→ guard.trigger()
```

不暴露可复制 guard。

## 3.4 BootstrapBuilder

推荐将 `Bootstrap` 明确命名为 builder：

```rust
pub struct BootstrapBuilder {
    instrumentation: Option<Arc<dyn Instrumentation>>,
    shutdown_guard: ShutdownGuard,
    shutdown_signal: ShutdownSignal,
}
```

构造：

```rust
impl BootstrapBuilder {
    pub fn new() -> Self;

    pub fn with_instrumentation(
        mut self,
        instrumentation: Arc<dyn Instrumentation>,
    ) -> Self;

    pub fn build(
        self,
    ) -> Result<BootstrappedApp, BootstrapError>;
}
```

默认值可以由 `new()` 填入：

```text
TracingInstrumentation
System-level shutdown pair
```

随着系统增长，可增加强类型必需依赖：

```rust
with_evidence(...)
with_config(...)
with_market_data(...)
with_storage(...)
```

但每个方法必须对应真实、稳定的 contract，不得建立通用 register API。

## 3.5 BootstrapError

目标错误：

```rust
#[non_exhaustive]
pub enum BootstrapError {
    MissingDependency {
        name: &'static str,
    },
    InvalidConfiguration {
        name: &'static str,
    },
    DependencyUnavailable {
        name: &'static str,
        source: BoxError,
    },
}
```

映射：

```text
MissingDependency      → XError::Missing
InvalidConfiguration   → XError::Invalid
DependencyUnavailable  → XError::Unavailable
```

第一阶段如果 kernel 新错误 API 尚未落地，可暂时使用 `XResult`，但必须登记迁移任务。

---

# 4. Bounded Context 设计

不要一次把所有 adapter 放进一个全局 AppContext。

按服务需要建立有界上下文。

## 4.1 MarketDataContext

未来需要时：

```rust
pub struct MarketDataContext {
    source: Arc<dyn MarketDataSource>,
    catalog: Arc<dyn InstrumentCatalog>,
    storage: Arc<dyn MarketDataStore>,
    evidence: Arc<dyn EvidenceAppender>,
    instrumentation: Arc<dyn Instrumentation>,
    shutdown: ShutdownSignal,
}
```

## 4.2 ExecutionContext

```rust
pub struct ExecutionContext {
    venue: Arc<dyn ExecutionVenue>,
    account: Arc<dyn AccountSource>,
    risk: Arc<dyn RiskDecisionPort>,
    evidence: Arc<dyn EvidenceAppender>,
    instrumentation: Arc<dyn Instrumentation>,
    shutdown: ShutdownSignal,
}
```

## 4.3 原则

```text
- 服务只拿到它需要的上下文；
- 不把整个 AppContext 传遍所有层；
- context 不提供动态注册；
- context 字段在 build 后不可替换；
- trait 位于 contracts 或明确的低层契约 crate；
- adapter 具体类型只在 bootstrap 可见。
```

---

# 5. 退役范围

## 5.1 删除对象

```text
crates/gate/
├── Cargo.toml
├── src/lib.rs
├── README.md
├── AGENTS.md
└── CHANGELOG.md

.agents/ssot/infra/gate/       # 从 active specs 移出
gate package workspace member
bootstrap → gate dependency
gate::Capability
gate::Gate
Gate::new
Gate::with_evidence
Gate::with_evidence_and_clock
Gate::register
Gate::resolve
Bootstrap::register_capability
AppContext::gate
gate mock feature
```

## 5.2 不删除对象

```text
.agent/gates/
tools/archgate/
docs 中关于 CI gate 的内容
workflow job: gate 或 policy gate
release gate
architecture gate
quality gate
evidence gate
```

建议后续把 CI job `gate` 改名为：

```text
policy-gates
```

以减少命名歧义，但这不是 runtime gate 删除的阻塞项。

---

# 6. 治理准备

## 6.1 RFC

由于这是：

```text
- L0 成员删除；
- bootstrap public API 变化；
- workspace member 删除；
- 依赖法修改；
```

必须先建立 RFC。

建议：

```text
RFC-XXX: Retire Runtime Gate Service Locator
```

RFC 必须回答：

```text
- 为什么不是移动目录；
- 为什么不是 TypeId registry；
- 为什么不支持动态插件；
- 当前消费者清单；
- typed context 目标；
- 迁移兼容策略；
- 删除门槛；
- 回滚路径。
```

## 6.2 ADR

RFC 批准后建立：

```text
ADR-XXX: Bootstrap Is the Sole Composition Root
```

决策：

```text
1. kernel 只保留 error / clock / lifecycle；
2. gate 退出 L0；
3. bootstrap 是唯一组合根；
4. 所有运行时依赖通过 typed fields 暴露；
5. 禁止字符串或 TypeId Service Locator；
6. 动态插件需求必须重新 RFC；
7. active spec 不再包含 gate。
```

## 6.3 Issue 分解

建议建立：

```text
GATE-RETIRE-00  Freeze and inventory
GATE-RETIRE-01  Add typed bootstrap context
GATE-RETIRE-02  Migrate bootstrap tests and consumers
GATE-RETIRE-03  Remove runtime gate API
GATE-RETIRE-04  Delete gate crate and workspace member
GATE-RETIRE-05  Update architecture SSOT and guards
GATE-RETIRE-06  Post-delete verification
```

每个 Issue 对应一个 Goal Runtime v3.1 执行单元。

---

# 7. Phase 0：冻结与完整盘点

## 7.1 冻结规则

在任何重构前先增加临时门禁：

```text
- 禁止新增 gate dependency；
- 禁止新增 `use gate::`；
- 禁止新增 `Gate::`；
- 禁止新增 `Capability` 实现；
- 禁止新增 register_capability；
- 禁止新增字符串 resolve。
```

临时扫描：

```bash
rg -n \
  'use gate::|gate::|Gate::|impl Capability|register_capability|\.gate\(\)|resolve\("' \
  --glob '*.rs' \
  .
```

## 7.2 盘点清单

必须从 `cargo metadata` 获取真实依赖者：

```bash
cargo metadata \
  --format-version 1 \
  --no-deps
```

再执行反向依赖：

```bash
cargo tree -i gate
```

盘点：

```text
production dependencies
dev dependencies
tests
examples
benches
docs
specs
architecture registry
CI scripts
code generators
Cargo.lock
downstream external repositories
```

## 7.3 当前已知面

当前仓库内已知：

```text
bootstrap 生产依赖 gate
bootstrap src 使用 Gate / Capability
bootstrap e2e test 使用 Capability 和 ctx.gate()
root Cargo workspace 包含 crates/gate
architecture spec 把 gate 标为 L0
```

当前没有发现真实 service 通过 gate 获取 Binance、Redis 等能力；这些测试已直接使用 contracts trait。

## 7.4 下游检查

如果该私有 workspace 有外部消费者，必须搜索：

```text
package = "gate"
use gate::
Gate
Capability
register_capability
```

没有完成下游搜索前，不允许直接删除 crate。

## 7.5 Phase 0 Exit Gate

```text
[ ] consumer inventory 完成
[ ] cargo tree 结果存证
[ ] source search 结果存证
[ ] external downstream 检查完成
[ ] no-new-gate guard 已启用
[ ] RFC 已起草
```

---

# 8. Phase 1：建立强类型替代

## 8.1 原则

这一阶段：

```text
新增替代
但不删除旧 gate
```

保证 main 始终可编译、可回滚。

## 8.2 新增结构

在 `bootstrap` 内新增：

```text
PlatformContext
BootstrappedApp
ShutdownController
BootstrapBuilder 或兼容保留 Bootstrap 名称
```

## 8.3 保持旧接口短期可用

临时保留：

```rust
#[deprecated(
    note = "runtime Gate is being retired; use typed AppContext"
)]
pub fn register_capability(...);
```

以及：

```rust
#[deprecated(
    note = "runtime Gate is being retired; use typed accessors"
)]
pub fn gate(&self) -> &Gate;
```

但只在确有下游消费者时保留。

若完整盘点确认只有仓库内测试使用，则不建立复杂兼容层，直接在下一阶段迁移测试。

## 8.4 禁止设计新的通用容器

本阶段禁止新增：

```text
HashMap<String, ...>
HashMap<TypeId, ...>
Box<dyn Any>
downcast_ref
get<T>()
resolve<T>()
register<T>()
plugin map
```

## 8.5 测试

增加：

```text
- build 得到只读 AppContext；
- instrumentation 可调用；
- shutdown controller 可触发 signal；
- build 后没有 mutation API；
- 缺少必需 typed dependency 时 build 失败；
- 不存在字符串 lookup。
```

## 8.6 Phase 1 Exit Gate

```text
[ ] typed AppContext 已存在
[ ] BootstrappedApp 管理 shutdown owner
[ ] 新 API 有单元测试
[ ] 没有引入新 Service Locator
[ ] 旧 API 未新增调用点
[ ] public API diff 已审阅
```

---

# 9. Phase 2：迁移消费者

## 9.1 Bootstrap 单元测试迁移

删除测试中的：

```text
DummyCap
register_capability
ctx.gate()
Gate::len()
Gate::resolve()
```

替换为：

```text
- typed context build；
- instrumentation accessor；
- shutdown trigger；
- required dependency validation。
```

## 9.2 E2E 测试迁移

删除：

```rust
struct E2ECap;
impl Capability for E2ECap;
```

删除：

```text
register_capability(...)
ctx.gate().resolve(...)
```

保留并强化已有强类型测试：

```text
MockBinanceAdapter
  → MarketDataSource
  → InstrumentCatalog
  → AccountSource
  → VenueTimeSource
  → ExecutionVenue

MockKvStore
  → KeyValueStore
```

这正是目标架构：直接面向 contracts trait，而不是经过 generic registry。

## 9.3 服务迁移模式

若发现其他消费者，逐个迁移：

旧：

```rust
let cap = ctx.gate().resolve("redis")?;
```

新：

```rust
let kv: &dyn KeyValueStore =
    ctx.market_data().kv_store();
```

或通过构造函数直接注入：

```rust
MarketDataService::new(
    Arc<dyn MarketDataSource>,
    Arc<dyn KeyValueStore>,
    Arc<dyn EvidenceAppender>,
)
```

推荐优先构造函数注入，context 只在组合边界使用。

## 9.4 不允许临时 downcast

禁止迁移方案：

```rust
ctx.gate()
    .resolve("redis")?
    .as_any()
    .downcast_ref::<RedisAdapter>()
```

这是架构倒退。

## 9.5 Phase 2 Exit Gate

```text
[ ] `use gate::` 生产调用为 0
[ ] `Gate::` 生产调用为 0
[ ] `Capability` 生产实现为 0
[ ] register_capability 调用为 0
[ ] AppContext::gate 调用为 0
[ ] bootstrap e2e 使用 typed contracts
[ ] 所有消费者测试通过
```

---

# 10. Phase 3：移除旧 API

## 10.1 Bootstrap 代码变化

删除：

```rust
use gate::{Capability, Gate};
```

从 `Bootstrap` 删除：

```rust
gate: Gate
```

删除：

```rust
register_capability
```

从 `AppContext` 删除：

```rust
gate: Gate
```

删除：

```rust
pub fn gate(&self) -> &Gate
```

## 10.2 Bootstrap Cargo.toml

删除：

```toml
gate = { path = "../gate" }
```

不要把 gate 改成 dev-dependency。

## 10.3 兼容层策略

只有在发现外部下游无法同批迁移时，才创建：

```text
crates/compat/gate-compat
```

要求：

```text
- 不属于 L0；
- 不进入 production app；
- 标记 deprecated；
- owner；
- expires；
- downstream 清单；
- 禁止新增消费者；
- 最长一个迁移窗口。
```

兼容层只帮助编译迁移，不能继续作为运行时架构。

## 10.4 Phase 3 Exit Gate

```text
[ ] bootstrap 不依赖 gate
[ ] AppContext 不含 Gate
[ ] public bootstrap API 无 register/resolve
[ ] cargo tree -i gate 不含生产依赖
[ ] compat 使用点为 0 或有带期限清单
```

---

# 11. Phase 4：删除 crate

## 11.1 删除文件

删除整个：

```text
crates/gate/
```

## 11.2 Workspace

根 `Cargo.toml` 删除：

```toml
"crates/gate",
```

运行：

```bash
cargo metadata --format-version 1
cargo check --workspace
```

更新 `Cargo.lock`。

## 11.3 Architecture registry

从：

```text
.architecture/workspace.toml
```

删除 gate unit。

不得把 gate 改标为 archived 继续留在 active registry。

历史通过 Git 保留。

## 11.4 Active specs

将：

```text
.agents/ssot/infra/gate/
```

从 active spec 树移出。

处理方式：

```text
- spec 标头改为 Superseded；
- 在 ADR 中记录历史路径和最终删除 commit；
- 可移动至 docs/archive/specs/gate/；
- 不再被 spec completeness gate 当作 active package。
```

## 11.5 架构总纲

修改 `docs/architecture/spec.md`：

旧：

```text
L0 — kernel · testkit · evidence · gate
```

新：

```text
L0 — kernel · evidence core
Testing — testkit
L1 Infra — ... bootstrap
```

删除：

```text
gate 的 L0 依赖规则
gate 公开接口说明
gate 路径映射
```

加强：

```text
bootstrap 是唯一组合根
禁止 runtime Service Locator
typed contracts only
```

## 11.6 结构生成

运行：

```bash
cargo run -p xtask -- gen-structure
```

提交更新后的：

```text
STRUCTURE.md
相关生成索引
```

## 11.7 CHANGELOG

根 CHANGELOG 和 bootstrap CHANGELOG 记录：

```text
Removed runtime `gate` crate.
Replaced string capability registry with typed bootstrap contexts.
Migration: use contracts traits and typed AppContext accessors.
```

## 11.8 Phase 4 Exit Gate

```text
[ ] crates/gate 不存在
[ ] workspace member 不存在
[ ] cargo metadata 无 package gate
[ ] Cargo.lock 无 workspace gate package
[ ] active architecture registry 无 gate
[ ] active spec 无 gate
[ ] STRUCTURE.md 无 runtime gate
```

---

# 12. Phase 5：防回流门禁

## 12.1 Dependency guard

`archgate` 新增：

```text
ARCH-COMPOSITION-001:
  package name "gate" 作为 runtime workspace member → fail。

ARCH-COMPOSITION-002:
  非 bootstrap 代码出现通用 runtime registry → fail。

ARCH-COMPOSITION-003:
  bootstrap 出现 HashMap<String, Arc<dyn ...>> → fail。

ARCH-COMPOSITION-004:
  bootstrap 出现 HashMap<TypeId, ...> / Any / downcast → fail。

ARCH-COMPOSITION-005:
  AppContext 暴露 mutation/register/insert API → fail。
```

## 12.2 Source guard

新增专用脚本或 xtask rule：

```text
forbidden patterns:
  use gate::
  gate::Gate
  gate::Capability
  register_capability
  AppContext::gate
  resolve("...") on runtime context
```

注意：

```text
不能全局禁止单词 gate
```

因为 `.agent/gates`、archgate、release gate 都是合法概念。

## 12.3 Negative fixtures

必须建立负向 fixture：

```text
fixture 1:
  新增 gate workspace package → gate 必须失败。

fixture 2:
  Bootstrap 使用 HashMap<String, Arc<dyn Any>> → 失败。

fixture 3:
  AppContext 增加 register(&self, ...) → 失败。

fixture 4:
  Service 从 bootstrap 获取具体 adapter 类型 → 失败。

fixture 5:
  服务通过字符串选择依赖 → 失败。
```

没有负向 fixture 的防回流规则不算完成。

## 12.4 API guard

生成 bootstrap public API 快照。

强制：

```text
- 不出现 Gate；
- 不出现 Capability；
- 不出现 register/resolve；
- AppContext 只读；
- concrete adapter 不被 re-export。
```

## 12.5 Phase 5 Exit Gate

```text
[ ] dependency guard
[ ] source guard
[ ] negative fixtures
[ ] public API snapshot
[ ] architecture drift check
[ ] no false positive on CI policy gates
```

---

# 13. 验证矩阵

## 13.1 静态验证

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo run -p xtask -- lint-deps
cargo run -p xtask -- crate-standard --check
cargo run -p archgate -- --json
cargo run -p xtask -- gen-structure --check
cargo machete
cargo deny check
```

## 13.2 聚焦验证

```bash
cargo test -p bootstrap
cargo test -p bootstrap --test e2e
cargo clippy -p bootstrap --all-targets -- -D warnings
```

## 13.3 删除证明

```bash
! cargo metadata \
  --format-version 1 \
  | jq -e '.packages[] | select(.name == "gate")'

! rg -n \
  'use gate::|gate::Gate|gate::Capability|register_capability|\.gate\(\)' \
  --glob '*.rs' \
  .
```

## 13.4 依赖证明

```bash
cargo tree -p bootstrap
cargo tree -i bootstrap
cargo tree -i gate  # 应报告 package 不存在
```

## 13.5 行为验证

```text
- AppContext build 成功；
- instrumentation 可用；
- shutdown 可触发；
- build 后无修改依赖入口；
- Binance contracts trait 可用；
- Redis KeyValueStore trait 可用；
- 缺少必需依赖时 build 明确失败。
```

---

# 14. Evidence 目录

每个阶段生成：

```text
evidence/architecture/gate-retirement/<phase>/
├── manifest.json
├── commit.txt
├── cargo-metadata-before.json
├── cargo-metadata-after.json
├── cargo-tree-before.txt
├── cargo-tree-after.txt
├── consumer-inventory.md
├── source-search-before.txt
├── source-search-after.txt
├── public-api.diff
├── architecture.diff
├── test.log
├── clippy.log
├── archgate.json
├── negative-fixtures.log
├── downstream-impact.md
└── verdict.md
```

最终 verdict 必须证明：

```text
runtime gate removed
CI/architecture gates retained
typed composition active
no consumer remains
no compatibility debt remains
anti-reintroduction guard active
```

---

# 15. 回滚策略

## 15.1 Phase 1–2

替代 API 与旧 API 并存时：

```text
直接 revert 当前 PR
```

不需要运行时 feature flag。

## 15.2 Phase 3

旧 API 已从 bootstrap 移除、crate 尚未删除：

```text
revert bootstrap removal PR
```

## 15.3 Phase 4

crate 已删除：

```text
revert deletion PR
```

禁止通过：

```text
临时重新实现另一个 registry
复制旧 Gate 到 bootstrap
关闭 archgate
添加永久 exception
```

回滚必须恢复已知旧版本，不创建第三种中间架构。

## 15.4 数据回滚

gate 当前没有生产持久化状态，因此没有数据迁移和数据回滚。

若后续发现隐藏状态消费者，立即停止删除阶段并重新评估。

---

# 16. PR 切分

推荐 5 个小批次 PR。

## PR-1：治理冻结

```text
RFC / ADR
consumer inventory
no-new-gate guard
status → retiring
```

禁止行为变化。

## PR-2：Typed Bootstrap

```text
PlatformContext
BootstrappedApp
ShutdownController
typed build tests
```

保留旧 gate API。

## PR-3：Consumer Migration

```text
bootstrap unit tests
bootstrap e2e
all discovered consumers
remove register_capability usage
remove ctx.gate usage
```

## PR-4：API Removal

```text
bootstrap 删除 gate dependency
删除 Gate fields/accessors
删除 deprecated bridge
public API update
```

## PR-5：Physical Deletion and Governance Closure

```text
delete crates/gate
workspace cleanup
registry/spec/docs cleanup
STRUCTURE regenerate
negative fixtures
final evidence
```

每个 PR 必须：

```text
- 独立 worktree；
- 不在 main 开发；
- main 保持 green；
- 有明确 rollback；
- 有 Evidence；
- 不混入 evidence/kernel 其他重构。
```

---

# 17. 1 天、7 天、30 天计划

## 1 天

```text
- 建立 RFC/ADR 草案；
- 完成 cargo metadata 和源码消费者盘点；
- 添加 no-new-gate guard；
- 将 gate status 标记为 retiring；
- 新增最小 PlatformContext / BootstrappedApp；
- 不删除 crate。
```

## 7 天

```text
- typed bootstrap 完成；
- bootstrap 单元和 e2e 全部迁移；
- 所有 runtime gate 调用为 0；
- bootstrap 删除 gate dependency；
- public API 和 architecture checks 通过；
- gate crate 进入待删除状态。
```

## 30 天

该迁移本身不应拖到 30 天。

30 天内应完成后续稳定化：

```text
- 删除 gate crate；
- 完成防回流 negative fixtures；
- 将 evidence 新接口接入 PlatformContext；
- 为真实服务建立最小 bounded contexts；
- 检查 AppContext 是否出现 god-object 趋势；
- 清零 compatibility exception。
```

---

# 18. 衡量指标

```text
runtime_gate_package_count                 = 0
runtime_service_locator_count              = 0
string_capability_lookup_count             = 0
typeid_registry_count                      = 0
gate_runtime_dependents                    = 0
post_build_dependency_mutation_entrypoints = 0
compat_gate_consumers                      = 0
typed_required_dependency_coverage         = 100%
negative_fixture_pass_rate                 = 100%
active_specs_claiming_gate_is_L0            = 0
```

---

# 19. 完成定义

只有全部满足，gate 退役才算 DONE。

## 19.1 消费者闭合

```text
[ ] cargo tree 无 gate 依赖者
[ ] source search 无 gate runtime 使用
[ ] external downstream 已迁移或确认不存在
[ ] compat 消费者为 0
```

## 19.2 架构闭合

```text
[ ] bootstrap 是唯一组合根
[ ] AppContext 强类型且只读
[ ] 无 string registry
[ ] 无 TypeId/Any registry
[ ] 无动态 register/resolve
[ ] service 仅依赖 contracts trait
```

## 19.3 物理闭合

```text
[ ] crates/gate 已删除
[ ] root workspace member 已删除
[ ] Cargo.lock 已更新
[ ] architecture registry 已更新
[ ] active specs 已移除
[ ] STRUCTURE 已更新
```

## 19.4 治理闭合

```text
[ ] RFC Approved
[ ] ADR Approved
[ ] CHANGELOG
[ ] public API diff
[ ] archgate
[ ] dependency guard
[ ] source guard
[ ] negative fixtures
[ ] Evidence
```

## 19.5 语义闭合

```text
[ ] CI / architecture / release gates 未被误删
[ ] gate 名称歧义已在文档解释
[ ] no-new-service-locator policy 生效
[ ] rollback 验证完成
```

---

# 20. 最终推荐路径

执行：

```text
PR-1 冻结
→ PR-2 Typed Bootstrap
→ PR-3 迁移消费者
→ PR-4 删除 API
→ PR-5 删除 crate + 防回流
```

不要执行：

```text
- 将 gate 移到 crates/gate；
- 将字符串 key 改成 TypeId；
- 给 Capability 增加 Any/downcast；
- 添加 sealed/frozen 后继续保留 registry；
- 把 gate 合并进 kernel；
- 把 gate 原样复制进 bootstrap；
- 为尚不存在的动态插件需求预建框架；
- 一次性 Big Bang 删除后再修编译。
```

最终系统应满足：

```text
contracts 表达依赖
adapters 实现依赖
bootstrap 组装依赖
typed context 暴露依赖
compiler 验证依赖
```

而不是：

```text
字符串注册
运行时发现
downcast
隐式全局容器
```

`gate` 的最佳处理不是修到 5/5，而是用可验证的小批次迁移把它安全地降到 0，并用机器门禁保证它永远不会以另一个名字重新出现。
