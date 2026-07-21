# `kernel` Crate 完整规范

```text
Spec ID:        SPEC-KERNEL-002
Title:          xhyper.rs L0 Kernel Runtime Semantics
Status:         Approved
Target Status:  Stable
Owner:          platform
Physical Path:  crates/kernel
Package:        xhyper-kernel
Lib:            kernel
Layer:          L0 / Kernel
Publish:        true (crates.io package: xhyper-kernel)
Current Version: 0.1.1
Target Version:  0.1.1
Source Goal:    GOAL-KERNEL-RUNTIME-SEMANTICS
Supersedes:     SPEC-KERNEL-001
```

---

## 0. 文档定位

本文件是 `kernel` crate 的实现、测试、演进和机器门禁的唯一规范。

状态必须按维度解释：`Approved` 是规范状态，architecture registry 的 `stable` 是
API 生命周期状态，`publish = true` 是分发策略。三者均不等于当前提交已经取得生产
认证；生产认证必须绑定当前 commit 的新鲜、完整 Evidence。本规范当前不作
`production-certified` 声明。

`kernel` 是整个 workspace 的语义信任根。它不提供业务能力，也不承担依赖注入、持久化、配置、观测、网络或异步运行时职责。它只定义全系统必须唯一且长期稳定的三类语义：

1. 错误如何被分类和响应；
2. 时间如何被获取、表示和比较；
3. 组件如何表达生命周期状态和关停信号。

本文件获批后：

- `crates/kernel` 的代码必须与本文件一致；
- `.architecture/`、`archgate`、CI、测试和 API 快照必须以本文件为输入；
- 任何新增公开项、依赖、feature 或职责都必须走 RFC；
- 当前旧规范、README、AGENTS、ADR 中与本文件冲突的内容必须同步修订。

---

## 1. 设计原则

### 1.1 Kernel 的本质

`kernel` 提供的不是“常用功能”，而是全系统被迫共享的语义。

一个能力只有同时满足以下四个条件，才允许进入 `kernel`：

```text
1. 全系统必须只有一种语义；
2. 两种并存会导致错误响应、时间失真或关停失控；
3. 不属于任何单一业务、领域、适配器或基础设施能力；
4. 语义预计以年为单位保持稳定。
```

任何不满足全部四项的内容，必须留在：

- `crates/types/*`
- `crates/contracts`
- `crates/*`
- `crates/adapters/*`
- `crates/domain/*`
- `crates/testkit`
- `apps/*`
- `tools/*`

### 1.2 Kernel 的非目标

以下内容永久禁止进入 `kernel`：

```text
- 配置加载、环境变量、文件系统；
- 日志、指标、追踪、OpenTelemetry；
- 网络、数据库、消息队列、序列化协议；
- tokio、async-std 或任何特定异步运行时；
- 依赖注入、Service Locator、插件注册；
- 健康检查、重试、熔断、限流、调度；
- 领域 ID、订单、持仓、行情、资金、账户；
- serde wire 表示；
- 全局单例和隐式默认 Clock；
- test mock、fixture、fake 实现；
- 为兼容遗留代码而长期保留的语义别名。
```

### 1.3 最小公开面原则

`kernel` 的健康指标不是功能数量，而是：

```text
- 公开 API 长期不变；
- 内部依赖为零；
- 外部生产依赖最小；
- 下游不需要例外；
- 无隐式时间源；
- 无字符串约定模拟错误分类；
- 关停行为可证明且无竞态。
```

---

## 2. 目录结构

```text
crates/kernel/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── clock.rs
│   └── lifecycle.rs
└── tests/
    ├── api_compile.rs
    ├── clock_contract.rs
    ├── lifecycle_concurrency.rs
    └── public_api.rs
```

禁止新增 `util.rs`、`common.rs`、`prelude.rs`、`helpers.rs` 等收容型模块。

新增模块必须先修改本规范并通过 RFC。

---

## 3. 依赖合同

### 3.1 Workspace 内部依赖

```text
kernel → ∅
```

`kernel` 不得依赖任何其他 workspace crate，包括：

- `contracts`
- `canonical`
- `decimalx`
- `testkit`
- `evidence`
- `gate`
- `bootstrap`
- 任意 domain、adapter、service 或 app

此规则无例外通道。

### 3.2 外部生产依赖白名单

```toml
[dependencies]
thiserror = { workspace = true }
```

生产依赖白名单到此为止。

明确禁止：

```text
anyhow
serde
serde_json
tokio
async-std
futures
chrono
time
tracing
log
parking_lot
once_cell
uuid
rand
bytes
```

