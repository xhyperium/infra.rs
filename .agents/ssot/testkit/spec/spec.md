# `testkit` 完整规范

```text
Spec ID:          SPEC-TESTKIT-002
Title:            infra.rs Deterministic Test Support
Status:           Stable
Target Status:    Stable
Owner:            platform / quality
Physical Path:    crates/testkit
Package:          testkit
Architecture:     T0 / Test Support Plane
Production Layer: none
Publish:          false
Current Version:  0.1.2
Target Version:   0.1.2
Supersedes:       existing testkit-spec.md and infra/testkitx-spec.md
Source Goal:      GOAL-DETERMINISTIC-TEST-SUPPORT
Active Path:      .agents/ssot/testkit/spec/spec.md
Ship:             main@testkit-v0.1.2 · PR #247 #254 #255
```

> **Status**：**Stable**（2026-07-14 ship）。实现 W0–W6 已合入 `main`（testkit **0.1.2**，PR #247 #254 #255）。
> 执行计划 / Tasks / 10x verdict 见模块 README 与 `plan/`（路径从模块根解析）。
> **Stable** 不等于 §24 全闭合以外的额外保证；residual 见 §24.0 与 `plan/residual-open.md`。


---

## 0. 文档定位

本文件是 `testkit` 的职责、API、依赖、测试图隔离、演进和机器门禁的唯一规范。

`testkit` 不属于生产运行时分层。

它位于与生产依赖图正交的测试支持平面：

```text
Production graph:
  kernel
    ↑
  types / contracts
    ↑
  infra / adapters / domain / services / apps

Test graph:
  testkit
    → kernel

  contract-testkit
    → contracts / kernel / canonical / decimalx
    → async test runtime（仅 test-support）

  integration harness
    → built binaries
    → real external services
```

因此，`testkit` 的准确身份是：

```text
T0 deterministic test support
```

而不是：

```text
L0 runtime
L1 infra
通用 mock 框架
真实集成测试系统
```

本文件获批后：

- **layer 身份**：`testkit` 为 **test-support**（非 L0/kernel runtime）。**infra.rs 不维护** `.architecture/workspace.toml` / archgate；layer 约束以文档叙述 + `cargo metadata` / 结构扫描为准（非 monorepo archgate 机控）。
- 架构叙述必须将 Test Support 画成正交测试平面（文档/对齐文即可）；
- `testkit` 只能以 `dev-dependency` 被业务 crate 消费；
- `xlib_test!`、`mock!`、`FixtureBuilder` 必须退役；
- provider contract suite 必须迁移到独立测试支持 crate；
- `testkit` 核心最终只保留经过真实复用证明的确定性原语。

---

# 1. 问题的底层本质

测试支持系统的目标不是“减少几行测试代码”，而是控制测试中的隐式输入。

测试不稳定通常来自：

```text
真实墙钟
真实单调钟
sleep
随机数
线程调度
全局状态
环境变量
外部服务
网络
磁盘
未版本化 fixture
默认值 mock
吞错宏
```

`testkit` 的核心价值是：

```text
把隐式输入变成显式、可注入、可推进、可检查的测试状态。
```

它不是为了制造更多抽象。

一个能力只有同时满足以下条件，才允许进入 `testkit` core：

```text
1. 至少两个独立 workspace 消费者需要；
2. 测试语义在消费者之间完全一致；
3. 不依赖具体业务领域；
4. 不连接外部服务；
5. 不要求具体 async runtime；
6. 能显著提高确定性或故障路径覆盖；
7. 直接手写在消费者中会导致重复错误；
8. API 可以长期稳定。
```

不满足任一项：

```text
留在消费方测试模块
或
进入 contract-testkit
或
进入 integration harness
```

---

# 2. 当前实现裁定

## 2.1 可以保留的方向

当前 `ManualClock` 已经分别维护：

```text
wall_nanos
mono_nanos
```

并实现 `Clock::now()` 与 `Clock::monotonic()`。

这解决了早期“ManualClock 只能控制墙钟，单调钟仍读取真实 Instant”的核心问题。

该方向保留，但实现必须进一步收紧：

```text
- 禁止算术溢出静默回绕；
- 支持显式 ClockError 注入；
- 提供一致快照；
- 定义并发线性化语义；
- 禁止用 SeqCst 替代状态模型；
- 控制 API 返回 Result；
- 不使用真实 sleep 验证。
```

## 2.2 必须删除的内容

### `xlib_test!`

当前展开只等价于：

```rust
#[test]
fn ...
```

它没有提供：

```text
隔离
超时
确定性
fixture
日志捕获
故障注入
资源清理
```

保留它只会：

```text
- 增加非标准语法；
- 隐藏真实测试属性；
- 增加 IDE、工具和诊断复杂度；
- 为未来偷偷加入全局行为创造入口。
```

目标终态删除，直接使用：

```rust
#[test]
#[tokio::test]
```

### `mock!`

当前只生成：

```rust
#[derive(Debug, Default, Clone)]
pub struct Name;
```

这不是 mock。

它没有：

```text
行为
调用记录
expectation
错误注入
线程语义
契约
```

它会制造“存在一个 Mock 类型，因此测试充分”的假象。

必须删除。

### `FixtureBuilder<T>`

当前只是：

```rust
PhantomData<T>
```

没有构建字段、默认策略、验证或 fixture。

它属于明确占位实现，必须删除。

### `provider_capability_contract_tests!`

当前宏在 `testkit` core 中直接引用：

```text
canonical
contracts
futures_util
tokio
```

这些依赖没有出现在 `testkit` 自身 Cargo.toml，而是在宏展开后要求调用方提供。

这是隐藏依赖。

宏还硬编码：

```text
stream 必须为空
server_time == 0
position/balance 必须为空
query_order == Pending
invalid venue cancel 必须失败
```

这些是特定 mock fixture 行为，不是所有 provider 实现都应遵守的合同。

