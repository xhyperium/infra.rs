# `kernel` Crate 规范

```text
Spec ID:         SPEC-KERNEL-002
Status:          Approved / Active
Owner:           platform
Physical Path:   crates/kernel
Package / lib:   kernel / kernel
Current Version: 0.3.1
Publish:         false
Layer:           L0 / Kernel
```

本文件是 `kernel` 当前实现、测试与演进合同。源码和测试提供可执行事实；两者与本文件冲突时，必须在同一变更中消除冲突，不得以历史 evidence 覆盖当前事实。

当前成熟度仅声明 **L1 Internal Ready**；L4 只指本文列出的已证支持面。它不表示 `production-certified`、全场景 Platform Ready 或 crates.io 发布状态。

当前交付版本为 `0.3.1`；相对 `0.3.0` 的 `wait_timeout` 行为与公开错误面变更已执行一次 patch bump。

## 1. 职责与边界

`kernel` 是 workspace 的 L0 语义信任根，只定义三类全局语义：

- `error`：调用方应如何响应错误；
- `clock`：墙钟、单调钟、时钟域及安全计算；
- `lifecycle`：组件状态与同步关停信号。

一个能力只有同时满足以下条件才可进入 `kernel`：全系统必须唯一；并存语义会造成错误响应、时间失真或关停失控；不属于单一领域；预计长期稳定。

配置、观测、网络、存储、序列化、异步运行时、依赖注入、重试、调度、领域类型、测试替身和组件编排均不属于 `kernel`。

`kernel` 不提供 `Component` trait、全局 `Clock`、serde wire 表示、运行时特定类型、prelude、宏或第三方 crate 重导出。

## 2. 依赖与编译合同

```text
workspace 生产依赖：无
第三方生产依赖：thiserror
默认 feature：空
unsafe：禁止
```

`proptest`、`serde` 和 `static_assertions` 仅可作为 dev-dependency。`loom` 仅可作为 `cfg(loom)` target dependency；默认生产依赖图不得包含它。

`src/lib.rs` 必须启用：

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
```

生产代码不得使用 `todo!`、`unimplemented!`、`unwrap()` 或 `expect()` 伪造正常失败。锁中毒可恢复内部状态，但不得伪装成功或“未触发”。

## 3. `error` 合同

### 3.1 公开类型

```rust
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type XResult<T> = Result<T, XError>;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Invalid,
    Missing,
    Conflict,
    Transient,
    Unavailable,
    Cancelled,
    DeadlineExceeded,
    Invariant,
    Internal,
}

pub struct XError { /* private */ }
```

`XError` 公开方法固定为：

```rust
impl XError {
    pub fn invalid(context: impl Into<String>) -> Self;
    pub fn missing(context: impl Into<String>) -> Self;
    pub fn conflict(context: impl Into<String>) -> Self;
    pub fn transient(context: impl Into<String>) -> Self;
    pub fn transient_after(context: impl Into<String>, retry_after: Duration) -> Self;
    pub fn unavailable(context: impl Into<String>) -> Self;
    pub fn cancelled(context: impl Into<String>) -> Self;
    pub fn deadline_exceeded(context: impl Into<String>) -> Self;
    pub fn invariant(context: impl Into<String>) -> Self;
    pub fn internal(context: impl Into<String>) -> Self;
    pub fn with_source(self, source: impl Into<BoxError>) -> Self;
    pub fn kind(&self) -> ErrorKind;
    pub fn context(&self) -> &str;
    pub fn retry_after(&self) -> Option<Duration>;
    pub fn is_retryable(&self) -> bool;
    pub fn is_bug(&self) -> bool;
}
```

分类按调用方反应定义：输入错误为 `Invalid`；资源缺失为 `Missing`；状态冲突为 `Conflict`；可重试短暂故障为 `Transient`；依赖不可用为 `Unavailable`。

取消为 `Cancelled`；截止时间已过为 `DeadlineExceeded`；程序不变量破坏为 `Invariant`；无法安全归入前述分类的内部故障为 `Internal`。

`XError` 必须保持不透明。调用方通过 `kind()`、`context()`、`retry_after()`、`is_retryable()` 和 `is_bug()` 查询，不得匹配内部表示或 Display 字符串。

每个 `ErrorKind` 必须有同名构造器。`Transient` 另有 `transient_after`；`with_source` 必须保留分类和 source chain。

仅 `Transient` 的 `is_retryable()` 为 `true`；仅 `Invariant` 的 `is_bug()` 为 `true`。`retry_after` 只对显式携带建议的瞬时错误返回值。

禁止 `not_found`、`other`、通用字符串到 `XError` 的 `From` 实现，以及允许绕过具名构造器的通用分类构造入口。

`ClockError` 转为 `XError` 时归类 `Unavailable` 并保留 source。时间源失败不得映射为 `Invalid`，也不得返回零值时间戳。

## 4. `clock` 合同

### 4.1 `Timestamp`

`Timestamp` 是 `i64` Unix epoch 纳秒。它可表示 epoch 前时间，但 `SystemClock::now` 在系统时间早于 epoch 时返回 `ClockError::BeforeUnixEpoch`。

```rust
pub struct Timestamp(i64);