需要新增外部依赖时，必须证明：

1. `std` 无法实现；
2. 依赖不会进入公开签名；
3. 依赖不会引入运行时或 wire 耦合；
4. 依赖新增收益高于信任根复杂度；
5. 已完成供应链、许可证和 MSRV 审计。

### 3.3 测试依赖

`proptest` 与 `static_assertions` 是 dev-dependency。编译负向合同由 rustdoc
`compile_fail` 与 static assertion 互补证明。`loom` 是 `cfg(loom)` target
dependency，只在显式 `RUSTFLAGS='--cfg loom'` 的模型测试构建中链接；默认生产图
不得包含它。

测试依赖不得进入默认生产依赖图或公开 API。依赖门禁必须解析完整 Cargo metadata，
包括 target/build dependency，不能只扫描 `[dependencies]` 文本。

### 3.4 Features

```toml
[features]
default = []
```

`kernel` 不允许任何 feature。

需要 feature 的能力通常不属于 L0。

---

## 4. Crate 级属性

`src/lib.rs` 必须至少包含：

```rust
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
```

并继承 workspace lints：

```toml
[lints]
workspace = true
```

禁止：

- `unsafe`
- `todo!`
- `unimplemented!`
- 生产代码中的 `panic!`
- 生产代码中的 `unwrap()` / `expect()`
- 静默默认值兜底
- 锁中毒时返回伪造成功值

---

# 5. `error` 模块规范

## 5.1 设计原则

错误必须按“调用方应该如何反应”分类，而不是按“错误来自哪个模块”分类。

调用方不得通过字符串匹配决定是否重试、降级或 fail-fast。

错误系统必须满足：

```text
- 分类稳定；
- source chain 可保留；
- 不依赖 anyhow；
- 可跨线程传递；
- 不要求 Clone；
- Display 面向人类，不是协议；
- Debug 不泄漏敏感业务数据；
- 未知错误使用率只减不增。
```

## 5.2 公开类型

```rust
pub type BoxError =
    Box<dyn std::error::Error + Send + Sync + 'static>;

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

pub struct XError {
    kind: ErrorKind,
    context: std::borrow::Cow<'static, str>,
    retry_after: Option<std::time::Duration>,
    source: Option<BoxError>,
}

impl std::fmt::Debug for XError { /* 与 Display 一样不格式化 source */ }
```

`XError` 的字段必须保持私有。

调用方只能通过构造器和查询方法使用错误语义。

## 5.3 ErrorKind 精确定义

### `Invalid`

请求、参数或输入本身非法。

```text
反应：
- 不自动重试；
- 修正输入后可重新提交；
- 不表示系统故障。
```

示例：

- 非法 symbol；
- JSON 字段缺失；
- 时间戳格式错误；
- 参数越界。

### `Missing`

请求的实体、资源或已声明依赖不存在。

```text
反应：
- 不立即自动重试；
- 调用方可选择 fallback；
- 若该实体按系统不变量必须存在，应在更高层转换为 Invariant。
```

示例：

- 未找到指定配置项；
- requested order ID 不存在；
- 启动期所需 capability 缺失。

### `Conflict`

输入本身合法，但与当前状态冲突。

```text
反应：
- 不按瞬时故障自动重试；
- 只有状态变化后重试才有意义。
```

示例：

- 重复注册；
- 已关闭对象再次启动；
- 乐观锁版本冲突。

### `Transient`

暂时性失败，保持相同语义的重试可能成功。

```text
反应：
- 可使用退避和抖动重试；
- retry_after 仅为提示，不是承诺。
```

示例：

- 短暂网络抖动；
- 瞬时限流；
- 短暂锁争用。

### `Unavailable`

下层依赖或必要基础能力不可用。

```text
反应：
- 默认传播；
- 由 lifecycle / composition 决定降级或 fail-fast；
- 不等同于普通 Transient。
```

示例：

- 系统时钟不可用；
- 必需存储离线；
- 启动依赖失败。

### `Cancelled`

操作被调用方或系统取消。

```text
反应：
- 不自动重试；
- 不记录为内部故障；
- 上层可将其视为正常终止路径。
```

### `DeadlineExceeded`

操作未在调用方给定的 deadline 内完成。

```text
反应：
- 本次操作终止；
- 是否重试由上层策略裁定；
- kernel 不把它自动标记为 retryable。
```

### `Invariant`

内部不变量、前置条件或不可发生状态被破坏。

```text
反应：
- 不重试；
- 视为 bug；
- 必须进入错误预算、告警或受控 fail-fast。
```