该宏必须迁移并拆分。

---

# 3. 目标组件划分

## 3.1 `testkit`

```text
path:    crates/testkit
package: testkit
plane:   test-support
role:    runtime-neutral deterministic primitives
```

目标依赖：

```text
testkit → kernel
```

目标公开面：

```text
ManualClock
ManualClockSnapshot
ManualClockFault
ManualClockError
```

首个稳定版本不再包含任何宏。

## 3.2 `contract-testkit`

```text
path:    crates/test-support/contracts
package: contract-testkit
plane:   test-support
role:    reusable contract conformance suites
```

当前允许依赖（以 Cargo.toml 为准）：

```text
kernel
contracts
canonical
decimalx
async-trait
bytes
futures-core
futures-util
tokio
```

它不是生产模块。

所有生产 crate 只能通过 `[dev-dependencies]` 使用。

契约套件按 trait 分开：

```text
market_data_source
instrument_catalog
account_source
venue_time_source
execution_venue
key_value_store
event_bus
object_store
time_series_store
analytics_sink
pub_sub
instrumentation
tx
repository
```

资源型 suite 必须接收调用方提供的唯一标识，或通过显式 `FixtureNamespace` additive wrapper 派生；命名空间不得读取真实时间、随机数或环境变量。EventBus/PubSub 的可移植 surface 只验证 subscribe/publish 可调用，不声明交付、重放、顺序、确认、背压或次数。0.1.1 `assert_event_bus` 保留为 Snapshot/Replay profile，禁止用于判定实时 adapter。AnalyticsSink / Instrumentation 的 observed suite 依赖调用方观察 seam，只做包含检查，不扩展生产 trait 合同。

TimeSeriesStore 的 0.1.2 `assert_time_series_store` 保留为 ClosedPoint 兼容 profile；可移植验证必须使用调用方窗口入口，不把 `[ts, ts]` 外推为所有后端的端点合同。

禁止使用一个“provider capability”大宏同时测试所有能力。

## 3.3 Integration Harness

```text
path:
  tools/harness
  scripts/integration
  或专用 CI workflow

role:
  real external service orchestration
```

负责：

```text
Docker / Compose
Redis / Kafka / PostgreSQL / TDengine
交易所 testnet
网络故障
进程 kill
真实端口
真实凭据注入
Evidence artifact
```

`testkit` 不承担这些职责。

## 3.4 Fixture 所有权

领域 fixture 默认由拥有其 schema 的 crate 管理。

示例：

```text
crates/domain/macro/tests/fixtures/
crates/adapters/exchange/binance/tests/fixtures/
crates/types/canonical/tests/vectors/
```

只有同一 fixture schema 被至少两个独立 crate 复用时，才允许建立：

```text
crates/test-support/fixtures/<schema>
```

禁止重新引入无行为通用 `FixtureBuilder<T>`。

---

# 4. 目录结构

## 4.1 Core

```text
crates/testkit/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs
│   └── clock.rs
└── tests/
    ├── manual_clock_contract.rs
    ├── manual_clock_concurrency.rs
    ├── compile_fail.rs
    └── production_graph_guard.rs
```

禁止新增：

```text
util.rs
common.rs
prelude.rs
mock.rs
fixture.rs
provider.rs
integration.rs
docker.rs
```

新增模块必须满足准入八问并通过 RFC。

## 4.2 Contract suites

```text
crates/test-support/contracts/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── src/
│   ├── lib.rs
│   ├── fixture.rs
│   ├── fakes/
│   └── suite/
│       ├── market_data_source.rs
│       ├── instrument_catalog.rs
│       ├── account_source.rs
│       ├── venue_time_source.rs
│       ├── execution_venue.rs
│       ├── key_value_store.rs
│       ├── event_bus.rs
│       ├── object_store.rs
│       ├── time_series_store.rs
│       ├── analytics_sink.rs
│       ├── pub_sub.rs
│       ├── instrumentation.rs
│       ├── repository.rs
│       └── tx.rs
└── tests/
    ├── suite_self_tests.rs
    └── negative_implementations.rs
```

---

# 5. 依赖合同

## 5.1 `testkit` 生产依赖

```toml
[dependencies]
kernel = {
    package = "kernel",
    path = "../kernel",
}
```

白名单到此为止。

禁止：

```text
contracts
canonical
decimalx
evidence
observex
configx
tokio
futures
serde
serde_json
rand
proptest
mockall
rstest
tracing
anyhow
```

测试依赖可使用：

```text
proptest
static_assertions
trybuild
loom
```

## 5.2 Features

```toml
[features]
default = []
```

`testkit` core 不允许 feature。

特别禁止：

```text
mock
async
tokio
snapshot
serde
real
integration
```

## 5.3 消费者规则

所有非测试支持 crate：

```toml
[dev-dependencies]
testkit = { path = "..." }
```

禁止：

```toml
[dependencies]
testkit = ...

[build-dependencies]
testkit = ...
```

同样适用于：

```text
contract-testkit
fixture support crates
integration test libraries
```

## 5.4 生产图隔离

所有 binary 和 library 的 normal dependency graph 必须满足：

```text
testkit ∉ production graph
contract-testkit ∉ production graph
```

机器门禁必须从 `cargo metadata` 同时检查 default 与 all-features 的 normal/build 闭包，并报告完整依赖路径。dev edge 与 test-support package 之间的 edge 不计生产污染。

验证：

```bash
node scripts/quality-gates/check-test-support-graph.mjs
node scripts/quality-gates/check-test-support-graph.mjs --json
```

不得依赖 feature resolver 的偶然行为保证隔离。

---

# 6. Crate 级规则

