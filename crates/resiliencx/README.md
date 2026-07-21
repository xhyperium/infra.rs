# resiliencx

L1 **重试**（active SSOT §2，ADR-005）。当前仅交付 `RetryConfig` + `retry_fn`。

**不是** 完整弹性系统：熔断 / 限流 / bulkhead **未实现**（SSOT §3 OPEN）。

## 公开面

| 项 | 说明 |
|----|------|
| `RetryConfig` | `max_attempts`（含首次）、`base_delay_ms` |
| `retry_fn` | 同步重试；仅 `XError::is_retryable()`（Transient）触发重试 |
| `retry_ok` / `retry_downcast` | 成功值装箱 / 拆箱（`RetryValue`） |
| `Instrumentation` | 可观测注入（本 crate 定义；本仓无 contracts） |
| `NoopInstrumentation` | 空实现 |

## 依赖

- 生产：`xhyper-kernel` 仅
- 禁止：`observex` / 反向依赖 transport·domain·app
- dev：`tokio`（测试运行时）

## 已知差距

- `base_delay_ms > 0` 使用 `std::thread::sleep`，会阻塞 executor；**不是**已批准的 async wait 合同。

## 验证

```bash
cargo test -p xhyper-resiliencx
cargo clippy -p xhyper-resiliencx --all-targets -- -D warnings
cargo llvm-cov -p xhyper-resiliencx --all-targets --summary-only
```

## SSOT

`.agents/ssot/infra/resiliencx/spec/spec.md`（用户路径别名 `.agent/ssot/resiliencx`）。

## 版本

0.1.0 · **≠** package stable / crates.io。
