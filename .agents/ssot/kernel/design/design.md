# DESIGN-KERNEL-002 · L0 运行时语义

| 字段 | 当前值 |
|------|--------|
| Source Spec | `SPEC-KERNEL-002` |
| Status | Active |
| Package / lib | `kernel` / `kernel` |
| Current version | `0.3.1` |
| Distribution | `publish = false` |
| Maturity | L1 Internal Ready；L4 仅限已证支持面 |
| Production certification | 未声明 |

本设计解释当前实现边界与关键取舍。`spec.md` 定义必须满足的契约；本设计说明边界与取舍；源码和测试提供可执行事实。gate 只记录与当前 commit 绑定的新鲜验证结果。

仓库位置：`evidence` crate 的 `path: crates/evidence`，其 current-state spec 位于 `.agents/ssot/evidence/`；`tools/evidence` 不是当前路径。

## 1. 总体结构

`kernel` 由三个深模块组成：

```text
error      调用方反应分类、opaque XError、source chain
clock      墙钟、单调钟、domain、安全算术
lifecycle  状态机、同步关停与有限协调等待
```

边界刻意保持窄：无 workspace 生产依赖，第三方生产依赖仅 `thiserror`，无 async runtime、serde、观测、网络或组件编排。

## 2. Error 设计

`ErrorKind` 按调用方反应而非来源模块分类。`XError` 不透明，避免下游绑定内部字段或用字符串匹配驱动控制流。

具名构造器让分类选择在创建点可审查。`with_source` 保留错误链，但 Display 不承担协议职责。

`ClockError` 统一映射到 `XError::Unavailable`。这表达“时间依赖不可用”，避免把环境故障误写成输入错误。

## 3. Clock 设计

### 3.1 两种时间

`Timestamp` 是业务绝对时间，允许因校时回退。`MonotonicInstant` 是进程内间隔采样点，不可持久化，也不能产生跨 domain 顺序。

checked 算术用足够宽的中间值覆盖完整 `i64` 纳秒域。反向差与溢出返回 `None`，不通过饱和制造看似有效的结果。

### 3.2 Domain 是比较资格

`ClockDomain` 不是全局唯一 ID，而是“这些单调点是否可比较”的资格标签。`ClockDomain::PROCESS` 只在当前进程生命周期内有效。

跨 domain 的 `PartialOrd` 和 `checked_duration_since` 返回 `None`。这样，测试时钟、仿真时钟和系统时钟不会因数值碰巧接近而产生错误间隔。

### 3.3 进程共享 origin

所有 `SystemClock` 共享一次初始化的进程 origin，并输出 `ClockDomain::PROCESS`。因此，依赖注入创建的多个 `SystemClock` 仍属于同一比较域。

```text
OnceLock<Instant> ── elapsed ──> MonotonicInstant
                              └─ domain = PROCESS
```

此设计消除“每个实例各自从零开始”导致的跨实例假顺序。进程重启后不保留比较资格，raw domain 值也不得写入 wire 或数据库。

### 3.4 隐藏构造 seam

`from_clock_elapsed` 与 `from_clock_elapsed_in` 是 `#[doc(hidden)]` 的公开 seam。它们让 `Clock` 实现和外部 `testkit::ManualClock` 构造不透明单调点。

隐藏仅减少普通文档噪声，不取消兼容性责任。业务模块不得直接使用；删除、改名或改变 domain 语义均按公开 API 变更处理。

`from_clock_elapsed` 固定使用进程 domain；`from_clock_elapsed_in` 供 testkit 传入独立 domain。`domain()` 让合同测试验证比较资格。

## 4. Lifecycle 设计

### 4.1 无 lost wake-up 的关停

`ShutdownSignal` 与 `ShutdownGuard` 共享 `Mutex<bool> + Condvar`。trigger 持同一把锁设置状态并唤醒全部 waiter；wait 用 while 循环抵抗伪唤醒。

guard 是唯一、消费式触发入口。signal 可克隆，状态不可逆。drop guard 不隐式解释为系统关停，所有权与关停意图由组合根显式连接。

### 4.2 `wait_timeout` 的窄例外

`Clock` 用于业务时间注入；`wait_timeout` 用于 std 线程协调。后者必须直接与 `Condvar` 和 `std::time::Instant` 配合，不能注入可任意跳变的业务时钟。

因此，`wait_timeout` 是 `Instant::now` 收口规则的明确例外，但只允许存在于该方法实现及其私有 helper 中。

```text
首次持锁观察到已触发 ───────────────> Ok(true)
未触发且 checked_add(timeout) 失败 ─> Err(DeadlineOverflow)
未触发且 deadline 可表示 ──────────> 等待到触发或 deadline
```