`src/lib.rs` 必须包含：

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
```

禁止：

```text
unsafe
todo!
unimplemented!
占位 public API
生产代码 panic!
静默算术回绕
真实 sleep
真实 SystemTime / Instant
环境变量读取
网络 / 文件 IO
全局 mutable state
隐式 runtime 初始化
隐式 tracing subscriber
```

测试断言本身允许 panic，因为 Rust 测试失败使用 panic 机制。

---

# 7. `ManualClock` 完整合同

## 7.1 目标

`ManualClock` 是 `kernel::Clock` 的确定性测试替身。

它必须支持：

```text
- 独立控制墙钟；
- 独立控制单调钟；
- 墙钟前进；
- 墙钟回退；
- 单调钟只前进；
- 注入墙钟错误；
- 获取一致快照；
- 多线程安全；
- 无真实时间依赖；
- 无算术静默溢出。
```

## 7.2 内部模型

推荐实现：

```rust
pub struct ManualClock {
    state: Mutex<State>,
}

struct State {
    wall: Timestamp,
    monotonic_elapsed: Duration,
    wall_fault: Option<ManualClockFault>,
}
```

选择 `Mutex` 而不是多个 atomics 的原因：

```text
- snapshot 必须一致；
- wall fault 与 wall value 必须线性化；
- checked update 更清晰；
- 测试控制路径不是高频生产热路径；
- 正确性优先于无锁。
```

不得使用多个独立 atomic 后宣称快照一致。

## 7.3 ManualClockFault

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualClockFault {
    BeforeUnixEpoch,
    Overflow,
    Unavailable,
}
```

映射：

```text
BeforeUnixEpoch → ClockError::BeforeUnixEpoch
Overflow        → ClockError::Overflow
Unavailable     → ClockError::Unavailable
```

如果 kernel 当前变体名称尚未迁移完成，testkit 必须跟随已批准 kernel spec，不得自创平行错误语义。

## 7.4 ManualClockError

```rust
#[non_exhaustive]
#[derive(Debug)]
pub enum ManualClockError {
    WallOverflow,
    MonotonicOverflow,
    MonotonicRegression,
    Synchronization,
}
```

必须手写 `Display + Error`；不得为便利增加 `anyhow`。

## 7.5 ManualClockSnapshot

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManualClockSnapshot {
    wall: Timestamp,
    monotonic_elapsed: Duration,
    wall_fault: Option<ManualClockFault>,
}
```

只读 getter：

```rust
impl ManualClockSnapshot {
    pub const fn wall(&self) -> Timestamp;
    pub const fn monotonic_elapsed(&self) -> Duration;
    pub const fn wall_fault(&self) -> Option<ManualClockFault>;
}
```

## 7.6 构造

```rust
impl ManualClock {
    pub fn new(initial_wall: Timestamp) -> Self;

    pub fn with_monotonic_elapsed(
        initial_wall: Timestamp,
        monotonic_elapsed: Duration,
    ) -> Self;
}
```

禁止 `Default`。

理由：

```text
默认 Timestamp(0) 会把未初始化测试输入伪装成合法 epoch 时间。
```

## 7.7 墙钟控制

```rust
impl ManualClock {
    pub fn set_wall(
        &self,
        wall: Timestamp,
    ) -> Result<(), ManualClockError>;

    pub fn advance_wall(
        &self,
        delta: Duration,
    ) -> Result<Timestamp, ManualClockError>;

    pub fn rewind_wall(
        &self,
        delta: Duration,
    ) -> Result<Timestamp, ManualClockError>;
}
```

要求：

```text
- advance 使用 checked_add；
- rewind 使用 checked_sub；
- 失败不修改状态；
- 墙钟允许回退；
- 不提供带符号纳秒 fetch_add；
- 不允许 release 模式回绕。
```

为迁移现有调用，可短期提供：

```rust
#[deprecated]
pub fn set_unix_nanos(...);

#[deprecated]
pub fn advance_unix_nanos(...);
```

必须带删除版本和使用点清单。

## 7.8 单调钟控制

```rust
impl ManualClock {
    pub fn set_monotonic_elapsed(
        &self,
        elapsed: Duration,
    ) -> Result<(), ManualClockError>;

    pub fn advance_monotonic(
        &self,
        delta: Duration,
    ) -> Result<MonotonicInstant, ManualClockError>;
}
```

规则：

```text
- set 值小于当前值 → MonotonicRegression；
- advance 溢出 → MonotonicOverflow；
- 失败不修改状态；
- 不提供 rewind；
- 不接受 signed delta。
```

## 7.9 Fault 控制

```rust
impl ManualClock {
    pub fn set_wall_fault(
        &self,
        fault: ManualClockFault,
    ) -> Result<(), ManualClockError>;

    pub fn clear_wall_fault(
        &self,
    ) -> Result<(), ManualClockError>;

