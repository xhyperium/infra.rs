# resiliencx

L1 **弹性**（重试 + 熔断 + 限流 + 舱壁；ADR-005）。

| 能力 | 类型 | 墙钟 |
|------|------|------|
| 重试 | `RetryConfig` / `retry_fn` | `base_delay_ms>0` 用 `thread::sleep`（已知差距） |
| 熔断 | `CircuitBreaker` 三态 | **无**；Open→HalfOpen 靠拒绝计数 |
| 限流 | `RateLimiter` 令牌桶 | **无**；调用方 `refill` |
| 舱壁 | `Bulkhead` / `BulkheadPermit` | **无**；满载立即 `Unavailable` |

**仍未交付**：async wait / backoff·jitter / retry budget / package stable。

## 公开面

| 项 | 说明 |
|----|------|
| `RetryConfig` / `retry_fn` / `retry_ok` / `retry_downcast` | 同步重试；仅 Transient 触发 |
| `CircuitConfig` / `CircuitBreaker` / `CircuitState` | 熔断；Open 拒绝 `Unavailable` |
| `RateLimitConfig` / `RateLimiter` | 令牌桶；不足 → `Unavailable` |
| `BulkheadConfig` / `Bulkhead` / `BulkheadPermit` | 并发舱壁；满载拒绝；RAII 归还 |
| `Instrumentation` | re-export `contracts::Instrumentation` |
| `NoopInstrumentation` | 空实现 |

## 依赖

- 生产：`xhyper-kernel` + `xhyper-contracts`
- **禁止**直接依赖 `observex`

## 验证

```bash
cargo test -p xhyper-resiliencx --all-targets
cargo clippy -p xhyper-resiliencx --all-targets -- -D warnings
node scripts/cov-gate-100.mjs -p xhyper-resiliencx --filter crates/resiliencx/src
```

## SSOT

`.agents/ssot/infra/resiliencx/spec/spec.md`（用户路径别名 `.agent/ssot/resiliencx`）。  
本仓对齐：[docs/ssot/resiliencx-ssot-alignment.md](../../docs/ssot/resiliencx-ssot-alignment.md)。

## 版本

0.1.0 · **≠** package stable / crates.io。