### `Internal`

暂时无法归入以上类别的内部错误。

```text
反应：
- 不自动重试；
- 必须进入使用量棘轮；
- 新增调用点必须附迁移计划；
- 长期目标是趋近于零。
```

## 5.4 构造器

必须提供：

```rust
impl XError {
    pub fn invalid(context: impl Into<String>) -> Self;
    pub fn missing(context: impl Into<String>) -> Self;
    pub fn conflict(context: impl Into<String>) -> Self;
    pub fn transient(context: impl Into<String>) -> Self;
    pub fn transient_after(
        context: impl Into<String>,
        retry_after: Duration,
    ) -> Self;
    pub fn unavailable(context: impl Into<String>) -> Self;
    pub fn cancelled(context: impl Into<String>) -> Self;
    pub fn deadline_exceeded(context: impl Into<String>) -> Self;
    pub fn invariant(context: impl Into<String>) -> Self;
    pub fn internal(context: impl Into<String>) -> Self;

    pub fn with_source(
        self,
        source: impl Into<BoxError>,
    ) -> Self;

    pub fn kind(&self) -> ErrorKind;
    pub fn context(&self) -> &str;
    pub fn retry_after(&self) -> Option<Duration>;

    pub fn is_retryable(&self) -> bool;
    pub fn is_bug(&self) -> bool;
}
```

语义：

```text
is_retryable() == true 仅适用于 Transient
is_bug()       == true 仅适用于 Invariant
```

`DeadlineExceeded` 是否重试由上层决定，kernel 不替上层选择。

## 5.5 禁止 API

禁止提供：

```rust
impl From<String> for XError
impl From<&str> for XError
impl From<anyhow::Error> for XError
pub fn other(...)
pub fn not_found(...)
```

理由：

- `From<String>` 会绕过显式分类；
- `anyhow` 会回流到信任根；
- `other` 会成为永久垃圾桶；
- `not_found → Invalid` 会制造语义谎言。

## 5.6 source chain

`with_source` 必须保持原有 `ErrorKind` 不变。

禁止：

- 附加 source 后自动改成 `Internal`；
- 丢弃原 source；
- 在 Display 中输出完整 source chain；
- 在 Debug 中输出 source 细节；
- 把敏感输入写入 context。

`std::error::Error::source()` 仍保留因果链供显式诊断调用；常规 `Display`/`Debug`
不得隐式展开它。调用方仍必须保证 `context` 本身已经脱敏。

## 5.7 ClockError 映射

必须实现：

```rust
impl From<ClockError> for XError
```

映射：

```text
ClockError::* → ErrorKind::Unavailable
```

时间源失败不得映射为 `Invalid` 或返回零值。

## 5.8 错误治理

`archgate` 必须维护：

```text
- XError::internal 使用点数量；
- 每个调用点 owner；
- 创建日期；
- 到期日期；
- 替代分类计划。
```

基线只允许下降，不允许无审批增加。

---

# 6. `clock` 模块规范

## 6.1 设计原则

```text
1. 墙钟和单调钟是不同语义；
2. 获取失败必须显式返回错误；
3. 不允许零值时间戳哨兵；
4. 不允许全局隐式 Clock；
5. 所有时间源必须显式注入；
6. 测试必须能够独立控制墙钟和单调钟；
7. 时间计算不得 panic、饱和或静默失真。
```

## 6.2 Timestamp

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct Timestamp(i64);
```

语义：

```text
- Unix epoch 纳秒；
- i64 表示；
- 无 Default；
- 无 serde；
- 无人类时间格式化；
- 可表示 epoch 前时间，但 SystemClock 不接受当前时间早于 epoch；
- checked 运算覆盖完整 `i64` 纳秒域，并使用宽于 i64 的中间值避免误报溢出；
- 显式构造仅用于 protocol 转换、fixture 和 testkit。
```

必须提供：

```rust
impl Timestamp {
    pub const fn from_unix_nanos(nanos: i64) -> Self;
    pub const fn as_unix_nanos(self) -> i64;

    pub fn checked_add(
        self,
        duration: Duration,
    ) -> Option<Self>;

    pub fn checked_sub(
        self,
        duration: Duration,
    ) -> Option<Self>;

    pub fn checked_duration_since(
        self,
        earlier: Self,
    ) -> Option<Duration>;
}
```

禁止提供：

```text
Default
Serialize / Deserialize
From<SystemTime>
Display 人类时间格式
饱和加减
隐式单位转换
from_nanos / as_nanos 简写别名
```

## 6.3 MonotonicInstant

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct MonotonicInstant(/* private */);
```