    pub fn wall_fault(
        &self,
    ) -> Result<Option<ManualClockFault>, ManualClockError>;
}
```

语义：

```text
- fault 存在时 Clock::now() 返回对应错误；
- fault 不改变保存的 wall value；
- clear 后重新返回保存的 wall；
- monotonic 不受 wall fault 影响。
```

首个稳定版本不提供“仅失败一次”队列。

只有两个以上真实消费者需要时，才可新增 scripted fault sequence。

## 7.10 Snapshot

```rust
impl ManualClock {
    pub fn snapshot(
        &self,
    ) -> Result<ManualClockSnapshot, ManualClockError>;
}
```

snapshot 必须在同一锁临界区读取所有字段。

## 7.11 Clock 实现

```rust
impl Clock for ManualClock {
    fn now(&self) -> Result<Timestamp, ClockError>;
    fn monotonic(&self) -> MonotonicInstant;
}
```

`now()`：

```text
获取锁
→ 锁失败映射 Unavailable
→ wall_fault 存在则返回对应 ClockError
→ 否则返回 wall
```

`monotonic()` 的 trait 无错误返回，因此必须采用可恢复的锁中毒策略：

```text
- 不在持锁期间执行调用方代码；
- poison 时恢复 inner state；
- 不伪造零值；
- 不 panic；
- 文档明确恢复语义。
```

如果必须报告同步失败，则需要先修改 `Clock::monotonic()` 合同；testkit 不得自行改写。

## 7.12 Clone 语义

`ManualClock` 不实现 `Clone`。

需要共享同一时钟：

```rust
Arc<ManualClock>
```

理由：

```text
Clone 容易让调用方误以为复制独立时间线；
Arc 明确表达共享状态。
```

## 7.13 Send / Sync

必须满足：

```rust
ManualClock: Send + Sync
```

通过 compile assertion 验证。

---

# 8. 宏退役合同

## 8.1 `xlib_test!`

状态：

```text
Deprecated → Removed
```

迁移：

旧：

```rust
xlib_test!(
    fn test_name() {
        ...
    }
);
```

新：

```rust
#[test]
fn test_name() {
    ...
}
```

删除门槛：

```text
workspace 调用点 = 0
external downstream 调用点 = 0
compile fixture 更新
active spec 不再要求该名字
```

## 8.2 `mock!`

状态：

```text
Immediate retirement
```

不提供兼容替代宏。

迁移方案：

```text
- 简单 fake：消费方手写结构体；
- trait fake：在消费方实现目标 trait；
- 多实现共同契约：使用 contract-testkit；
- 调用记录：消费方明确持有 Arc<Mutex<Vec<Call>>>；
- 复杂 expectation：先证明真实需求，再评审专用工具。
```

禁止用另一个生成空壳的宏替代。

## 8.3 `provider_capability_contract_tests!`

状态：

```text
Move and split
```

迁移到：

```text
contract-testkit
```

拆成独立 suite：

```text
market_data_source_contract
instrument_catalog_contract
account_source_contract
venue_time_source_contract
execution_venue_contract
```

## 8.4 `FixtureBuilder<T>`

状态：

```text
Immediate deletion
```

没有迁移 API。

消费方必须建立具体 fixture builder，例如：

```rust
MacroPointFixture
OrderFixture
TickFixture
```

只有真实字段和验证存在时才允许命名为 builder。

---

# 9. Contract Testkit 规范

## 9.1 原则

契约测试必须证明：

```text
```text
同一个 contract 的多个实现具有相同可观察语义。
```

它不是：

```text
验证某个 mock 返回预设默认值。
```

## 9.2 Suite 所有权

每个 suite 对应一个 contracts trait。

示例：

```text
contracts::KeyValueStore
→ contract_testkit::key_value_store

contracts::MarketDataSource
→ contract_testkit::market_data_source
```

Suite 不得依赖具体 adapter crate。

## 9.3 分层测试类型

### Fake contract

验证：

```text
- trait object 可调用；
- 明确配置的输入输出；
- 错误注入；
- 生命周期；
- 调用记录；
- 无外部 IO。
```

### Sandbox contract

验证：

```text
- 本地真实服务；
- 真实协议；
- 可重复 fixture；
- 隔离 namespace；
- cleanup。
```

### Real/Testnet contract

验证：

```text
- 真实远端；
- 网络；
- 凭据；
- 延迟；
- 限流；
- API 漂移。
```

三类不得用同一组默认断言混为一谈。

## 9.4 Suite API

优先使用普通函数，而不是宏。

示例：

```rust
pub async fn assert_key_value_store(
    store: &dyn KeyValueStore,
) -> ContractResult;

pub async fn assert_key_value_store_isolated(
    store: &dyn KeyValueStore,
    fixture: &FixtureNamespace,
) -> ContractResult;
```

若 Rust test discovery 要求生成独立测试函数，可提供极薄声明宏：

```rust
key_value_store_contract_tests!(
    module = redis_fake_contract,
    factory = || InMemoryKvStore::new(),
    profile = KeyValueStoreProfile::in_memory(),
);
```

宏只负责生成测试入口，真实断言在普通函数中。

## 9.5 显式 Profile

不得硬编码：

```text
server_time == 0
stream 必须为空
balance 必须为空
order 必须 Pending
```

应通过 profile 明确：

```rust
pub struct VenueTimeProfile {
    expected_relation: TimeRelation,
}

pub enum StreamExpectation {
    EmptyAndClosed,
    AtLeastOneItem,
    FixtureSequence(Vec<Digest>),
}
```

首个版本只实现真实需要的最小 profile，不预建复杂 DSL。

## 9.6 ContractResult

契约 suite 不得大量 `unwrap()` 造成无法定位的失败。

推荐：

```rust
pub struct ContractFailure {
    contract: &'static str,
    case: &'static str,
    detail: String,
}
```

Suite 返回：

```rust
Result<(), ContractFailure>
```

## 9.7 Hidden dependency 禁止

导出宏或函数使用的所有外部 crate：

```text
必须出现在 contract-testkit Cargo.toml
```

禁止通过宏展开让调用方偶然提供：

```text
tokio
canonical
contracts
futures_util
```

---

# 10. Mock、Fake、Stub、Simulator 术语

必须统一术语。

## 10.1 Stub

```text
固定输入返回固定输出；
不验证调用。
```

## 10.2 Fake

```text
具有简化但可工作的实现；
例如内存 KeyValueStore。
```

## 10.3 Mock

```text
验证交互；
具有 expectation 或调用记录。
```

## 10.4 Simulator

```text
模拟状态变化、时序和故障；
例如交易所撮合模拟器。
```

命名规则：

```text
没有 expectation / call verification 的类型不得命名 Mock。
```

因此当前：

```text
MockBinanceAdapter
MockKvStore
```

必须逐项审计。

若只是简化实现，应改名：

```text
FakeBinanceAdapter
InMemoryKvStore
```

该改名属于下游任务，不阻塞 `testkit` spec，但必须登记。

---

# 11. 测试确定性规则

所有使用 `testkit` 的测试必须满足：