旧行为把不可表示的 deadline 回退为当前时刻，并返回普通 timeout。这会把调用错误伪装成真实超时，现改为 `WaitTimeoutError` typed error。

固定 deadline 只建立一次。伪唤醒后重算 remaining，而不是重启完整 timeout，避免实际等待被无限延长。

`wait_timeout` 在 `cfg(loom)` 下不可用。loom 验证无超时的核心 Condvar 协议；std 单元与集成测试验证 timeout、trigger 和 deadline overflow。

## 5. 公开接口

crate 根公开面为：

```rust
pub use clock::{
    Clock, ClockDomain, ClockError, MonotonicInstant, SystemClock, Timestamp,
};
pub use error::{BoxError, ErrorKind, XError, XResult};
pub use lifecycle::{
    ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal,
    WaitTimeoutError,
};
```

`ShutdownSignal::wait_timeout` 的目标签名：

```rust
pub fn wait_timeout(
    &self,
    timeout: Duration,
) -> Result<bool, WaitTimeoutError>;
```

`Ok(true)`、`Ok(false)` 和 `Err(DeadlineOverflow)` 是三个不可合并的结果。下游必须显式处理错误，不得使用 `unwrap_or(false)` 再次抹平语义。

## 6. 被拒绝的方案

公开面以 `docs/api-baselines/kernel.txt` 与 `scripts/quality-gates/check-public-api.mjs` 做棘轮；并发协议由 `kernel-loom.yml` 持续验证。`archgate` 与 `.architecture/**` 对本仓为 OOS，不能作为本仓门禁或证据来源。

| 方案 | 拒绝原因 |
|------|----------|
| 每个 `SystemClock` 保存私有 origin | 不同注入实例的采样点无法可靠比较 |
| 单调点无 domain | 无法识别 test/system 或不同仿真时钟的错误比较 |
| 公开绝对 ticks getter | 鼓励持久化、跨进程比较和业务依赖绝对值 |
| deadline overflow 当 timeout | typed failure 被伪造成正常控制流 |
| `wait_timeout` 使用注入 `Clock` | 业务墙钟/仿真钟不满足 std Condvar 的协调语义 |
| AtomicBool + Condvar | 无同 mutex 协议，存在 lost wake-up 风险 |
| 通用 `Component` trait | 把 async 与编排压力过早下沉 L0 |

## 7. 测试分层

```text
unit/property   纯语义、边界、Duration::MAX typed error
integration     常规 timeout/trigger、进程 domain、跨 domain
loom            wait/trigger 核心并发协议
doctest         compile_fail 负向消费面
public API      根导出、签名与隐藏 seam 的兼容性棘轮
```

真实 sleep 或忙等待只能作为 std 行为烟雾，不能替代 loom 的并发证明。历史 coverage、Miri 或 mutation 结果不自动继承到当前 commit。

## 8. 版本与发布

当前交付是 `kernel 0.3.1`，`publish = false`；公开错误面与返回签名变更已执行一次 patch bump，并须与 Cargo、CHANGELOG、API 基线和本组 SSOT 同步。

历史 `xhyper-kernel 0.1.1` 发布事件不属于当前包身份或分发策略。本文不作 crates.io、production-certified 或全量 L4 Platform Ready 声明。

## 9. 关键决策

| ID | 决策 |
|----|------|
| KD-1 | Error 按调用方反应分类，`XError` 保持不透明 |
| KD-2 | 墙钟与单调钟分离，所有 `Clock` 明确实现两者 |
| KD-3 | `ClockDomain` 决定单调点比较资格 |
| KD-4 | 所有 `SystemClock` 共享进程 origin 与 `PROCESS` domain |
| KD-5 | 两个 `#[doc(hidden)]` 构造 seam 获批准但限制调用方 |
| KD-6 | `wait_timeout` 是 std 单调协调等待的窄例外 |
| KD-7 | deadline 不可表示返回 `WaitTimeoutError`，绝不伪装 timeout |
| KD-8 | loom 证明核心 wait/trigger；std 测试证明 timeout 错误面 |
| KD-9 | 当前只声明 L1 Internal Ready 与 L4 已证支持面 |

## 10. 证据纪律

`.agents/ssot/kernel/evidence/2026-07-14/` 仅为历史快照，不得冒充本轮 PASS。新结论必须绑定当前 commit、命令、工具链和输出。

gate 中的 PASS 只能在命令实际执行并留存新鲜证据后填写。未运行、跳过或只引用旧制品时，状态必须保持待验证。
