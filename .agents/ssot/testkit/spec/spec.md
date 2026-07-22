# SPEC-TESTKIT-002 · 确定性测试支持合同

| 字段 | 值 |
|---|---|
| 状态 | Active |
| package / lib | `testkit` / `testkit` |
| 当前版本 | `0.1.3` |
| 发布 | `publish = false` |
| 架构身份 | T0 测试支持平面；L1 确定性测试原语；无生产运行时层级 |
| 物理路径 | `crates/testkit` |
| 依赖 | normal dependency 仅 `kernel` |

本文是 infra.rs 本仓 `testkit` 当前状态的 active SSOT。规格状态不证明实现、测试或门禁已经通过；实现事实以 `crates/testkit`、`cargo metadata` 和本轮新鲜证据为准。

## 1. 目标与边界

`testkit` 把测试中的隐式时间与步骤执行状态变成显式、可控制、可检查的输入。它只能通过业务 crate 的 `dev-dependencies` 使用，不得进入生产依赖图。

```text
生产图：kernel → types/contracts → infra/adapters → services/apps

测试图：testkit → kernel
        contract-testkit → testkit/contracts/canonical
        external integration harness → 已构建二进制/真实外部服务
```

本 crate 当前包含两类能力：

1. `ManualClock`：`kernel::Clock` 的确定性测试替身。
2. `IntegrationHarness`：crate 内的确定性 scenario runner，顺序执行内存闭包并记录结果。

这里的 `IntegrationHarness` 不是“真实外部集成测试 harness”。后者负责网络、进程、容器、端口、凭据、真实服务、故障注入与 CI evidence，归属 `tools/` 或 CI，当前仍为 OOS。不得因公开类型名相同而把外部能力塞进 `testkit`。

`contract-testkit` 负责 trait 级 Fake 与 contract suite，不属于本 crate 的公开面。

## 2. 公开面

允许的公开类型分为两组：

```text
ManualClock
ManualClockError
ManualClockFault
ManualClockSnapshot

IntegrationHarness
HarnessReport
HarnessRunError
StepOutcome
StepRecord
```

“ManualClock 族四类型”只描述时钟子模块，不是整个 crate 的公开类型上限。不得以“仅四类型”为由删除确定性 runner，也不得用 runner 为任意测试工具扩张公开面。

公开项必须有中文 rustdoc；crate 保持 `#![forbid(unsafe_code)]`、`#![deny(missing_docs)]` 与 `#![deny(unreachable_pub)]`。

## 3. `ManualClock` 合同

### 3.1 状态模型

单个私有 `Mutex<State>` 同时保护：

```text
wall: Timestamp
monotonic_elapsed: Duration
wall_fault: Option<ManualClockFault>
```

同一实例的状态操作以取得该 Mutex 为线性化点。禁止把三个字段拆成独立锁或原子量后再拼装快照；禁止以 `SeqCst` 代替状态不变量。

`ClockDomain` 是实例级不可变身份，不参与可变状态锁。每个 `ManualClock` 实例在 allocator 未耗尽的前提下获得独立 domain；跨 domain 的 `checked_duration_since` 必须返回 `None`。

### 3.2 构造与哨兵

- `ManualClock::new(initial_wall)` 要求调用方显式传入 `Timestamp`；单调流逝初始为零。
- `with_monotonic_elapsed` 同时显式接收墙钟和单调流逝。
- 不实现 `Default`，不实现 `Clone`；共享同一时间线使用 `Arc<ManualClock>`。
- 显式传入 Unix epoch 0 是调用方选择的合法测试数据；禁止把 epoch 0 当作读取失败、锁失败、fault、panic 或缺失记录的 sentinel/fallback。

### 3.3 Checked 控制路径

以下控制操作返回 `Result<_, ManualClockError>`：

- 设置、推进、回退墙钟；
- 设置、推进单调钟；
- 设置、清除、读取墙钟 fault；
- 读取一致快照。

