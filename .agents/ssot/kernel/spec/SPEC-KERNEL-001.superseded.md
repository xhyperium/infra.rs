# `kernel` Crate 完整规范（SPEC-KERNEL-001）

```text
状态:     draft（目标终态）+ 本轮兼容基线见 DESIGN-kernel-v1
Source Goal: GOAL-20260713-001
适用版本: P0-B (PR-2) 起
物理路径: crates/kernel
package:  kernel（重命名 kernel 属后续 PR-12；路径已是 crates/kernel）
layer:    kernel / L0
status:   stable（公开语义演进需 RFC）
owner:    platform
publish:  false
管线制品: .agents/ssot/kernel/<phase>/<phase>.md（Code → crates/kernel）
```

---

## 一、定位裁定

### 1.1 kernel 是什么

kernel 是 workspace 依赖图的**唯一底座**：所有 crate 可以依赖它，它不依赖任何内部 crate。它只提供三种全系统必须统一、且不属于任何业务能力的语义：

```text
1. 错误的表示与传播语义    (error)
2. 时间的获取与表示语义    (clock)
3. 组件生命周期语义        (lifecycle)
```

### 1.2 kernel 不是什么（负面清单，与正面定义同等效力）

```text
✗ 不是工具箱     —— 不收容 "好几个 crate 都用到" 的杂项函数
✗ 不是类型仓库   —— 业务标识/数值/canonical 类型归 types/*
✗ 不是 prelude   —— 不 re-export 第三方 crate
✗ 不含 IO        —— 无网络、无文件、无环境变量读取
✗ 不含配置       —— 归 infra/configx
✗ 不含日志/指标  —— 归 infra/observex; kernel 类型只保证可被观测
                    (Debug/Display), 不主动观测
✗ 不含 async 运行时耦合 —— 不依赖 tokio; trait 中不出现
                    运行时特定类型
```

**准入判据**：一个东西进入 kernel，当且仅当同时满足：

```text
□ 全系统必须只有一种语义 (两种并存会导致数据失真)
□ 不属于任何单一 capability
□ 无 IO、无第三方重依赖
□ 语义稳定 (预期年级别不变)
```

不满足任一条 → 归 types / infra / capability-local。

---

## 二、依赖规范

### 2.1 内部依赖

```text
kernel → (无)     # 绝对规则, archgate 强制, 无例外通道
```

### 2.2 外部依赖白名单

```toml
[dependencies]
thiserror = "..."     # 仅用于 derive, 不出现在公开类型签名中
# —— 白名单到此为止 ——
```

**明确禁止**（archgate 规则，出现即 fail）：

```text
anyhow            # 裁定核心: 公开 API 与私有实现都禁止
tokio / async-std # 运行时中立
serde             # kernel 类型无 wire 职责, 见 §6
chrono / time     # 时间表示自持, 见 §4.2
tracing / log     # 观测归 infra
```

`std` 中允许使用 `std::time::SystemTime` 的位置**有且仅有一处**：`clock::SystemClock` 的实现体内（见 §4.4）。

### 2.3 features

```toml
[features]
default = []
# kernel 无 feature。需要 feature 的东西不属于 kernel。
```

---

## 三、模块规范：`error`

### 3.1 设计原则

```text
1. 错误分类表达 "调用方应如何反应", 而非 "哪里出的错"
2. source 链保留完整因果, 但外层类型 (anyhow 等) 不进入枚举定义
3. 全部 Send + Sync + 'static
4. #[non_exhaustive], 允许后续加变体不破坏下游
```

### 3.2 公开 API

```rust
/// 全系统统一的顶层错误分类。
/// 变体按 "调用方反应" 划分, 而非按来源划分。
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum XError {
    /// 输入/请求本身非法。重试无意义, 调用方修正后重试。
    #[error("invalid input: {context}")]
    Invalid {
        context: Cow<'static, str>,
        #[source]
        source: Option<BoxError>,
    },

    /// 暂时性失败。调用方可按退避策略重试。
    #[error("transient failure: {context}")]
    Transient {
        context: Cow<'static, str>,
        /// 提示性重试间隔, 非承诺
        retry_after: Option<core::time::Duration>,
        #[source]
        source: Option<BoxError>,
    },

    /// 前置条件/内部不变量被破坏。表示 bug, 不应重试,
    /// 应记录并进入受控降级或 fail-fast。
    #[error("invariant violated: {context}")]
    Invariant {
        context: Cow<'static, str>,
        #[source]
        source: Option<BoxError>,
    },

    /// 依赖的下层组件不可用 (时钟/传输/存储)。
    /// 与 Transient 的区别: 不可用性由 lifecycle 层裁决, 
    /// 调用方通常应传播而非重试。
    #[error("unavailable: {context}")]
    Unavailable {
        context: Cow<'static, str>,
        #[source]
        source: Option<BoxError>,
    },

    /// 未归类的内部错误。P0-B 过渡期收容 anyhow 迁移遗留,
    /// 长期目标: 使用率棘轮递减 (见 §3.4)。
    #[error("internal: {context}")]
    Internal {
        context: Cow<'static, str>,
        #[source]
        source: Option<BoxError>,
    },
}

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type XResult<T> = Result<T, XError>;
```

