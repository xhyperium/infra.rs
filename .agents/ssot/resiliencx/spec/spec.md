# `resiliencx` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.2` L1 进程内弹性合同；Internal Ready，非 package stable |
| Package / lib | `resiliencx` / `resiliencx` |
| Path | `crates/resiliencx` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Complete mirror | `spec/xhyper-resiliencx-complete-spec.md`，必须与本文件 `cmp` 一致 |
| Verified | 2026-07-23 · coverage `1208 / 1208`；候选已重冻，本地 reviewer 完成、verifier 技术/证据初验完成；GitHub CI artifact pending |

## 1. 定位与依赖

本 crate 提供单进程内的有限重试、预算、熔断、令牌桶限流与舱壁。
普通依赖为 `kernel`、`contracts`、`async-trait`；feature `tokio` 提供非阻塞 sleep 与整次 retry deadline。
可观测性复用 `contracts::Instrumentation`；禁止直接依赖 observex。

## 2. 重试安全合同

生产入口：

```text
RetryContext::new(config, safety, instrumentation, op)
    .with_budget(budget)
    .with_jitter_seed(seed)
retry_fn_safe(context, wait, operation)
retry_async_safe(context, wait, operation)
call_with_retry_budget_safe(budget, attempts, safety, op, instrumentation, operation)
call_with_retry_budget_async_safe(budget, attempts, safety, op, instrumentation, operation)
```

`RetrySafety` 取值：

- `ReadOnly`：调用方保证操作无外部副作用；
- `Idempotent`：调用方保证重复执行的领域效果等价；
- `UnsafeSideEffect`：不保证安全重复。

当 `max_attempts > 1` 且 safety 为 `UnsafeSideEffect` 时，安全入口必须在首次调用前返回 `Invalid`。
`max_attempts == 1` 可执行一次不安全副作用。声明由调用方负责，本 crate 不从闭包内容静态证明幂等性。

只有名称带 `_safe` 的上述入口和 `retry_async_with_deadline` 执行显式 safety 校验。
以下 API 为 unchecked compatibility，不执行 safety 校验，不得描述为生产安全入口：

- `call_with_retry_budget` / `call_with_retry_budget_async`；
- `retry_fn` / `retry_fn_with_wait` / `retry_fn_with_budget` / `retry_fn_with_wait_budget`；
- `retry_async` / `retry_async_with_budget`。

## 3. 重试、预算与观测合同

1. `max_attempts` 含首次；0 返回 `Invalid`。
2. 仅 `XError::is_retryable()` 触发 retry；non-retryable 立即返回；次数耗尽返回最后原始错误。
3. 每次真正 retry 前先消费一个可选 `RetryBudget` 令牌。
4. budget 耗尽时同步与异步统一返回 `budget_exhausted_error()`，不得返回前一次瞬态错误。
5. 预算成功消费且即将 retry 时，调用 `record_retry(op, attempt)`；`attempt` 是刚失败的尝试序号，从 1 起。
6. 未实际发起的 retry 不得记录观测事件。
7. async 在退避前原子 reserve 预算（空则立即标准错误）；退避完成后 commit 并记录 retry。
   deadline 在退避期取消会通过 RAII refund 预留，不消耗预算、不产生虚假事件。
8. 同步默认 wait 为阻塞线程的 `ThreadSleepWait`；async 安全生产路径使用 `AsyncWait` / `TokioSleepWait`。

### 3.1 Generic Adapter budget

- sync/async safe Adapter 入口返回 generic `T`，不要求 `RetryValue` 装箱；
- unchecked generic async core 统一 budget exhaustion 与失败 attempt 观测；safe async 入口先校验 safety
  再委托该 core；
- safety 与 `max_attempts` 在首次闭包调用或首次 operation future 构造前校验；
- `record_retry(op, attempt)` 记录刚失败的 attempt，从 1 起；
- 无 wait 的 generic async Adapter 在 retry operation 已构造并被轮询后取消，视为该 attempt 已实际发起，
  已消费预算不退还；这不同于在退避 wait 中取消时 reservation 自动 refund。

