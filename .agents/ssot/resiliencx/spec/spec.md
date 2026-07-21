# `resiliencx` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 部分实现合同；仅 retry，非生产完整弹性系统 |
| Package / lib | `xhyper-resiliencx` / `resiliencx` |
| Path | `crates/resiliencx` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Candidate | [SPEC-INFRA-RESILIENCX-002](../../../../draft/xhyper-resiliencx-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Implementation snapshot | `b0934baa`（2026-07-15） |
| Document commit | `e0b98df4` |
| Verified at | `e0b98df4`（相关实现路径未变化） |

## 1. 定位与依赖

L1 弹性职责目标为重试/熔断/限流。当前普通依赖仅 `xhyper-kernel`、`xhyper-contracts`，dev 使用 `tokio`。

可观测性只通过 `contracts::Instrumentation` 注入；禁止直接依赖 observex（ADR-005 / R3）。当前 workspace 没有 owner 外的 `retry_fn` / `RetryConfig` 生产引用。

## 2. 当前公开 API

```text
RetryConfig { pub max_attempts: u32, pub base_delay_ms: u64 }
retry_fn(config, instrumentation, op, closure) -> XResult<T>
```

当前行为：

1. `max_attempts` 含首次；0 返回 `XError::Invalid`。
2. 仅 `XError::is_retryable()`（当前仅 `Transient`）触发重试。
3. non-retryable 错误立即返回；耗尽后返回最后一个原始错误。
4. 每次真正发起 retry 前调用 `record_retry(op, attempt)`。
5. `base_delay_ms > 0` 时在 async 函数内调用 `std::thread::sleep`。

第 5 项会阻塞 executor worker，且不能由 ManualClock 控制或在 backoff 中取消；这是当前已知差距，不是批准的生产等待合同。

## 3. 未实现能力

- injected async wait、deadline、cancellation；
- backoff/jitter、Retry-After、operation idempotency/safety；
- retry budget、execution report；
- circuit breaker、rate limiter、bulkhead/concurrency limiter。

crate 名称与 description 不构成这些能力已交付的证据。候选设计仅见 Candidate Draft，未批准前不是 active API。

## 4. 当前测试

6 个测试覆盖：首试成功、失败后成功、全部耗尽、non-retryable、zero attempts、Default。

反例条件：`thread::sleep` 被移除、owner 外出现 consumer 或新策略 API 落地时，本实现清单和差距必须重审。

## 5. 验收

```bash
cargo test -p xhyper-resiliencx
cargo check -p xhyper-resiliencx --all-targets
cargo clippy -p xhyper-resiliencx --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

通过条件：当前 retry 行为与源码一致；无 observex 横向依赖；文档不把未实现策略写成已交付。

## 6. 追溯

- [ADR-005](../../../../../docs/architecture/adr/005-resiliencx-observability-boundary.md)
- `docs/architecture/spec.md` §4.4
- `crates/resiliencx/{Cargo.toml,src/lib.rs}`