```text
- 不读取真实时间；
- 不用 sleep 等待状态变化；
- 不依赖测试执行顺序；
- 不依赖共享进程全局状态；
- 不依赖未固定随机 seed；
- 不依赖本地时区和 locale；
- 不依赖开发机环境变量；
- 不使用默认端口；
- 不吞掉后台任务错误；
- 不把 retry 当作修复 flaky test。
```

## 11.1 Sleep 禁止

单元和契约测试中：

```text
std::thread::sleep
tokio::time::sleep
```

默认禁止。

合法场景：

```text
- 专门验证真实 runtime timer；
- integration/chaos test；
- 带批准的例外；
- 有最大 deadline；
- 不作为并发正确性唯一证明。
```

## 11.2 随机性

需要随机数据时：

```text
- seed 显式；
- 失败输出 seed；
- property test 可重放；
- 不使用 thread_rng 隐式 seed 作为唯一输入。
```

随机工具不进入 `testkit` core，除非至少两个消费者需要完全相同的 seed 合同。

## 11.3 环境隔离

涉及 env var 的测试：

```text
- 必须串行或进程隔离；
- 必须恢复旧值；
- 优先测试显式 config 输入；
- testkit 不提供全局 env mutation helper。
```

---

# 12. 测试 API 预算

`testkit` stable public API 上限：

```rust
pub use clock::{
    ManualClock,
    ManualClockError,
    ManualClockFault,
    ManualClockSnapshot,
};
```

禁止：

```text
prelude
宏
第三方 re-export
assertion DSL
通用 mock 框架
通用 fixture builder
runtime executor
global test context
lazy static registry
Docker API
network helpers
```

任何新增公开项必须：

```text
- 两个独立消费者；
- API 使用样例；
- 替代方案分析；
- 依赖影响；
- 稳定性分析；
- RFC；
- API snapshot。
```

---

# 13. 测试合同

## 13.1 ManualClock 单元测试

必须覆盖：

```text
- 构造初始 wall；
- 初始 monotonic = 指定值；
- wall advance；
- wall rewind；
- wall 边界溢出；
- rewind 边界溢出；
- monotonic advance；
- monotonic regression 拒绝；
- monotonic overflow；
- wall fault 注入；
- clear fault；
- fault 不改变 wall；
- fault 不影响 monotonic；
- snapshot 一致；
- 失败不修改状态；
- 无 Default；
- 无 Clone；
- Send + Sync。
```

## 13.2 Clock contract suite

`testkit` 自身必须对 `ManualClock` 运行 kernel Clock 合同：

```text
- now 返回当前 wall 或显式错误；
- wall 允许回退；
- monotonic 非递减；
- wall 和 monotonic 独立；
- 无真实时间推进；
- 两次读取之间没有 control call 时值不变。
```

SystemClock 不应该被要求通过“值不变”测试。

因此 Clock suite 必须分为：

```text
ClockCommonContract
ManualClockDeterminismContract
SystemClockSmokeContract
```

禁止错误地要求所有 Clock 实现语义完全相同。

## 13.3 Property tests

必须覆盖：

```text
任意 Timestamp + Duration 的 checked advance
任意 Timestamp - Duration 的 checked rewind
任意 monotonic Duration + Duration
失败前后 snapshot 不变
任意 fault set/clear sequence
```

## 13.4 并发测试

必须覆盖：

```text
多线程读取 now
多线程读取 monotonic
控制线程推进 wall
控制线程推进 monotonic
snapshot 不出现撕裂组合
无数据竞争
无死锁
共享通过 Arc 显式发生
```

ManualClock 的线性化点：

```text
成功获取 state mutex 后的状态读写。
```

## 13.5 Compile-fail / compile assertions

验证：

```text
ManualClock: !Default
ManualClock: !Clone
ManualClock: Send + Sync
testkit 不导出 xlib_test
testkit 不导出 mock
testkit 不导出 FixtureBuilder
testkit 不导出 provider macro
```

## 13.6 Mutation testing

不得存活：

```text
advance_wall checked_add → wrapping_add
rewind_wall checked_sub → wrapping_sub
monotonic regression 判断反转
fault 被忽略
fault 清除无效
snapshot 字段错配
monotonic 与 wall 共用状态
失败后仍修改状态
```

目标：

```text
mutation score >= 90%
```

## 13.7 Coverage

```text
line coverage   >= 95%
branch coverage >= 90%
```

`testkit` 体积应很小，因此不接受低覆盖率。

## 13.8 Miri

定期执行：

```bash
cargo miri test -p testkit
```

## 13.9 Contract-testkit 自测试

每个 contract suite 必须至少有：

```text
一个明确通过的 reference fake
一个故意违反合同的 broken fake
```

当前选择矩阵覆盖 contracts 的 14 个 trait；`tests/negative_implementations.rs` 含 15 个 broken case（AnalyticsSink 分 callable 与 observed 两例）。每个反例必须断言精确的 `ContractFailure.contract` 与 `ContractFailure.case`，禁止仅以“返回任意错误”计为杀死。

Fake/reference/broken 自测只证明 suite 的局部判别能力，不是 Sandbox、Real/Testnet 或生产 backend readiness 证据。

Broken fake 必须使对应 suite 失败。

否则 suite 只证明“能运行”，没有证明“能杀死错误实现”。

---

# 14. 测试图隔离门禁

## 14.1 Cargo metadata 规则

`xtask lint-deps` 必须检查：

```text
TESTKIT-GRAPH-001:
  testkit 只能作为 dev dependency。

TESTKIT-GRAPH-002:
  contract-testkit 只能作为 dev dependency。

TESTKIT-GRAPH-003:
  test-support crate 不得作为 build dependency。

TESTKIT-GRAPH-004:
  app/service normal graph 不得包含 test-support layer。

TESTKIT-GRAPH-005:
  feature 启用不得把 test-support 引入 normal graph。
```

## 14.2 Release 规则

所有 release binary：

```bash
cargo tree -p <binary> -e normal
```

不得包含：