impl Timestamp {
    pub const fn from_unix_nanos(nanos: i64) -> Self;
    pub const fn as_unix_nanos(self) -> i64;
    pub fn checked_add(self, duration: Duration) -> Option<Self>;
    pub fn checked_sub(self, duration: Duration) -> Option<Self>;
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration>;
}
```

checked 算术必须覆盖完整 `i64` 纳秒域。溢出或反向差返回 `None`，不得 panic、饱和或把反向差伪装成 `Duration::ZERO`。

`Timestamp` 不实现 `Default`、serde 或人类时间格式化。显式纳秒构造仅用于协议边界、fixture 与 testkit。

### 4.2 `ClockDomain` 与 `MonotonicInstant`

```rust
pub struct ClockDomain(u64);

impl ClockDomain {
    pub const PROCESS: Self;
    pub const fn from_raw(id: u64) -> Self;
    pub const fn as_raw(self) -> u64;
}

pub struct MonotonicInstant { /* private */ }
```

`ClockDomain` 标识单调采样点的比较域。`PROCESS` 是当前进程的系统时钟域标签；其 raw 值不是跨进程身份，进程重启后的采样点不得比较。

`from_raw` 不分配唯一值。自定义时钟必须保证 ID 不与 `PROCESS` 或其他存活时钟冲突；不得依靠碰撞把不同来源伪装为同一 domain。

所有 `SystemClock` 实例共享同一个进程 origin，并产生 `ClockDomain::PROCESS` 中的采样点。因此，同一进程内不同 `SystemClock` 实例的采样点可比较。

testkit 或仿真时钟必须使用独立 domain。跨 domain 的 `partial_cmp` 与 `checked_duration_since` 返回 `None`，不得形成可靠顺序或间隔。

同 domain 内，反向 `checked_duration_since` 返回 `None`；相等采样点返回 `Some(Duration::ZERO)`。单调点绝对值无业务意义，不得持久化或序列化。

以下构造 seam 是经批准的公开 ABI，但必须 `#[doc(hidden)]`：

```rust
impl MonotonicInstant {
    pub const fn domain(self) -> ClockDomain;
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration>;

    #[doc(hidden)]
    pub const fn from_clock_elapsed(elapsed: Duration) -> Self;

    #[doc(hidden)]
    pub const fn from_clock_elapsed_in(
        elapsed: Duration,
        domain: ClockDomain,
    ) -> Self;
}
```

`from_clock_elapsed` 使用 `ClockDomain::PROCESS`；`from_clock_elapsed_in` 接受显式 domain。两者只供 `kernel` 的 `Clock` 实现和 `testkit::ManualClock` 使用。

隐藏文档不等于私有 API。任何改名、删除或放宽调用边界仍按公开 API 变更治理。业务代码不得直接调用这两个构造 seam。

### 4.3 `Clock`、`ClockError` 与 `SystemClock`

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Result<Timestamp, ClockError>;
    fn monotonic(&self) -> MonotonicInstant;
}

#[non_exhaustive]
pub enum ClockError {
    BeforeUnixEpoch,
    Overflow,
    Unavailable,
}