墙钟加减与单调钟推进必须使用 checked arithmetic。单调钟不得回退。任何溢出、回退或同步失败均须返回对应 typed error，且整个状态保持调用前值。

### 3.4 Fault 与 `Clock` 实现

`ManualClockFault::{BeforeUnixEpoch, Overflow, Unavailable}` 只影响 `Clock::now()`，并分别映射到同名语义的 `kernel::ClockError`。设置 fault 不改变已保存的墙钟或单调钟；清除 fault 后恢复读取原墙钟。

- `Clock::now()`：正常时返回保存的墙钟；fault 返回对应 `ClockError`；锁中毒返回 `ClockError::Unavailable`。
- `Clock::monotonic()`：无错误通道，因此锁中毒时恢复 poison 中的原 `State`；不伪造零值、不 panic、不改变 trait 签名。
- 任何代码都不得在持有内部锁时执行调用方闭包。

### 3.5 一致快照

`ManualClockSnapshot` 在同一锁临界区读取墙钟、单调流逝与 fault。字段私有，只通过只读 getter 访问。快照不得将同步失败或 fault 降级为默认值。

### 3.6 Domain allocator 边界

当前 domain allocator 的唯一性声明只覆盖“单进程生命周期内计数器未耗尽”的范围。allocator 耗尽必须保留为显式 residual，不能靠整数回绕继续分配并宣称 domain 唯一。若实现尚未提供 typed exhaustion 路径，则不得宣称覆盖无限构造次数或跨进程唯一性。

## 4. `IntegrationHarness` 合同

### 4.1 定位

`IntegrationHarness` 是围绕一个 `ManualClock` 的单进程、顺序、一次性 scenario runner。step 是内存闭包；runner 不创建线程或 runtime，不访问网络、文件、环境变量、进程、容器或真实时间，也不 sleep。

它适合验证“执行 step → 推进确定性时钟 → 检查 typed 结果”的 crate 内场景。需要真实 Redis/Kafka/Postgres、交易所 testnet、端口、凭据、进程 kill 或网络故障的测试必须进入 external integration harness，不能通过 feature 隐藏进本 crate。

### 4.2 消费型生命周期

runner 使用所有权表达一次性状态迁移：

```rust
pub fn step<F, E>(self, name: impl Into<String>, f: F) -> Self
where
    F: FnOnce(&ManualClock) -> Result<(), E> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static;

pub fn step_advance_wall(self, name: impl Into<String>, delta: Duration) -> Self;
pub fn step_advance_monotonic(self, name: impl Into<String>, delta: Duration) -> Self;
pub fn run(self) -> Result<HarnessReport, HarnessRunError>;
```

`step*` 消费并返回 builder；`run` 消费 builder。Rust 所有权因此使运行后追加和成功/失败后重跑均不能编译，而不是运行时静默忽略或返回缓存结果。空 scenario 可以成功并返回空的 `HarnessReport`。

step 按登记顺序执行；首个非成功结果后停止，后续 step 不执行。builder 不实现可制造第二份独立执行队列的 `Clone`。

### 4.3 Step、panic 与结果

- step 名称由调用方显式提供；runner 不生成依赖全局状态的随机名称。
- step 的错误类型必须满足 `Error + Send + Sync + 'static`；runner 保存该 source，形成 `StepOutcome::Failed`，并使 `run()` 返回保留 source chain 的 `HarnessRunError`。
- step panic 必须用 `catch_unwind` 截获，形成 panic 结果并使 `run()` 返回 typed error；不得让 panic 穿透 runner，也不得在 unwind 时二次 panic。
- panic payload 不能安全转换为文本时使用明确的非文本说明，不得伪装成成功。
- `assert_*` 辅助只属于 `HarnessReport`，可按 Rust 测试惯例 panic；builder 执行控制路径本身不得靠 panic 报错。

`StepOutcome` 固定区分 `Passed`、`Failed`、`Panicked` 与 `ObservationFailed`；不得压缩回 `bool + String`。