### 3.3 构造与转换规范

```rust
impl XError {
    pub fn invalid(context: impl Into<Cow<'static, str>>) -> Self { ... }
    pub fn transient(context: impl Into<Cow<'static, str>>) -> Self { ... }
    pub fn invariant(context: impl Into<Cow<'static, str>>) -> Self { ... }
    pub fn unavailable(context: impl Into<Cow<'static, str>>) -> Self { ... }
    pub fn internal(context: impl Into<Cow<'static, str>>) -> Self { ... }

    /// 附加 source, 链式
    pub fn with_source(self, source: impl Into<BoxError>) -> Self { ... }

    /// 分类查询 —— 调用方据此决策, 禁止 match 变体后再看字符串
    pub fn is_retryable(&self) -> bool { ... }   // Transient
    pub fn is_bug(&self) -> bool { ... }         // Invariant
}
```

**禁止提供**：

```rust
impl From<anyhow::Error> for XError   // 禁止 —— 会让 anyhow 从依赖图回流
impl From<String> for XError          // 禁止 —— 强制显式分类
```

anyhow 迁移在**调用方 crate 内**通过 `XError::internal(...).with_source(...)` 完成，kernel 不提供糖。

### 3.4 治理规则

```text
archgate:
  □ kernel 公开签名出现 anyhow → fail (全 workspace 同规则)
  □ Internal 变体使用点计数进入棘轮: 只减不增
  □ 下游为 XError 新增分类需求 → RFC 修订本 spec, 
    禁止在下游用 Internal + 字符串约定伪造分类
```

---

## 四、模块规范：`clock`

### 4.1 设计原则

```text
1. 时间获取失败是显式错误, 禁止任何静默降级 (禁止返回 0/默认值)
2. 墙钟与单调钟是两个不同语义, 类型上分开, 禁止混用
3. 系统时间调用点全 workspace 收口到本模块唯一实现内
4. 可测试性通过 trait 注入实现, fake 实现归 testkit, 不在 kernel
```

### 4.2 时间类型

```rust
/// 墙钟时间点。内部: 自 Unix epoch 的纳秒数, i64。
/// 表示范围 ±292 年, 满足交易域需求。
/// 
/// 不变量:
///   - 只能通过 Clock::now() 或显式构造函数创建
///   - 无 Default 实现 (禁止 "零值时间戳" 存在)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    /// 显式构造, 用于 protocol 层反序列化与 fixture。
    /// 命名刻意冗长, 阻止随手使用。
    pub const fn from_unix_nanos(nanos: i64) -> Self { ... }
    pub const fn as_unix_nanos(&self) -> i64 { ... }

    pub fn checked_add(&self, d: Duration) -> Option<Timestamp> { ... }
    pub fn checked_duration_since(&self, earlier: Timestamp) 
        -> Option<Duration> { ... }
    // 溢出返回 None, 不 panic, 不饱和 —— 饱和也是静默失真
}

/// 单调时刻。仅用于测量间隔, 不可序列化, 不可跨进程比较。
/// 刻意不提供 as_nanos —— 单调钟的绝对值无意义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonotonicInstant(/* private */);

impl MonotonicInstant {
    pub fn duration_since(&self, earlier: MonotonicInstant) -> Duration { ... }
}
```

**明确不实现**：

```text
Default for Timestamp        # 零值时间戳是 P0-B 修复的病灶, 类型上根除
Serialize/Deserialize        # wire 表示归 types/protocol, 见 §6
From<SystemTime>             # 阻止绕过 Clock trait 的旁路构造
Display 输出人类可读时间      # 需要格式化 → observex; 
                             # kernel 只提供 Debug (输出纳秒原值)
```

### 4.3 Clock trait