语义：

```text
- 只用于测量间隔；
- 绝对值无业务意义；
- 不可持久化；
- 不可跨进程比较；
- 不可跨不同 Clock 实例比较；
- 不提供公开绝对 ticks getter。
```

必须提供：

```rust
impl MonotonicInstant {
    pub fn checked_duration_since(
        self,
        earlier: Self,
    ) -> Option<Duration>;
}
```

反向比较必须返回 `None`，禁止饱和为 `Duration::ZERO`。

为支持 `Clock` 实现和 `testkit::ManualClock`，允许提供一个明确标记的构造入口：

```rust
#[doc(hidden)]
pub const fn from_clock_elapsed(
    elapsed: Duration,
) -> Self;
```

`archgate` 必须限制其调用位置只能出现在：

```text
crates/kernel/src/clock.rs
crates/testkit/*
```

由 **KERNEL-TIME-004** 机控（`FROM_CLOCK_ELAPSED_ALLOW` allowlist；kernel_rules.rs:156-320）。

## 6.4 Clock trait

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Result<Timestamp, ClockError>;
    fn monotonic(&self) -> MonotonicInstant;
}
```

强制要求：

- `monotonic` 不得有默认实现；
- 每个 Clock 实现必须同时明确实现墙钟和单调钟；
- 墙钟允许因 NTP 或人工校时回退，不承诺非递减；
- 排序 deadline 与测量间隔必须使用单调钟，不得使用墙钟差值；
- 调用方不得直接调用 `SystemTime::now()` 或 `Instant::now()`；
- `Clock` 不提供 sleep、timeout、interval；
- 异步时间由上层运行时 adapter 实现。

## 6.5 ClockError

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClockError {
    #[error("system clock is before Unix epoch")]
    BeforeUnixEpoch,

    #[error("clock value exceeds representable nanoseconds")]
    Overflow,

    #[error("time source unavailable")]
    Unavailable,
}
```

禁止返回默认时间戳代替错误。

## 6.6 SystemClock

```rust
#[derive(Debug, Clone)]
pub struct SystemClock {
    origin: std::time::Instant,
}
```

必须提供：

```rust
impl SystemClock {
    pub fn new() -> Self;
}

impl Default for SystemClock {
    fn default() -> Self;
}

impl Clock for SystemClock {
    fn now(&self) -> Result<Timestamp, ClockError>;
    fn monotonic(&self) -> MonotonicInstant;
}
```

实现要求：

```text
now():
  SystemTime::now()
  → duration_since(UNIX_EPOCH)
  → u128 nanos
  → checked i64
  → Timestamp

monotonic():
  self.origin.elapsed()
  → MonotonicInstant
```

禁止：

```text
- SystemClock 为 Copy；
- 失败时返回 0；
- 使用 chrono/time；
- 使用全局静态 Clock；
- 每次 monotonic() 建立新 origin。
```

## 6.7 时间 API 收口

生产代码中以下 API 只允许出现在 `SystemClock` 实现：

```text
SystemTime::now
Instant::now
```

允许的例外：

```text
- crates/testkit 中的测试时钟实现；
- 明确登记、带到期时间的迁移例外。
```

`std::thread::sleep` 不得用于 kernel 单元测试证明时钟正确性。

---

# 7. `lifecycle` 模块规范

## 7.1 设计原则

```text
1. lifecycle 只提供共同语言和关停原语；
2. 不提供启动编排；
3. 不提供健康检查；
4. 不提供自动重启；
5. 不依赖 tokio；
6. 关停必须一次触发、多方观察、不可逆；
7. 阻塞等待不得存在 lost wake-up。
```

## 7.2 ComponentState

```rust
#[non_exhaustive]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
pub enum ComponentState {
    Created,
    Starting,
    Running,
    Draining,
    Stopped,
    Failed,
}
```

合法转换：

```text
Created  → Starting
Starting → Running
Starting → Failed
Running  → Draining
Running  → Failed
Draining → Stopped
Draining → Failed
```

其他转换一律非法。

## 7.3 LifecycleError

```rust
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    thiserror::Error,
)]
#[error(
    "illegal component state transition: {from:?} -> {to:?}"
)]
pub struct LifecycleError {
    pub from: ComponentState,
    pub to: ComponentState,
}
```

## 7.4 状态转换 API

```rust
impl ComponentState {
    pub const fn can_transition_to(
        self,
        to: ComponentState,
    ) -> bool;

    pub fn try_transition(
        self,
        to: ComponentState,
    ) -> Result<ComponentState, LifecycleError>;
}
```

