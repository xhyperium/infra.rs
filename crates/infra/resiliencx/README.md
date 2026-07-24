# resiliencx

L1 进程内弹性原语：重试、重试预算、熔断、限流与舱壁（ADR-005）。

Cargo package / lib 均为 `resiliencx`，当前版本 `0.1.2`，且 `publish = false`。

| 能力 | 生产入口 / 类型 | 边界 |
|------|-----------------|------|
| 安全重试 | `RetryContext` + `RetrySafety` + `retry_fn_safe` / `retry_async_safe` | 多次尝试前显式声明只读或幂等；不安全副作用被拒绝 |
| 安全 Adapter budget | `call_with_retry_budget_safe` / `call_with_retry_budget_async_safe` | generic 返回值；首次 operation 前校验 safety |
| 整次 deadline | `retry_async_with_deadline`（feature `tokio`） | 覆盖尝试与退避；cooperative cancellation，不撤销已发生副作用 |
| 重试预算 | `RetryBudget` + safe retry / Adapter 入口 | 每次真正 retry 消耗令牌；耗尽统一返回标准 budget 错误 |
| 退避 | `Backoff::{Constant, Exponential}` | `Wait` / `AsyncWait` 可注入 |
| jitter | `retry_delay_ms_with_seed` / `apply_seeded_jitter` | seed 由调用方注入；非加密 RNG |
| 熔断 | `CircuitBreaker` 三态 | 本地、无墙钟；Open 按拒绝次数推进 |
| 限流 | `RateLimiter` 令牌桶 | 本地、无墙钟；仅显式 `refill` |
| 舱壁 | `Bulkhead` / `BulkheadPermit` | 本地并发上限；满载立即拒绝，无排队/等待 |

## 生产安全重试

```rust
use resiliencx::{
    NoWait, NoopInstrumentation, RetryConfig, RetryContext, RetrySafety, retry_downcast, retry_fn_safe,
    retry_ok,
};

let config = RetryConfig::fixed(3, 0);
let mut operation = || Ok(retry_ok(42u8));
let value = retry_fn_safe(
    RetryContext::new(&config, RetrySafety::ReadOnly, &NoopInstrumentation, "read-profile")
        .with_jitter_seed(7),
    &NoWait,
    &mut operation,
)?;
assert_eq!(retry_downcast::<u8>(value)?, 42);
# Ok::<(), kernel::XError>(())
```

`RetryContext` 聚合 config、safety、instrumentation、op、可选 budget 与 caller seed。
`RetrySafety` 是调用方声明，不是对闭包的静态证明。`max_attempts > 1` 时，
`UnsafeSideEffect` 会在首次调用前返回 `Invalid`；`max_attempts == 1` 仍允许单次执行。

`call_with_retry_budget`、`call_with_retry_budget_async`、`retry_fn`、`retry_fn_with_wait`、
`retry_fn_with_budget`、`retry_fn_with_wait_budget`、`retry_async`、`retry_async_with_budget` 为
unchecked compatibility，**不会**替调用方校验副作用安全性。它们不得直接称为生产入口；新生产接线
必须使用对应 `*_safe` 入口。

## async deadline 与取消

feature `tokio` 下，`retry_async_with_deadline` 用 `tokio::time::timeout` 包裹整次安全重试，
包括每次 operation future 与退避等待。超时统一映射为 `XError::deadline_exceeded`。

取消是 cooperative cancellation：超时时待执行 future 停止被轮询，但已经完成的网络写入、
数据库提交或其他外部副作用不会被自动撤销；operation 自行派生的后台任务也可能继续运行。
因此 deadline 不能替代幂等键、事务或补偿机制。

async budget 在退避前原子 reserve；deadline 在退避期取消时，未 commit 的预留由 RAII 自动 refund，
且不会产生 `record_retry`。预算初始耗尽时在进入 wait 前立即返回标准 budget 错误。

## jitter 去相关

兼容入口 `retry_delay_ms` / `apply_deterministic_jitter` 只依赖 attempt；相同配置的实例会得到
同一序列，**不具备抗群聚保证**。需要实例去相关时，调用方使用
`RetryContext::with_jitter_seed` 把不同 seed 接入 safe sync/async/deadline 的实际退避；
纯计算场景可使用 `retry_delay_ms_with_seed` / `apply_seeded_jitter`。

## 本地立即拒绝边界

熔断、限流、舱壁均是单进程内原语，不做跨进程协调。`RateLimiter::try_acquire` 与
`Bulkhead::try_enter` 资源不足时立即返回 `Unavailable`，没有公平队列、排队 deadline 或自动 refill。
这些行为不能被描述为分布式限流、排队舱壁或按时间冷却熔断。

## 验证

```bash
cargo fmt --all --check
cargo test -p resiliencx --all-features --all-targets
cargo clippy -p resiliencx --all-features --all-targets -- -D warnings
cmp .agents/ssot/infra/resiliencx/spec/spec.md \
    .agents/ssot/infra/resiliencx/spec/xhyper-resiliencx-complete-spec.md
```

SSOT：`.agents/ssot/infra/resiliencx/spec/spec.md`
对齐：[docs/ssot/resiliencx-ssot-alignment.md](../../docs/ssot/resiliencx-ssot-alignment.md)

**未宣称**：package stable、分布式弹性平台、自动墙钟策略、撤销外部副作用。