```rust
/// 时间源抽象。全系统获取时间的唯一合法入口。
pub trait Clock: Send + Sync {
    /// 墙钟。失败时返回错误 —— 调用方禁止用默认值兜底。
    fn now(&self) -> Result<Timestamp, ClockError>;

    /// 单调钟。用于超时/延迟测量。
    fn monotonic(&self) -> MonotonicInstant;
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ClockError {
    /// 系统时间早于 Unix epoch 或超出 i64 纳秒表示范围
    #[error("system time out of representable range")]
    OutOfRange,
    /// 底层时间源不可用
    #[error("time source unavailable")]
    Unavailable,
}

impl From<ClockError> for XError {
    fn from(e: ClockError) -> Self {
        // 裁定: 时钟失败 → Unavailable 分类, 由 lifecycle 层
        // 决定 fail-fast (marketd 的裁定是进程级 fail-fast)
        XError::unavailable("clock").with_source(e)
    }
}
```

### 4.4 SystemClock

```rust
/// 生产实现。全 workspace 唯一允许调用 SystemTime::now() 的位置。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Result<Timestamp, ClockError> {
        // SystemTime::now() → duration_since(UNIX_EPOCH)
        //   Err  → ClockError::OutOfRange   (时间早于 epoch)
        //   溢出 i64 纳秒 → ClockError::OutOfRange
        // 禁止: unwrap / 返回 0 / 饱和
    }
    fn monotonic(&self) -> MonotonicInstant {
        // std::time::Instant 包装 —— Instant::now() 不会失败
    }
}
```

### 4.5 治理规则

```text
archgate:
  □ kernel 之外出现 SystemTime::now / Instant::now / 
    chrono::Utc::now / OffsetDateTime::now_utc → fail
    (存量登记 exceptions.toml, 带过期时间, 棘轮消除)
  □ Timestamp::from_unix_nanos 的调用点限定在:
    types/protocol, testkit, fixtures 加载路径
    —— services/domain 中出现 → warning (防旁路构造)

testkit (非 kernel) 提供:
  FakeClock: 手动推进、可注入 ClockError、
             墙钟/单调钟独立控制 (测试时钟回退场景)
```

---

## 五、模块规范：`lifecycle`

### 5.1 设计原则

```text
1. 定义 "组件如何启动、运行、终止" 的最小共同语义
2. 只提供语义与信号原语, 不提供编排 —— 编排归 apps/composition
3. 运行时中立: 不依赖 tokio, 用 std 原语 + 轮询/回调形态表达,
   async 包装由 infra 或 app 层完成
```

### 5.2 组件状态模型

```rust
/// 组件生命周期状态。全系统统一状态机, 禁止各 crate 自定义变体。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Created,
    Starting,
    Running,
    Draining,     // 收到 shutdown, 拒绝新工作, 处理存量
    Stopped,      // 干净终止
    Failed,       // 不可恢复失败终止
}

/// 合法转换 (其余转换非法, 由类型状态或运行时断言强制):
///   Created  → Starting
///   Starting → Running | Failed
///   Running  → Draining | Failed
///   Draining → Stopped | Failed
```

### 5.3 shutdown 原语

```rust
/// 关停信号。一次触发, 多方观察, 不可逆。
/// (线程安全, 内部为原子标记 + 唤醒机制)
pub struct ShutdownSignal { /* private */ }
pub struct ShutdownGuard { /* private */ }   // 触发方持有

impl ShutdownSignal {
    pub fn new() -> (ShutdownGuard, ShutdownSignal);
    pub fn is_triggered(&self) -> bool;
    /// 阻塞等待 (async 包装由上层提供)
    pub fn wait(&self);
}

impl ShutdownGuard {
    pub fn trigger(self);   // 消费 self —— 不可逆性由所有权表达
}

impl Clone for ShutdownSignal { ... }   // 多观察者
```

### 5.4 组件契约

```rust
/// 可管理组件的最小契约。
/// 实现方承诺:
///   - drain() 后不再产生新的对外副作用
///   - drain 存量处理有界 (deadline 由调用方持有并强制)
pub trait Component: Send {
    fn state(&self) -> ComponentState;
    /// 请求进入 Draining。幂等。
    fn drain(&mut self);
}
```

**明确不提供**：依赖排序、并行启动、健康检查、重启策略。这些是 composition/infra 职责——kernel 只保证所有组件说同一种生命周期语言。marketd 的 `SIGTERM → drain → checkpoint → exit 0` 序列在 `apps/marketd/shutdown.rs` 中用这些原语装配。