### 3.2 当前 Adapter 消费事实

- Redis 生产 client budget 路径只使用 safe async wrapper：GET/EXISTS/PTTL/MGET 为 `ReadOnly`，
  无 TTL SET 与 MSET 为 `Idempotent`，DEL/PEXPIRE/相对 TTL SET 为 `UnsafeSideEffect`；
- Postgres 当前 `PostgresPool` 没有 budget 字段或自动接线；仅提供显式 safety wrapper。
  任意 SQL 不可从字符串证明只读或幂等，调用方未显式声明时必须保守视为 `UnsafeSideEffect`；
- Postgres/Redis 的旧 `with_budget*` / `with_retry*` wrapper 保留为 unchecked compatibility。

## 4. deadline 与 cancellation 合同

feature `tokio` 下，`retry_async_with_deadline` 包裹整次 `retry_async_safe`，deadline 覆盖 operation future、
所有 retry 与退避等待。超时统一映射 `XError::deadline_exceeded`。

取消采用 Tokio cooperative cancellation：超时后待执行 future 不再被轮询，但本 crate不保证撤销已经发生的
网络写入、数据库提交或其他外部副作用；operation 自行派生的后台任务也可能继续运行。调用方仍须使用
幂等键、事务或补偿机制。

## 5. backoff 与 jitter 合同

- `Backoff::Constant` 与 `Backoff::Exponential` 计算有限退避；
- `retry_delay_ms` / `apply_deterministic_jitter` 仅由 attempt 驱动，保留历史确定性结果；
- attempt-only jitter 在相同配置的实例间同相，**不具备抗群聚保证**；
- `RetryContext::with_jitter_seed` 将 caller seed 接入 safe sync/async/deadline 的实际退避；
- `retry_delay_ms_with_seed` / `apply_seeded_jitter` 同时公开纯计算入口；仍非加密 RNG。

## 6. 其他本地原语合同

- Circuit Breaker：`Closed` / `Open` / `HalfOpen`；无墙钟，Open 按拒绝次数推进 HalfOpen。
- Rate Limiter：满桶起步，显式 `refill`；`try_acquire` 不足时立即 `Unavailable`，不部分扣减。
- Bulkhead：RAII 许可与本地并发上限；`try_enter` 满载时立即 `Unavailable`，无排队/等待 deadline。
- Bulkhead 状态锁 poisoned 时恢复 inner；permit drop 必须归还槽位，不得永久泄漏容量。

以上均为单进程原语，不提供跨进程协调、公平队列、自动墙钟 refill/cooldown 或分布式一致性。

## 7. 验收

```bash
cargo fmt --all --check
cargo test -p resiliencx --all-features --all-targets
cargo clippy -p resiliencx --all-features --all-targets -- -D warnings
cargo test -p postgresx -p redisx --all-features --all-targets
cmp .agents/ssot/resiliencx/spec/spec.md \
    .agents/ssot/resiliencx/spec/xhyper-resiliencx-complete-spec.md
```

必须覆盖：安全拒绝发生在首次闭包/future 前；只读/幂等与单次不安全操作可执行；async budget parity；
attempt 观测一致性；deadline 成功/超时映射；bulkhead poison 恢复；seed jitter；Redis 实际 client 分类；
Redis 零 attempts 在 future/driver 前拒绝；legacy async budget 标准错误与失败 attempt 序号。

## 8. 非宣称

- package stable / crates.io；
- 分布式熔断、限流或舱壁；
- 自动墙钟 refill/cooldown；
- deadline 撤销已发生副作用；
- attempt-only jitter 抗群聚；
- execution report、Retry-After 或完整弹性平台。

## 9. 追溯

- Findings：`plan/round-01-findings.md`、`plan/round-02-findings.md`、`plan/round-03-findings.md`
- Alignment：`docs/ssot/resiliencx-ssot-alignment.md`
- 实现：`crates/resiliencx/{src/**,tests/**,README.md,docs/**,CHANGELOG.md}`