pub struct SystemClock;
```

`Clock::monotonic` 不得有默认实现。每个时钟必须同时明确实现墙钟和单调钟；墙钟允许回退，deadline 和间隔必须使用单调时间。

`SystemClock::now` 使用 `SystemTime::now`，并对 epoch 前和 `i64` 纳秒溢出返回 typed error，不得返回哨兵值。

进程单调 origin 由 `OnceLock<Instant>` 或等价的一次初始化机制共享。`SystemClock::monotonic` 计算共享 origin 的 elapsed，并写入 `ClockDomain::PROCESS`。

禁止为每个 `SystemClock` 建立独立 origin，也禁止每次采样重建 origin。`SystemClock` 可 `Clone` 和 `Default`，不承担 sleep、timeout 或 interval。

除下一节明确批准的协调等待外，生产业务取时必须经注入的 `Clock`；`SystemTime::now` 与 `Instant::now` 不得散落到其他生产路径。

## 5. `lifecycle` 合同

### 5.1 状态机

`ComponentState` 包含 `Created`、`Starting`、`Running`、`Draining`、`Stopped` 与 `Failed`。状态转换接口为：

```rust
impl ComponentState {
    pub const fn can_transition_to(self, to: ComponentState) -> bool;
    pub fn try_transition(self, to: ComponentState)
        -> Result<ComponentState, LifecycleError>;
}

pub struct LifecycleError {
    pub from: ComponentState,
    pub to: ComponentState,
}
```

合法转换仅有：

```text
Created  → Starting
Starting → Running | Failed
Running  → Draining | Failed
Draining → Stopped | Failed
```

`Stopped` 和 `Failed` 是终态。非法转换由 `LifecycleError { from, to }` 返回，不得 panic。

### 5.2 关停信号

`ShutdownSignal` 可克隆，`ShutdownGuard` 不可克隆。`ShutdownGuard::trigger(self)` 消费唯一触发入口；触发一次后不可重置。

实现使用同一 `Mutex<bool>` 与 `Condvar`。触发路径必须持锁写入 `true` 并 `notify_all`；等待路径必须在 `while !triggered` 循环内等待，以抵抗伪唤醒和 lost wake-up。

trigger 前已等待、trigger 后开始等待、以及 signal clone 都必须观察到相同的不可逆状态。直接 drop guard 不自动触发、不 panic、不记录日志。

### 5.3 有限协调等待

标准库构建提供：

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WaitTimeoutError {
    DeadlineOverflow,
}

impl ShutdownSignal {
    #[cfg(not(loom))]
    pub fn wait_timeout(
        &self,
        timeout: Duration,
    ) -> Result<bool, WaitTimeoutError>;
}
```

返回语义固定为：

```text
Ok(true)  = 已观察到关停触发；
Ok(false) = 可表示的 deadline 到期，且仍未触发；
Err(DeadlineOverflow) = 当前 std::time::Instant 无法表示 deadline。
```

deadline 可表示且调用时已经触发时，必须返回 `Ok(true)`。零时长在未触发时返回 `Ok(false)`。

持锁后的首次完成状态检查必须先于 deadline 构造；已触发立即返回 `Ok(true)`。仅在未触发时，以单次 `Instant::now().checked_add(timeout)` 建立 deadline；不可表示时必须返回 typed error，禁止回退到“当前时刻”并伪装成普通 timeout。

循环每次按固定 deadline 计算剩余时长，抵抗伪唤醒。锁中毒按既有策略恢复 guard，但不得改变触发状态或返回含糊结果。

`wait_timeout` 是 `Clock` 收口规则的窄例外：它只协调当前进程线程，不生成业务时间，不暴露采样点，也不接受可注入业务时钟。

该 API 仅在 `cfg(not(loom))` 存在，因为 loom 的 `Condvar` 不提供等价 `wait_timeout`。无超时的 wait/trigger 协议仍必须由 loom 证明。

## 6. 公开 API 冻结面

crate 根必须导出：

```rust
pub mod clock;
pub mod error;
pub mod lifecycle;

pub use clock::{
    Clock, ClockDomain, ClockError, MonotonicInstant, SystemClock, Timestamp,
};
pub use error::{BoxError, ErrorKind, XError, XResult};
pub use lifecycle::{
    ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal,
    WaitTimeoutError,
};
```

模块内公开构造器与方法以第 3–5 节为准。新增公开项、依赖、feature 或职责必须有 Approved RFC、下游影响分析、API diff 和迁移方案。