状态转换不得 panic。

## 7.5 ShutdownSignal / ShutdownGuard

```rust
#[must_use]
#[derive(Clone, Debug)]
pub struct ShutdownSignal {
    inner: Arc<ShutdownInner>,
}

#[must_use]
#[derive(Debug)]
pub struct ShutdownGuard {
    inner: Arc<ShutdownInner>,
}
```

必须提供：

```rust
impl ShutdownSignal {
    pub fn new() -> (ShutdownGuard, ShutdownSignal);
    pub fn is_triggered(&self) -> bool;
    pub fn wait(&self);
}

impl ShutdownGuard {
    pub fn trigger(self);
}
```

语义：

```text
- guard 不可 Clone；
- signal 可 Clone；
- trigger 消费 guard；
- trigger 后所有已阻塞 waiter 被唤醒；
- trigger 前创建、trigger 后创建的 observer 都能观察到状态；
- signal 不可重置；
- wait 已触发时立即返回；
- 锁中毒不得伪装成“未触发”。
```

## 7.6 并发实现约束

推荐实现：

```rust
struct ShutdownInner {
    triggered: Mutex<bool>,
    cv: Condvar,
}
```

`wait`：

```text
1. 获取 mutex；
2. while !triggered:
3.     cv.wait；
4. triggered 后返回。
```

`trigger`：

```text
1. 获取同一 mutex；
2. 设置 triggered = true；
3. notify_all；
4. 释放 mutex。
```

禁止以下实现：

```text
- waiter 使用 mutex，trigger 不获取同一 mutex；
- 仅依赖 AtomicBool + notify_all；
- 用 sleep 轮询；
- 用 SeqCst 掩盖错误的 Condvar 协议；
- 锁中毒时返回 false；
- lost wake-up 风险未经过 loom 验证。
```

## 7.7 Guard Drop 语义

`ShutdownGuard` 被直接 drop 时：

```text
- 不自动触发 shutdown；
- 不 panic；
- 不记录日志；
- 由组合根保证 guard 生命周期。
```

原因：kernel 不应把普通所有权结束隐式解释为系统关停。

上层 `bootstrap` 必须保证：

- guard 在运行期由明确 owner 持有；
- OS signal 路径显式调用 `trigger`；
- 不允许在构建完成时意外 drop guard。

## 7.8 不提供 Component trait

本版本不公开通用 `Component` trait。

原因：

```text
- 当前没有至少两个真实生产实现证明抽象稳定；
- start/stop/drain 往往涉及 async、deadline 和领域副作用；
- 过早冻结 trait 会把 composition 语义错误地下沉到 L0。
```

只有在满足以下条件后才能通过 RFC 增加：

```text
1. 至少两个独立生产组件存在；
2. 两者 API 可抽象为相同同步语义；
3. 不引入 async runtime 类型；
4. 不承担依赖排序和编排；
5. 有完整迁移和兼容性分析。
```

---

# 8. 公开 API 冻结面

`src/lib.rs` 的公开导出必须严格限制为：

```rust
pub mod clock;
pub mod error;
pub mod lifecycle;

pub use clock::{
    Clock,
    ClockError,
    MonotonicInstant,
    SystemClock,
    Timestamp,
};

pub use error::{
    BoxError,
    ErrorKind,
    XError,
    XResult,
};

pub use lifecycle::{
    ComponentState,
    LifecycleError,
    ShutdownGuard,
    ShutdownSignal,
};
```

公开项总数以此为上限基线。

禁止：

- `prelude`;
- 第三方 crate re-export；
- 宏；
- test mock；
- 全局实例；
- 运行时特定类型；
- 未经 RFC 的新增公开构造器。

---

# 9. Serde、Wire 与持久化政策

所有 kernel 类型默认：

```text
- 不实现 Serialize；
- 不实现 Deserialize；
- 不定义 wire schema；
- 不定义数据库表示；
- 不定义 JSON 格式；
- 不定义人类时间格式。
```

需要传输或持久化时，由 `types/canonical` 或协议层定义：

```rust
pub struct TimestampWireV1 {
    pub unix_nanos: i64,
}
```

并显式实现：

```rust
From<Timestamp> for TimestampWireV1
TryFrom<TimestampWireV1> for Timestamp
```

`ComponentState`、`ErrorKind` 的 wire 表示同样由协议层显式版本化，不直接 serde kernel enum。

---

# 10. Panic 与失败策略

## 10.1 普通路径

以下情况必须返回 `Result` 或 `Option`，不得 panic：

