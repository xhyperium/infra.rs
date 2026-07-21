# resiliencx SSOT 对齐（infra.rs）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-21 |
| Active SSOT | `.agents/ssot/infra/resiliencx/spec/spec.md` |
| 用户路径别名 | `.agent/ssot/resiliencx` → `.agents/ssot/infra/resiliencx` |
| 实现 | `crates/resiliencx` · package `xhyper-resiliencx` |

## 结论

| 能力 | 状态 | 证据 |
|------|------|------|
| 重试 §2 | **PASS** | `retry.rs` + `tests/retry_contract.rs` |
| 退避 / 确定性 jitter | **PASS** | `Backoff` / `retry_delay_ms` / `jitter_bps` |
| 可注入 wait | **PASS** | `Wait` / `retry_fn_with_wait` / `NoWait` / `RecordingWait` |
| 熔断 | **PASS**（无墙钟） | `circuit.rs` |
| 限流（令牌桶） | **PASS** | `rate_limit.rs` |
| 舱壁（bulkhead） | **PASS** | `bulkhead.rs` |
| Instrumentation | **PASS** | re-export `contracts::Instrumentation`；禁止 observex |
| LCOV 行 100% | **PASS** | `cov-gate-100.mjs -p xhyper-resiliencx` |
| async runtime wait / retry budget / stable | **DEFER** | residual OPEN |

## 重试退避合同（本仓）

- `RetryConfig::{base_delay_ms, backoff, jitter_bps}`
- `Backoff::Constant`：每次 `base_delay_ms`
- `Backoff::Exponential { factor, max_delay_ms }`：`min(base * factor^(attempt-1), max)`
- `jitter_bps`：确定性伪抖动（非加密 RNG）；`0` 关闭
- `retry_fn` 默认 `ThreadSleepWait`；`retry_fn_with_wait` 注入自定义 wait
- `base_delay_ms == 0` → 不 wait

## 熔断 / 限流 / 舱壁

见历史合同：三态熔断、令牌桶、`Bulkhead` RAII；均无墙钟冷却/自动 refill/排队超时。

## 验证

```bash
cargo test -p xhyper-resiliencx --all-targets
cargo clippy -p xhyper-resiliencx --all-targets -- -D warnings
node scripts/cov-gate-100.mjs -p xhyper-resiliencx --filter crates/resiliencx/src
```
