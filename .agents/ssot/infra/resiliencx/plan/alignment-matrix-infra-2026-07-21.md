# Alignment Matrix — infra.rs `resiliencx` 1:1

| 字段 | 值 |
|------|-----|
| Matrix ID | `ALIGN-INFRA-RESILIENCX-20260721` |
| Scope | `.agents/ssot/infra/resiliencx/**` ↔ `crates/infra/resiliencx` |
| 更新 | 2026-07-23 · round-01 |

## A. 公开 API

| Claim | Live | 状态 |
|-------|------|------|
| Package/lib `resiliencx` | `Cargo.toml` + workspace members | MATCH |
| 有限 sync/async retry | `retry.rs` + contract tests | MATCH |
| `RetryContext` + `RetrySafety` 生产安全入口 | `retry_fn_safe` / `retry_async_safe` | MATCH |
| async budget parity | `retry_async_with_budget` / `retry_async_safe` | MATCH |
| 整次 retry deadline | `retry_async_with_deadline`（feature `tokio`） | MATCH |
| caller-seeded jitter | `RetryContext::with_jitter_seed` + 纯计算 helper | MATCH |
| `Instrumentation` 注入 | re-export `contracts::Instrumentation` | MATCH |
| circuit / limiter / bulkhead | 对应模块 + tests | MATCH |

## B. 依赖 / 禁令

| Claim | 状态 |
|-------|------|
| `kernel` + `contracts` + `async-trait`；Tokio 可选 | MATCH |
| 无 observex | MATCH / POLICY |
| 不反向依赖 transport/domain/app | MATCH |

## C. 诚实边界

| Claim | 状态 |
|-------|------|
| 低层兼容 retry API 不校验 safety | MATCH |
| deadline cooperative cancellation 不撤销外部副作用 | MATCH |
| attempt-only jitter 不抗群聚 | MATCH |
| circuit/rate-limit/bulkhead 为本地原语；资源不足立即拒绝 | MATCH |
| package stable / 分布式协调 | OPEN / HUMAN |

## D. 验证

当前 round-01 验证证据见 [`round-01-findings.md`](round-01-findings.md)。