```text
- 时间溢出；
- epoch 前系统时间；
- 非法生命周期转换；
- 反向时间差；
- 锁中毒；
- 普通输入错误。
```

## 10.2 锁中毒

对于 `ShutdownInner` 的单一布尔状态：

```text
- 使用 poisoned.into_inner() 恢复；
- 恢复后继续遵守同一状态机；
- 不返回虚假默认值；
- 不把 poison 作为对外 panic 合同。
```

## 10.3 不变量错误

kernel 自身不主动 `panic!` 报告 `Invariant`。

调用方通过：

```rust
return Err(XError::invariant(...));
```

上层 composition 决定是否 fail-fast。

---

# 11. 测试合同

## 11.1 单元测试

### error

必须覆盖：

```text
- 每个 ErrorKind 的构造器；
- kind/context/retry_after；
- with_source 不改变 kind；
- is_retryable 仅 Transient 为 true；
- is_bug 仅 Invariant 为 true；
- Display 不包含 source 细节；
- Internal 使用点不通过模糊构造器新增。
```

### Timestamp

必须覆盖：

```text
- i64::MIN / i64::MAX 边界；
- checked_add 溢出；
- checked_sub 溢出；
- 相等时间差为 Some(Duration::ZERO)；
- earlier > self 返回 None；
- 大于 u64 纳秒可转换边界；
- 无 Default 的编译失败测试。
```

### Clock

必须覆盖：

```text
- SystemClock::now 返回可表示时间；
- SystemClock monotonic 非递减；
- Clock trait 没有 monotonic 默认实现；
- 时间错误映射为 XError::Unavailable；
- ManualClock 可独立控制 wall/monotonic。
```

### lifecycle

必须覆盖：

```text
- 全部合法转换；
- 全部非法转换；
- trigger-before-wait；
- wait-before-trigger；
- 多 observer；
- trigger 后新 observer 立即可见；
- 1000 次并发回归；
- poison recovery；
- guard 不可 Clone；
- signal 可 Clone；
- guard drop 不触发。
```

## 11.2 Loom 模型测试

必须使用 loom 覆盖：

```text
- waiter 检查状态与进入 park 的竞争窗口；
- trigger 与多个 waiter 并发；
- trigger 已完成后 observer 读取；
- 不存在 lost wake-up；
- 不存在永久阻塞状态。
```

在 loom 测试通过前，不得宣称 ShutdownSignal 并发正确。

## 11.3 Property testing

必须使用 property test 覆盖：

```text
- 任意 i64 Timestamp 与任意 Duration 的 checked 运算；
- 任意 ComponentState 二元组合的转换矩阵；
- XError 构造器分类一致性。
```

## 11.4 Compile-fail 测试

使用 trybuild 或等价机制验证：

```text
- Timestamp: !Default；
- MonotonicInstant: !Default；
- ShutdownGuard: !Clone；
- kernel 类型无 serde derive；
- kernel 不导出 Component trait；
- 下游不能访问私有字段。
```

## 11.5 覆盖率

```text
line coverage      >= 95%
branch coverage    >= 90%
```

覆盖率必须由 CI 强制，不能只作为本地 recipe。

## 11.6 Mutation testing

```text
mutation score >= 90%
```

以下变异不得存活：

```text
- is_retryable 反转；
- is_bug 反转；
- Timestamp 边界判断反转；
- lifecycle 合法转换删除/新增；
- wait while 改为 if；
- trigger 删除 notify_all；
- trigger 不持锁；
- 时间错误返回零值；
- monotonic 反向差饱和为零。
```

执行策略：

```text
PR:
  变更命中 crates/kernel 时执行受影响 mutation set

Nightly:
  cargo mutants -p kernel 全量执行
```

## 11.7 Miri

Nightly 或定期执行：

```text
cargo miri test -p kernel
```

虽然禁止 unsafe，Miri 仍用于检查标准库交互和测试代码未定义行为。

---

# 12. CI 与机器门禁

## 12.1 必须通过的命令

```bash
cargo fmt -- --check
cargo clippy -p kernel --all-targets -- -D warnings
cargo test -p kernel --all-features
cargo llvm-cov -p kernel --fail-under-lines 95
cargo run -p xhyper-archgate -- --json
cargo xtl lint-deps
cargo xtl crate-standard --check
cargo semver-checks check-release -p xhyper-kernel
```

`kernel` 没有 feature，但统一命令允许保留 `--all-features`。

## 12.2 archgate 规则

必须机器强制：