```text
testkit
contract-testkit
test fixture crate
integration harness library
```

## 14.3 Source guard

禁止生产源码：

```rust
use testkit::...
use contract_testkit::...
```

允许位置：

```text
#[cfg(test)] module
tests/
benches/
examples explicitly marked non-production
```

但 Cargo 依赖仍必须是 dev-dependency。

## 14.4 Macro expansion guard

目标终态 core 无宏，因此不再存在宏把测试依赖生成到生产代码的旁路。

contract-testkit 宏如果存在，必须：

```text
- 只能生成 #[cfg(test)] 项；
- 有 compile fixture 证明；
- 不导出 production symbols。
```

---

# 15. Archgate 规则（infra.rs OOS）

> **infra.rs OOS**：本仓**不移植** `tools/archgate` / `.architecture`。
> 下列 TESTKIT-* 规则 ID 仅为 **历史 monorepo 参考**；**不**构成本仓强制验收条件，也**不**要求 `cargo run -p archgate`。
> 可选残留机控（非 archgate）：结构扫描 / `lint-deps` / `test-graph-check`（若本仓落地）/ CI job / public-api 快照。

历史规则 ID 参考（仅枚举，不强制实现）：

```text
TESTKIT-LAYER-001, TESTKIT-DEP-001, TESTKIT-FEATURE-001,
TESTKIT-API-001, TESTKIT-MACRO-001, TESTKIT-PLACEHOLDER-001,
TESTKIT-TIME-001, TESTKIT-IO-001, TESTKIT-GRAPH-001,
TESTKIT-HIDDEN-DEP-001, TESTKIT-NAMING-001
```

语义摘要（文档约束，非 archgate 机控）：

- **LAYER**：testkit 为 test-support 平面（文档 + cargo metadata 叙述）
- **DEP / FEATURE**：normal dep 仅 kernel；无非 default feature
- **API / MACRO / PLACEHOLDER**：公开面预算；禁宏与 ZST placeholder
- **TIME / IO**：禁真实时间 / sleep / 文件网络环境 IO
- **GRAPH**：生产 normal graph 不得引入 testkit
- **HIDDEN-DEP / NAMING**：禁隐藏宏依赖；Mock* 命名策略（warning→fail 由政策决定）

---

# 16. CI

## 16.1 Core 必须执行

```bash
cargo fmt -- --check
cargo clippy -p testkit --all-targets -- -D warnings
cargo test -p testkit
cargo llvm-cov -p testkit --fail-under-lines 95
cargo mutants -p testkit
cargo miri test -p testkit
# N/A (infra.rs OOS): cargo run -p archgate -- --json  — 本仓不引入 archgate
cargo run -p xtask -- lint-deps          # 若本仓有 xtask；否则结构扫描 / cargo metadata
cargo run -p xtask -- crate-standard --check
```

## 16.2 Contract-testkit

```bash
cargo clippy -p contract-testkit --all-targets -- -D warnings
cargo test -p contract-testkit
cargo test -p contract-testkit --test negative_implementations
```

## 16.3 Production graph

```bash
cargo run -p xtask -- test-graph-check
```

建议新增专用命令，而不是把所有逻辑继续堆入通用文本扫描。

输出必须列出：

```text
test-support package
consumer
dependency kind
target
feature path
verdict
```

## 16.4 Nightly

```text
full mutation
Miri
property test extended cases
contract suite broken-implementation matrix
workspace production graph audit
```

---

# 17. 文档要求

## 17.1 README

必须说明：

```text
- testkit 是 test-support，不是 L0 runtime；
- 为什么只提供 ManualClock；
- 如何通过 Arc 共享；
- 如何推进 wall / monotonic；
- 如何注入 ClockError；
- 为什么不提供 mock!；
- 为什么不提供 xlib_test!；
- contract-testkit 和 integration harness 在哪里。
```

## 17.2 AGENTS

必须包含：

```text
- 只能 dev-dependency；
- 依赖白名单；
- 公开 API 预算；
- 禁止 placeholder；
- 禁止真实时间和 sleep；
- 变更验证命令；
- 两消费者准入规则。
```

## 17.3 CHANGELOG

必须记录：

```text
- 宏退役；
- FixtureBuilder 删除；
- provider suite 迁移；
- ManualClock API 迁移；
- layer 从 kernel 改为 test-support。
```

---

# 18. 版本与稳定性

## 18.1 当前状态

当前 registry 应保持：

```toml
status = "incubating"
```

并修改：

```toml
layer = "test-support"
```

在所有验收闭合前不得标为 stable。

## 18.2 版本

目标：

```text
0.1.0 → 0.1.1
```

仓库 patch-default 不能掩盖 API 删除。

宏和 placeholder 删除必须：

```text
- Approved RFC；
- 消费者清单；
- 同批迁移；
- CHANGELOG；
- public API diff；
- compatibility decision。
```

## 18.3 发布

```text
publish = false
```

不发布 crates.io。

即使只在 workspace 内使用，仍执行 API 快照，因为大量测试可能依赖它。

---

# 19. 迁移流程

## 19.1 Phase 0：冻结

立即禁止新增：

```text
xlib_test!
mock!
FixtureBuilder
provider_capability_contract_tests!
testkit normal dependency
```

执行：

```bash
rg -n \
  'xlib_test!|mock!|FixtureBuilder|provider_capability_contract_tests!' \
  --glob '*.rs' \
  .

cargo metadata --format-version 1
```

保存消费者清单。

## 19.2 Phase 1：ManualClock V2

实现：

```text
Mutex state
checked wall operations
checked monotonic operations
fault injection
snapshot
no Clone / no Default
```

先新增 V2 API，保留旧方法 deprecated。

## 19.3 Phase 2：迁移时钟消费者

旧：

```rust
ManualClock::new(100)
clock.set(200)
clock.advance(50)
clock.advance_mono(10)
```

新：