---

## 六、serde 与 wire 政策

```text
kernel 全部公开类型: 无 Serialize / Deserialize
理由 (裁定 §三-假设一): 内存模型与 wire representation 解耦

需要落盘/上线的 kernel 类型 (目前仅 Timestamp):
  types/protocol 中定义显式 wire struct:
    struct TimestampWire { unix_nanos: i64 }
  + 显式 From/TryFrom 双向转换
  epoch 与单位语义固化在 protocol schema 版本中
```

---

## 七、公开 API 全貌（冻结面）

```rust
// lib.rs —— 这就是 kernel 的全部, 增项需 RFC
pub mod error {
    pub enum XError;            // §3.2
    pub type BoxError;
    pub type XResult<T>;
}
pub mod clock {
    pub struct Timestamp;       // §4.2
    pub struct MonotonicInstant;
    pub trait Clock;            // §4.3
    pub enum ClockError;
    pub struct SystemClock;     // §4.4
}
pub mod lifecycle {
    pub enum ComponentState;    // §5.2
    pub struct ShutdownSignal;  // §5.3
    pub struct ShutdownGuard;
    pub trait Component;        // §5.4
}
```

**API 预算**：kernel 公开项总数以上述为上限基线。任何新增公开项走 RFC，且必须通过 §1.2 的准入判据四问。archgate 用 rustdoc JSON 对公开项集合做快照比对，未申报的新增公开项 → fail。

---

## 八、测试与 Evidence 要求

```text
单元测试 (kernel 内):
  □ Timestamp 溢出边界: checked_add / checked_duration_since 
    在 i64 边界返回 None
  □ XError 分类查询与变体的一致性
  □ ShutdownSignal: 多观察者、trigger 幂等不可达 (类型保证, 
    编译测试)、并发 wait 唤醒
  □ ComponentState 非法转换拒绝

契约测试 (testkit 内, 对 Clock 实现):
  □ now() 单调性不做断言 (墙钟允许回退), monotonic() 断言单调
  □ FakeClock 与 SystemClock 通过同一 suite

覆盖率要求: kernel 是全系统信任根, line coverage ≥ 95%,
           且 mutation testing 纳入 (kernel 体积小, 成本可承受)
```

---

## 九、稳定性与演进

```text
status = stable, 意味着:
  □ 破坏性变更需要 RFC + 全下游影响分析 + 迁移 PR 先行
  □ 新增枚举变体依赖 #[non_exhaustive], 属非破坏变更, 但仍需申报
  □ semver: workspace 内部统一版本, publish = false, 
    但 API 快照 diff 等价于 semver 审计

演进红线:
  □ 永不加入: async trait、tokio 类型、serde、全局单例 
    (包括 "全局默认 Clock" —— Clock 永远显式注入)
  □ Internal 错误变体在 anyhow 迁移完成后评估删除 (RFC)
```

---

## 十、迁移映射（kernel → kernel）

```text
现状                               处置
──────────────────────────────────────────────────────────────
XError(anyhow::Error 变体)      → PR-2: 重构为 §3.2 形态,
                                  调用方经 Internal+source 过渡
SystemClock 返回 0              → PR-2: Result<Timestamp, ClockError>
其余 kernel 内容          → 逐项过 §1.2 准入四问:
  通过    → 留在 kernel
  不通过  → 迁往 types/infra/testkit 或删除
  (清单在 PR-2 描述中给出, 每项附裁决理由)
物理路径                         → P2 PR-12: git mv → crates/kernel,
                                  package 改名, 纯机械操作
```

---

## 十一、本 spec 的 gate 落点汇总

```text
dependency.toml:   kernel 内部依赖 = ∅; 外部白名单 = [thiserror]
public_api.toml:   公开项快照比对; anyhow/serde/tokio 类型零出现
stability.toml:    kernel status = stable, 破坏变更需 RFC 字段
archgate 扫描:     kernel 外系统时间调用 = 0 (棘轮);
                   Timestamp 旁路构造点限域;
                   XError::Internal 使用计数棘轮
```

---

## 结语

kernel 的规范本质上只有一句话：

> **kernel 提供的不是功能，而是全系统被迫共享的三种语义——错误如何被反应、时间如何被信任、组件如何被终止。任何不满足"必须全局唯一"判据的东西，进入 kernel 都是对信任根的稀释。**

它的健康指标不是功能增长，而是**公开 API 长期不变 + 下游对它的例外为零**。kernel 越无聊，系统越健康。