```text
KERNEL-DEP-001:
  内部 workspace dependency 数量必须为 0。

KERNEL-DEP-002:
  生产外部依赖只能是 thiserror。

KERNEL-FEATURE-001:
  Cargo.toml 除 default=[] 外不得存在 feature。

KERNEL-API-001:
  公开 API 必须与冻结清单一致。

KERNEL-API-002:
  新增公开项未关联 Approved RFC → fail。

KERNEL-PUBLISH-001:
  Cargo、architecture registry 与本 Spec 的 publish 值不一致 → fail。

KERNEL-TIME-001:
  kernel 之外生产代码出现 SystemTime::now → fail。

KERNEL-TIME-002:
  kernel 之外生产代码出现 Instant::now → fail。

KERNEL-TIME-003:
  Timestamp::from_unix_nanos 调用位置不在允许清单 → fail/warn。

KERNEL-ERR-001:
  XError::internal 使用点不得超过基线。

KERNEL-ERR-002:
  出现字符串匹配决定错误分类 → fail。

KERNEL-SERDE-001:
  kernel 公开类型实现 Serialize/Deserialize → fail。

KERNEL-ASYNC-001:
  kernel 依赖或公开签名出现 tokio/async-std 类型 → fail。

KERNEL-UNSAFE-001:
  unsafe 代码数量必须为 0。

KERNEL-LIFECYCLE-001:
  ShutdownSignal 实现未通过 loom test → fail。
```

## 12.3 Public API 快照

必须生成并提交：

```text
.architecture/api/kernel-public-api.txt
```

快照来源优先级：

```text
rustdoc JSON
→ cargo-public-api
→ 受控源码扫描
```

源码字符串匹配不能作为唯一 API 证明。

---

# 13. 性能预算

kernel 不追求极端微优化，但必须避免不必要分配和锁争用。

## 13.1 Error

```text
- 静态 context 应允许 Cow::Borrowed；
- 动态 context 可分配；
- 构造 XError 不得隐式采集 backtrace；
- 不为 Clone 牺牲 source chain。
```

## 13.2 Clock

```text
- Timestamp 操作为 O(1)；
- SystemClock::now 不分配；
- monotonic 不分配；
- 无动态分派之外的隐藏全局锁。
```

## 13.3 Shutdown

```text
- is_triggered 允许使用 mutex；
- 正确性优先于无锁；
- trigger 为 O(number_of_waiters) 唤醒；
- wait 不 busy-loop。
```

若未来证明 `is_triggered` 锁成本成为真实瓶颈，可通过 RFC 引入“AtomicBool + 正确 Condvar 协议”，不得直接局部优化。

---

# 14. 文档要求

每个公开项必须包含：

```text
- 语义；
- 不变量；
- 失败行为；
- 示例；
- 非职责；
- 与其他模块的边界。
```

README 必须回答：

```text
- kernel 为什么存在；
- 允许进入 kernel 的准入四问；
- 如何注入 Clock；
- 如何使用 ShutdownSignal；
- 为什么 kernel 不提供 serde/async/Component trait。
```

AGENTS 必须列出：

```text
- 禁止依赖；
- 禁止 API；
- 验证命令；
- 变更流程；
- public API 风险。
```

---

# 15. 版本与兼容性

## 15.1 当前发布状态

```text
publish = true (crates.io package: xhyper-kernel)
```

已 publish 至 crates.io（包名 `xhyper-kernel` 0.1.1；`[lib] name = "kernel"` 保留 `use kernel::`）。仍按公开 API 治理，因为 workspace 全部下游依赖该 crate。

`publish = true` 只表达可发布/已发布事实，不构成生产认证。

## 15.2 版本策略

当前仓库采用 patch-default：

```text
0.1.0 → 0.1.1
```

但版本号不能自动授权破坏性变更。

破坏性变更必须同时具备：

```text
- Approved RFC；
- 全下游影响清单；
- 同一迁移批次；
- CHANGELOG；
- API diff；
- 回滚方案；
- Evidence。
```

## 15.3 稳定状态

只有满足本规范第 18 章全部验收条件后，registry 才可标记：

```toml
status = "stable"
```

在此之前必须是：

```toml
status = "incubating"
```

不得出现“spec draft、registry stable”的双重事实。

`stable` 表示受冻结 API 与兼容性流程治理，不表示当前 commit 已完成生产环境认证。

---

# 16. 迁移计划

## 16.1 Error 迁移

当前：

```text
XError enum variants
not_found → Invalid
other → Internal
```

目标：

```text
ErrorKind + opaque XError
missing / conflict / cancelled / deadline_exceeded
删除 not_found / other
```

步骤：