```rust
let clock = ManualClock::new(
    Timestamp::from_unix_nanos(100),
);

clock.set_wall(
    Timestamp::from_unix_nanos(200),
)?;

clock.advance_wall(
    Duration::from_nanos(50),
)?;

clock.advance_monotonic(
    Duration::from_nanos(10),
)?;
```

迁移完成后删除旧 API。

## 19.4 Phase 3：删除无价值 API

删除：

```text
xlib_test!
mock!
FixtureBuilder
```

如果 workspace 调用点已经为零，不建立兼容层。

## 19.5 Phase 4：拆分 provider suite

建立：

```text
crates/test-support/contracts
```

将 provider 大宏拆成 trait-level suites。

修改 Binance / OKX：

```toml
[dev-dependencies]
testkit = ...
contract-testkit = ...
```

测试真实行为分别放在：

```text
fake contract
sandbox contract
real/testnet contract
```

## 19.6 Phase 5：架构对齐

修改：

```text
# N/A (infra.rs OOS): .architecture/workspace.toml  — 本仓不维护 .architecture / archgate
# layer=test-support 以文档 + cargo metadata 叙述为准
docs/architecture/spec.md   # 或本仓 docs/ssot 对齐文
STRUCTURE.md                # 若存在
.agents/ssot/testkit/
旧 infra/testkitx active spec
README
AGENTS
CHANGELOG
```

删除双重 spec：

```text
.agents/ssot/testkit/testkit-spec.md        # → 已归档为 spec/TESTKIT-SPEC-001.superseded.md
.agents/ssot/testkitx/testkitx-spec.md
```

保留一个活动权威：

```text
.agents/ssot/testkit/spec/spec.md
```

历史文档标记 Superseded 或移入 archive。

## 19.7 Phase 6：防回流

实现：

```text
test graph checker
public API snapshot
negative fixtures
macro export guard
placeholder guard
production graph guard
```

---

# 20. PR 切分

推荐：

## PR-1：Spec 与冻结

```text
SPEC-TESTKIT-002
ADR
consumer inventory
no-new-placeholder guard
layer → test-support proposal
```

## PR-2：ManualClock V2

```text
state model
checked API
fault injection
snapshot
tests
```

## PR-3：消费者迁移

```text
ManualClock old API → V2
删除 deprecated 使用点
```

## PR-4：删除宏和 placeholder

```text
xlib_test!
mock!
FixtureBuilder
public API snapshot
```

## PR-5：Contract suites 拆分

```text
contract-testkit
provider suite split
Binance / OKX migration
negative implementations
```

## PR-6：治理闭合

```text
architecture docs
active spec dedupe
test graph gate
negative fixtures
Evidence
```

每个 PR：

```text
独立 worktree
禁止 main 开发
可独立回滚
main 始终 green
不混入 kernel/evidence/gate 的大规模改造
```

---

# 21. Evidence

每次 testkit 变更生成：

```text
evidence/testkit/<date>-<change-id>/
├── manifest.json
├── commit.txt
├── cargo-metadata.json
├── consumers.json
├── production-graph.json
├── public-api.diff
├── fmt.log
├── clippy.log
├── tests.log
├── coverage.json
├── mutants.json
├── miri.log
├── contract-negative-tests.log
├── archgate.json              # N/A（infra.rs 不引入 archgate；不构成本仓验收）
└── verdict.md
```

Evidence 必须证明：

```text
- 当前 commit；
- testkit 未进入 production graph；
- broken fake 能被 contract suite 杀死；
- ManualClock 不读取真实时间；
- 无宏/placeholder 回流；
- SKIP 不计 PASS。
```

## 21.1 Stable evidence（2026-07-14 实测）

战役 ship（main@testkit-v0.1.1，PR #247 #254 #255）时的门禁实测结果。证据目录：`evidence/testkit/2026-07-14-stable-gates/`（仓库根 `evidence/`，非本 spec 目录）。

| 门禁 | 结果 |
|------|------|
| unit / contract / concurrency | **PASS** |
| property (proptest) | **PASS** |
| `cargo mutants -p testkit` | **missed=0**（caught=12, unviable=18） |
| `cargo +nightly miri test -p testkit` | **PASS** |
| line coverage gate | CI `testkit-quality` ≥95% **PASS** |
| `cargo xtl test-graph-check` | **PASS** |
| inventory-ssot / migration | **PASS** |

> 诚实声明：以上为 2026-07-14 ship 时点实测。`branch coverage ≥90%` 仍为 **OPTIONAL**（line≥95% 已强制），见 §24.3 与 residual；不据此宣称 §24 全闭合以外的保证。

---

# 22. 1 天、7 天、30 天计划

## 1 天

```text
- 重新评级为 incubating / 2.5 of 5；
- 冻结新增宏与 FixtureBuilder 使用；
- 完成消费者清单；
- 批准 T0 Test Support 定位；
- 起草 ManualClock V2 API；
- 添加 production graph 检查原型。
```

## 7 天

```text
- ManualClock V2 完成；
- 全部消费者迁移；
- xlib_test! / mock! / FixtureBuilder 删除；
- provider 大宏移出 core；
- contract-testkit 骨架和首个 KeyValueStore suite 完成；
- negative broken fake 可被杀死。
```

## 30 天

```text
- provider trait suites 完整；
- Binance / OKX fake、sandbox、real 分层；
- 所有 shared mock/fake 命名审计；
- test graph gate 完整；
- mutation / Miri / property tests 纳入 CI；
- 无 test-support 进入 release graph；
- 达到 stable 验收。
```

---

# 23. 衡量指标

```text
testkit_production_dependents              = 0
contract_testkit_production_dependents     = 0
testkit_public_macro_count                 = 0
testkit_placeholder_public_type_count      = 0
real_time_calls_in_testkit                 = 0
sleep_calls_in_unit_contract_tests         = 0
manual_clock_unchecked_arithmetic_count    = 0
manual_clock_line_coverage                 >= 95%
manual_clock_branch_coverage               >= 90%
manual_clock_mutation_score                >= 90%
contract_suite_broken_impl_kill_rate       = 100%
hidden_macro_dependency_count              = 0
active_testkit_spec_count                  = 1
flaky_retry_usage                          = 0
```

