# `resiliencx` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 部分实现合同；仅 retry，非生产完整弹性系统 |
| Package / lib | `xhyper-resiliencx` / `resiliencx` |
| Path | `crates/resiliencx` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Candidate / Draft | 历史 Draft 能力列表见 §3 OPEN；**不**覆盖本文 |
| Path alias | 用户 `.agent/ssot/resiliencx` ≡ 本仓 `.agents/ssot/infra/resiliencx` |
| Verified | 2026-07-21 · Lines Cover 100%（`cargo llvm-cov -p xhyper-resiliencx`） |

## 1. 定位与依赖

L1 弹性职责目标为重试/熔断/限流。**当前普通依赖仅 `xhyper-kernel`**。

可观测性通过本 crate 公开的 [`Instrumentation`] trait 注入（ADR-005）；**禁止**直接依赖 observex。本仓 **无** `xhyper-contracts`：上游 `contracts::Instrumentation` 语义在本 crate 本地复刻，避免 contracts 依赖。

当前 workspace 尚无 owner 外的生产 consumer。

## 2. 当前公开 API

```text
RetryConfig { pub max_attempts: u32, pub base_delay_ms: u64 }
Instrumentation { record_retry / record_circuit_open / record_circuit_close }
NoopInstrumentation
RetryValue = Box<dyn Any + Send>
retry_ok(T) / retry_downcast<T>(RetryValue)
retry_fn(config, instrumentation, op, &mut dyn FnMut() -> XResult<RetryValue>) -> XResult<RetryValue>
```

当前行为：

1. `max_attempts` 含首次；0 返回 `XError::Invalid`。
2. 仅 `XError::is_retryable()`（当前仅 `Transient`）触发重试。
3. non-retryable 错误立即返回；耗尽后返回最后一个原始错误。
4. 每次真正发起 retry 前调用 `record_retry(op, attempt)`。
5. `base_delay_ms > 0` 时调用 `std::thread::sleep`（阻塞调用线程）。

第 5 项会阻塞调用线程，且不能由 ManualClock 控制或在 backoff 中取消；这是当前已知差距，不是批准的生产等待合同。

### 本仓相对上游的诚实适配

| 项 | 上游 xhyper | 本仓 infra.rs |
|----|-------------|---------------|
| 异步 | `async fn` + 泛型 Future | **同步** `FnMut` + `RetryValue` 装箱（消除 llvm 泛型 monomorph 覆盖空洞） |
| Instrumentation | `contracts::Instrumentation` | 本 crate trait（无 contracts 包） |
| 依赖 | kernel + contracts | **仅 kernel** |

语义合同（attempts / retryable / record_retry / sleep / zero→Invalid）与 §2 行为列表一致。

## 3. 未实现能力（OPEN / DEFER — 不得标 DONE）

- injected async wait、deadline、cancellation；
- backoff/jitter、Retry-After、operation idempotency/safety；
- retry budget、execution report；
- circuit breaker、rate limiter、bulkhead/concurrency limiter；
- package stable / crates.io。

crate 名称与 description 不构成这些能力已交付的证据。

## 4. 当前测试

- 首试成功、失败后成功、全部耗尽、non-retryable、zero attempts、Default；
- `base_delay_ms>0` 短 sleep 分支；
- `record_retry` 次数/参数；
- `retry_ok` / `retry_downcast`（含类型不匹配）；
- 库外 `tests/public_api.rs`。

覆盖率目标：`cargo llvm-cov -p resiliencx --all-targets --summary-only` → **Lines Cover 100%**。

## 5. 验收

```bash
cargo test -p xhyper-resiliencx
cargo check -p resiliencx --all-targets
cargo clippy -p resiliencx --all-targets -- -D warnings
cargo fmt -p resiliencx -- --check
cargo llvm-cov -p resiliencx --all-targets --summary-only
```

通过条件：当前 retry 行为与源码一致；无 observex 横向依赖；文档不把未实现策略写成已交付；Lines 100%。

## 6. 追溯

- Active：本文件 ≡ `spec/resiliencx-complete-spec.md`（须 `cmp`）
- Residual：`plan/residual-open.md`
- Alignment：`plan/alignment-matrix-infra-2026-07-21.md`
- 实现：`crates/resiliencx/{Cargo.toml,src/**,tests/**}`