```text
1. 新增 ErrorKind 和新构造器；
2. 迁移全部调用点；
3. 禁止新增 not_found / other；
4. 删除兼容 API；
5. 更新 API 快照；
6. 更新所有断言，禁止匹配 Display 字符串。
```

## 16.2 Clock 迁移

当前问题：

```text
- Clock::monotonic 有真实 Instant 默认实现；
- ManualClock 只控制 wall clock；
- MonotonicInstant 反向差饱和为 0；
- 存在 from_nanos/as_nanos 简写。
```

步骤：

```text
1. 删除 monotonic 默认实现；
2. testkit ManualClock 增加独立 monotonic 状态；
3. 反向 monotonic 差改为 None；
4. 迁移 from_nanos/as_nanos 调用；
5. 限制 from_clock_elapsed 调用位置；
6. 增加 Clock contract suite。
```

## 16.3 Lifecycle 迁移

当前问题：

```text
- AtomicBool + Condvar 存在 lost wake-up 风险；
- Component trait 没有真实生产实现；
- 测试依赖 sleep，不能证明并发正确。
```

步骤：

```text
1. 改为同一 Mutex<bool> + Condvar 协议；
2. 增加 loom；
3. 删除 sleep 作为正确性证明；
4. 移除 Component trait；
5. 迁移仅存在于测试中的实现；
6. 更新 bootstrap 对 guard 生命周期的管理。
```

---

# 17. Evidence 要求

每次 kernel 变更必须生成：

```text
evidence/kernel/<date>-<change-id>/
├── manifest.json
├── commands.log
├── fmt.log
├── clippy.log
├── test.log
├── coverage.json
├── mutants.json
├── archgate.json
├── public-api.diff
├── downstream-impact.md
└── verdict.md
```

`manifest.json` 至少包含：

```json
{
  "commit": "<sha>",
  "toolchain": "<rustc version>",
  "spec": "SPEC-KERNEL-002",
  "package": "kernel",
  "commands": [],
  "artifacts": [],
  "result": "PASS|FAIL"
}
```

禁止：

- 手写 PASS 代替命令输出；
- 用旧 commit evidence 冒充当前结果；
- mutation/coverage SKIP 计为 PASS；
- 只记录成功命令，不记录失败命令。

---

# 18. 完成定义

本章定义验收条件，不在规范正文中维护随 commit 变化的 PASS 台账。当前验证状态见
[`gate/gate.md`](../gate/gate.md)；`evidence/2026-07-14/` 仅是不可变历史快照，不能
冒充当前提交的 Evidence。

## 18.1 规格闭合

- 本文件 Approved；
- 旧 spec 已 superseded；
- README / AGENTS / CHANGELOG 对齐；
- registry 状态一致；
- 无未登记 Unknown。

## 18.2 代码闭合

- 仅 error / clock / lifecycle；
- 内部依赖为 0，默认生产外部依赖只有 thiserror；
- `cfg(loom)` 只进入模型测试构建；
- 无非默认 feature、unsafe、Component trait、not_found / other；
- `ErrorKind` 不提供通用 `into_xerror` 构造旁路；
- `XError` 的常规 Display/Debug 不展开 source；
- Timestamp checked 算术覆盖完整 i64 域，墙钟允许回退；
- Clock::monotonic 无默认实现；
- ShutdownSignal 无 lost wake-up。

## 18.3 测试闭合

- 单元测试、property tests 与 loom tests 通过；
- 六个真实下游 rustdoc `compile_fail` 合同通过，static assertion 作为补充；
- line coverage >= 95%，branch coverage >= 90%；
- mutation score >= 90%，Miri 通过；
- 测试证据绑定被验收的 commit，SKIP 不计 PASS。

## 18.4 治理闭合

- public API 从当前源码重新生成并完成 diff；
- archgate KERNEL-* 16 条通过，含 API-002、TIME-004、PUBLISH-001；
- lint-deps 与 cargo-deny 通过；
- Internal 所有等价构造路径均受棘轮约束；
- 下游迁移完成；
- Evidence 可追溯到被验收的当前 commit。

---

# 19. 最终裁定

`kernel` 的职责永久限制为：

```text
error
clock
lifecycle
```

其公开价值不是功能丰富，而是：

```text
错误分类不含糊；
时间来源不旁路；
关停信号不丢失；
公开 API 不漂移；
依赖图永远向下收敛。
```

任何“为了方便”进入 kernel 的新增能力，默认视为拒绝，除非它能通过准入四问并完成 RFC、下游影响分析、机器门禁和完整 Evidence。