---

# 24. 完成定义

只有全部满足，才允许：

```text
Progress = 3/3
Quality  = 5/5
Status   = stable
```

## 24.0 Ship 后状态（2026-07-14）

战役已于 2026-07-14 ship（main@testkit-v0.1.1，PR #247 #254 #255），**Status = Stable**（DEF-001…010 全 CLOSED，Stable CLAIMED）。下方 §24.1–.6 的 `[ ]` 为合同启动期的验收清单原貌；ship 时点实测证据见 §21.1。

**残留（非阻塞 Stable）**：
- `branch coverage ≥90%`：**OPTIONAL**（line≥95% 已强制 CI；见 §24.3 与 residual-open）。
- 全 contract suite 矩阵 / Miri 进 CI required 周期：演进度量，不影响 0.1.1 Stable 评级。
- integration harness：跨 crate 依赖（INFRA-010+），非 testkit 本体范围。

> 诚实声明：Spec **Approved / Stable** 不等于 production runtime 就绪（testkit 本就 `publish=false`、T0 test-support plane，无 production layer）。

## 24.1 定位闭合

```text
[ ] layer = test-support
[ ] 不再声明为 L0 runtime
[ ] active spec 只有一份
[ ] README / AGENTS / architecture 对齐
```

## 24.2 Core 闭合

```text
[ ] 只依赖 kernel
[ ] 无 feature
[ ] 无宏
[ ] 无 FixtureBuilder
[ ] 无 provider suite
[ ] ManualClock V2
[ ] 无真实时间
[ ] 无 sleep
[ ] 无 unchecked arithmetic
[ ] 无 Clone / Default
```

## 24.3 测试闭合

```text
[ ] unit
[ ] property
[ ] concurrency
[ ] compile assertions
[ ] line >= 95%
[ ] branch >= 90%
[ ] mutation >= 90%
[ ] Miri
```

## 24.4 Contract 闭合

```text
[ ] trait-level suites
[ ] 无具体 adapter dependency
[ ] 显式 profile
[ ] broken implementation negative tests
[ ] fake/sandbox/real 分层
[ ] 无隐藏依赖
```

## 24.5 图隔离闭合

```text
[ ] 所有消费均为 dev-dependency
[ ] 无 build-dependency
[ ] 所有 release normal graph 无 test-support
[ ] feature 不会泄漏 test-support
[ ] machine gate 生效
```

## 24.6 治理闭合

```text
[ ] RFC / ADR
[ ] public API snapshot
[ ] # N/A (infra.rs OOS): archgate — 本仓不移植；非验收勾选
[ ] xtask test-graph-check（或等价结构扫描 / 图隔离证明）
[ ] negative fixtures
[ ] CHANGELOG
[ ] Evidence
```

---

# 25. 最终裁定

`testkit` 应保留，但它不是一个“测试工具大全”。

稳定终态：

```text
testkit
  = 极小的 runtime-neutral deterministic primitives
  = 当前只保留 ManualClock

contract-testkit
  = 面向 contracts trait 的可复用一致性套件

integration harness
  = 真实基础设施、网络、进程和故障测试
```

必须删除：

```text
xlib_test!
mock!
FixtureBuilder
provider_capability_contract_tests! from core
```

核心原则：

```text
测试抽象只有在杀死真实错误时才有价值。

生成一个空 Mock 类型不是测试能力；
包装 #[test] 不是测试能力；
零字段 FixtureBuilder 不是测试能力；
硬编码默认返回值的 provider 宏不是通用合同。

testkit 的质量不由公开 API 数量决定，
而由它消除了多少隐式时间、隐藏依赖、测试漂移和生产图污染决定。
```

---

# 26. Ship record（已实现范围）

战役执行波次（W0–W6 + 0.1.1 release）已全部 CLOSED 并合入 `main`。

| Wave | 状态 | 证据 |
|------|------|------|
| W0 计划/冻结 | **CLOSED** | PR [#247](https://github.com/xhyperium/infra.rs/pull/247) |
| W1 ManualClock V2 | **CLOSED** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) |
| W3 删宏/FixtureBuilder | **CLOSED** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) |
| W4 contract-testkit | **CLOSED** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) · package `contract-testkit` |
| W5 layer=test-support | **CLOSED** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) · `workspace.toml` |
| W6 test-graph-check | **CLOSED** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) · `cargo xtl test-graph-check` |
| 0.1.1 release | **CLOSED** | PR [#255](https://github.com/xhyperium/infra.rs/pull/255) · tag `testkit-v0.1.1` |

> 诚实声明：Ship record 记录战役执行进度，**不**等于 §24 全闭合以外的额外质量保证。residual 见 `plan/residual-open.md`（DEF-001…010 全 CLOSED + 1 OPTIONAL）。

---

# 27. 双镜像校验

本文件 `xhyper-testkit-complete-spec.md` 与短名 `spec/spec.md` 是**字节级双镜像**（AGENTS.md §2.4）。任一侧改动必须同步另一侧，合并前 `cmp` 必须 exit 0：

```bash
cmp .agents/ssot/testkit/spec/spec.md \
    .agents/ssot/testkit/spec/xhyper-testkit-complete-spec.md
```

- 短名 `spec/spec.md` 是管线入口；`xhyper-testkit-complete-spec.md` 是完整合同副本（长描述性文件名，便于外部引用）。
- 只改一侧视为漂移；CI / README 验证段强制 `cmp`。
- 被取代的旧 Spec 留在 `spec/` 内，文件名带 `.superseded`（见 `spec/TESTKIT-SPEC-001.superseded.md`）。