公开类型默认不实现 serde。需要 wire 或持久化时，由协议层定义版本化 DTO 并显式转换。

## 7. 测试合同

### 7.1 错误与状态

必须覆盖九类错误的构造、查询、source chain、retry/bug 判定，以及全部 `ComponentState` 二元转换矩阵。

### 7.2 时间

必须覆盖 `Timestamp` 的 `i64::MIN`、`i64::MAX`、完整跨度、溢出、反向差和 property test。

必须覆盖同一进程内两个独立 `SystemClock` 的 domain 相同且采样点可比较；不同 test domain 的比较和差值均为 `None`。

必须覆盖隐藏构造 seam 的默认进程 domain、显式 domain 和反向差。业务调用边界由结构扫描、API 审查与 testkit 合同共同约束。

### 7.3 关停

标准库测试必须覆盖 trigger-before-wait、wait-before-trigger、多 observer、guard drop、锁中毒和并发回归。

`wait_timeout` 必须覆盖未触发的常规超时 `Ok(false)`、超时前触发 `Ok(true)`、已触发立即返回、零时长，以及未触发时 `Duration::MAX` 返回 `Err(WaitTimeoutError::DeadlineOverflow)`。若信号在调用前已经触发，完成状态优先，必须在构造 deadline 前返回 `Ok(true)`；不可表示的 timeout 不得覆盖已经发生的完成事实。

Duration 不可表示测试不得只断言“不 panic”；它必须断言 typed error，防止 deadline overflow 被误报为普通 timeout。

loom 必须覆盖 wait/trigger 竞争、多 waiter 与 trigger 后观察。loom 不负责 `wait_timeout`，因为该 API 在 `cfg(loom)` 下不存在。

### 7.4 编译与公开面

rustdoc `compile_fail` 与 static assertion 必须验证：关键类型无 `Default`/serde、guard 不可 Clone、无 `Component` trait、字段不可访问及禁止的模糊错误构造。

doctest 必须单独执行，因为 `--all-targets` 不包含 doctest。公开 API 测试和基线必须包含 `ClockDomain`、两个隐藏构造 seam、`wait_timeout` 新签名与 `WaitTimeoutError`。

## 8. 验证与门禁

交付至少执行：

```bash
cargo fmt --all -- --check
cargo clippy -p kernel --all-targets -- -D warnings
cargo test -p kernel --all-targets
cargo test -p kernel --doc
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

coverage、Miri 和 mutation 是否为本轮硬门禁，以仓库当前 CI 接线为准；未运行不得写 PASS。archgate 与 `.architecture/**` 对 infra.rs 为 OOS，不得作为验收条件。

公开 API 基线必须从当前源码生成或核对。源码字符串扫描可作为辅助门禁，不得替代编译、doctest 或 API 工具。

## 9. 版本、状态与证据

当前事实是 package `kernel`、lib `kernel`、version `0.3.1`、`publish = false`。历史 `xhyper-kernel 0.1.1` 与 crates.io 事件只属于历史记录，不是当前分发合同。

本轮行为变更交付采用 patch bump。版本 bump 不自动授权破坏性变更；公开面变化仍必须更新 CHANGELOG、API 基线、下游调用和本组 SSOT。

`Approved` 表示合同获批；L1 Internal Ready 表示内部使用准备度；L4 只覆盖新鲜证据已证明的支持面。这些状态均不等于 production certification。

历史 `.agents/ssot/kernel/evidence/2026-07-14/` 是不可变快照。它可以解释历史决策，但不能作为当前 commit 的 fresh PASS。

当前 PASS 只能来自绑定被验收 commit、工具链、命令和结果的新鲜证据。SKIP、旧日志或手写结论不得计为 PASS。

## 10. 完成定义

交付完成必须同时满足：

- 源码、测试、根导出、API 基线和本组 SSOT 一致；
- `ClockDomain`、共享进程 origin 与隐藏构造 seam 的合同被测试；
- `wait_timeout` 不可表示 deadline 返回 typed error，不再伪装普通 timeout；
- 常规 timeout、trigger、跨 domain、doctest、loom 与公开 API 合同通过；
- 版本按 patch-default 同步，且无 crates.io 或 production-certified 误导；
- gate 只记录本次实际运行的新鲜结果。
