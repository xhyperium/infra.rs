# kernel — Agent 行为规则

> 适用 crate：`crates/kernel/`（package / lib `kernel`，当前版本 `0.3.1`）
>
> 分发：`publish = false`
>
> 成熟度：L1 Internal Ready；L4 仅限新鲜证据已证明的支持面
>
> 父级规则：[`crates/AGENTS.md`](../AGENTS.md)
>
> 实现合同：`.agents/ssot/kernel/spec/spec.md`

本 crate 不作 production-certified、crates.io 发布或全量 L4 Platform Ready 声明。行为变更交付按 patch-default bump，最终版本必须与 Cargo、CHANGELOG 和 SSOT 同步。

## 职责

`kernel` 只提供：

- `error`：`ErrorKind`、`XError`、`XResult`、`BoxError`；
- `clock`：`Timestamp`、`MonotonicInstant`、`ClockDomain`、`Clock`、`SystemClock`、`ClockError`；
- `lifecycle`：`ComponentState`、`ShutdownSignal`、`ShutdownGuard`、`LifecycleError`、`WaitTimeoutError`。

禁止引入配置、观测、网络、存储、serde wire、异步运行时、依赖注入、组件编排、全局 Clock、测试替身或 `Component` trait。

## K1：依赖与编译边界

- workspace 生产依赖必须为零；
- 第三方生产依赖仅允许 `thiserror`；
- `proptest`、`serde`、`static_assertions` 仅为 dev-dependency；
- `loom` 仅为 `cfg(loom)` target dependency；
- 除 `default = []` 外不得新增 feature；
- 保持 `forbid(unsafe_code)`、`deny(missing_docs)`、`deny(unreachable_pub)`。

新增依赖、feature、公开项或职责必须先更新 SPEC，并完成 RFC、API diff 和下游影响分析。

## K2：错误语义

- 分类按调用方反应，不按来源模块；
- `XError` 保持不透明，禁止字符串匹配驱动控制流；
- 仅 `Transient` 可重试，仅 `Invariant` 表示程序 bug；
- 禁止 `not_found`、`other`、字符串 `From` 和通用分类构造旁路；
- `ClockError` 映射为 `XError::Unavailable` 并保留 source。

## K3：时间与 domain

- 墙钟与单调钟不得混用；
- `Clock::monotonic` 不得有默认实现；
- `Timestamp` checked 算术覆盖完整 `i64` 纳秒域；
- 所有 `SystemClock` 必须共享进程 origin 和 `ClockDomain::PROCESS`；
- 同进程不同 `SystemClock` 的单调点可比较；
- 跨 domain 的顺序与差值必须返回 `None`；
- raw domain 不得作为跨进程身份、wire 或持久化值。

以下 seam 获批准，但必须保持 `#[doc(hidden)]`：

```rust
MonotonicInstant::from_clock_elapsed(Duration)
MonotonicInstant::from_clock_elapsed_in(Duration, ClockDomain)
```

它们只供 `kernel` 的 `Clock` 实现和 `testkit::ManualClock` 使用。隐藏不等于私有，破坏性修改仍走公开 API 治理。

## K4：关停与有限等待

- `ShutdownSignal` 使用同一 `Mutex<bool> + Condvar` 协议；
- trigger 必须持锁写 `true` 并 `notify_all`；
- wait 必须用 while 循环，锁中毒通过 `into_inner` 恢复；
- signal 可 Clone，guard 不可 Clone；drop guard 不触发；
- 核心 wait/trigger 并发正确性必须由 loom 验证。

`ShutdownSignal::wait_timeout` 是 std 单调协调等待的窄例外。它可在实现内部使用 `Instant::now`，但不得生成业务时间或扩散成通用时间旁路。

目标返回面：

```rust
Result<bool, WaitTimeoutError>
```

语义固定为 `Ok(true)` 已触发、`Ok(false)` 正常到期未触发、`Err(DeadlineOverflow)` deadline 不可表示。

禁止用当前时刻替代不可表示的 deadline，禁止将其伪装为普通 timeout。`Duration::MAX` 必须由测试精确断言 typed error。

`wait_timeout` 仅在 `cfg(not(loom))` 存在；loom 继续验证无超时的核心协议。

## K5：公开 API

crate 根必须重导出 `ClockDomain` 和 `WaitTimeoutError`。公开 API 基线必须包含两个隐藏构造 seam 与 `wait_timeout` 的 `Result` 签名。

所有公开项必须有中文 rustdoc。破坏性变更必须同步 SPEC、设计、测试合同、CHANGELOG、API 基线和所有下游。

## 测试合同

至少覆盖：

- `Timestamp` 边界与 property test；
- process domain、共享 origin、同域与跨域比较；
- 常规 timeout、trigger、已触发、零时长；
- `Duration::MAX -> Err(WaitTimeoutError::DeadlineOverflow)`；
- trigger-before-wait、wait-before-trigger、多 observer 与 poison recovery；
- doctest `compile_fail`、static assertion、loom 与 public API gate。

## 验证

```bash
cargo fmt --all -- --check
cargo clippy -p kernel --all-targets -- -D warnings
cargo test -p kernel --all-targets
cargo test -p kernel --doc
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
node scripts/quality-gates/check-public-api.mjs
node scripts/quality-gates/check-workspace-deps.mjs
```

`--all-targets` 不包含 doctest，不能省略 `cargo test -p kernel --doc`。archgate 与 `.architecture/**` 对本仓 OOS。

## Evidence 与完成声明

只使用绑定当前 commit 的命令输出声明 PASS。历史 `.agents/ssot/kernel/evidence/2026-07-14/`、SKIP 或未运行项不得冒充 fresh evidence。

Agent 修改代码时必须更新相应测试和公开 API 基线；宣布完成前逐项核对 `.agents/ssot/kernel/gate/gate.md`。

## 修订记录

| 版本 | 日期 | 修订 |
|------|------|------|
| v2.0.0 | 2026-07-23 | 对齐 0.3.0 当前身份；批准 ClockDomain、共享 origin、隐藏 seam 与 typed timeout error |