### 4.4 记录与时钟失败

`StepRecord` 的字段必须私有，并为名称、outcome 和 step 前后可用的 `ManualClockSnapshot` 提供只读 getter。调用方不得直接改写记录。

runner 在执行 step 前后读取时钟/快照时：

- snapshot 同步失败或 snapshot 中存在 wall fault 时必须形成 `StepOutcome::ObservationFailed`，并返回 `HarnessRunError`；
- 失败必须通过缺失 snapshot 的显式类型表达；
- 禁止 `unwrap_or(0)`、空字符串或成功布尔值掩盖失败；
- 禁止使用 epoch 0 作为“读取失败”的墙钟记录；
- 已产生的记录保持可查询，未执行的 step 不生成成功记录。

`HarnessReport` 持有最终 `ManualClock` 与 `StepRecord` 集合，提供只读访问和断言 helper。`HarnessRunError` 是 `run()` 的唯一失败通道，保留 terminal report 与业务/时钟 source（若存在），并正确实现 `Error::source()`；调用方可从错误检查已经产生的记录。公开错误/结果枚举使用 `#[non_exhaustive]` 保留兼容扩展空间。

## 5. 依赖与消费合同

- normal dependency 仅允许 `kernel`；第三方测试依赖必须从 workspace 统一引用。
- `default = []`；不得通过 feature 引入 async runtime、网络、I/O、tracing 全局初始化或外部服务客户端。
- 业务 crate 只能在 `[dev-dependencies]` 引用 `testkit`。
- `testkit` 与 `contract-testkit` 的生产 normal dependents 必须为零。
- `publish = false`；本合同不承诺 crates.io 下游兼容性。

## 6. 禁止项

禁止重新引入：

- `xlib_test!`、`mock!`、`FixtureBuilder`；
- `provider_capability_contract_tests!` 或依赖调用方隐式提供 crate 的导出宏；
- 无行为 public placeholder；
- 真实 `SystemTime` / `Instant`、sleep、随机数、环境变量、网络、文件 I/O、全局可变状态；
- unchecked 时间算术、静默回绕、错误到默认值的降级；
- 把 T0/L1 test-support 描述成 production runtime 或 package stable。

## 7. 验收合同

实现变化交付前至少验证：

1. ManualClock：显式构造、wall/mono checked 边界、fault 映射、一致快照、poison 恢复、独立 domain、跨 domain 不可比较。
2. Runner：成功顺序、首错停止、空 scenario、业务失败 source、panic、clock fault、消费后不可重跑/追加、私有记录 getter、report 所有权与无 epoch 0 sentinel。
3. 公开面：compile/API surface 锁定新增 typed 类型、消费型 builder 签名与 `StepOutcome` 四态，旧 public 字段访问不能继续编译。
4. 图隔离：normal dependents 为零；默认 feature 为空；源树无网络/进程/I/O/真实时间能力。
5. 质量：相关 unit/integration/property/concurrency 测试、fmt、clippy 与仓库质量门禁产生本轮新鲜结果。

历史 release、tag、PR 或 evidence 只能说明当时版本，不能作为本轮变更的 fresh PASS。

## 8. 版本与 residual

- 当前交付版本为 `0.1.3`；相对 `0.1.2` 的 runner fail-closed 行为仅执行一次 PATCH bump。
- typed runner 与 fail-closed 行为是可观察行为变化；主实现交付时必须按仓库版本规则执行 PATCH bump，并同步 Cargo、锁文件、消费者 path-version、CHANGELOG/对齐文与本 SSOT 的版本字段。
- `R-CLK-DOMAIN-EXHAUSTION`：domain allocator 耗尽的 typed 失败/防回绕尚需实现或明确接受；在闭合前保持 OPEN，不得用 process-lifetime 假设冒充无限唯一性证明。
- external integration harness（tools/CI）保持 OOS；OOS 不等于 PASS，也不阻塞 crate 内确定性 runner 的合同闭合。
