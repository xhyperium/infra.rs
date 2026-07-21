# Residual Open — resiliencx（infra.rs）

| 字段 | 值 |
|------|-----|
| 更新 | 2026-07-21 |
| Active | [spec/spec.md](../spec/spec.md) |

> OPEN 不是失败；**假装 CLOSED / DONE** 才是失败。

## OPEN / DEFER

| ID | 摘要 | 标签 |
|----|------|------|
| OPEN-ASYNC-WAIT | injected async wait / ManualClock 可控 delay | DEFER |
| OPEN-BACKOFF | backoff / jitter / Retry-After | DEFER |
| OPEN-BUDGET | retry budget / execution report | DEFER |
| OPEN-CIRCUIT | circuit breaker | DEFER |
| OPEN-LIMITER | rate limiter / bulkhead | DEFER |
| OPEN-IDEM | operation idempotency / safety 分类 | DEFER |
| DEFER-STABLE | package stable / crates.io | HUMAN_ONLY |

## REJECTED（持续禁止）

| ID | 项 |
|----|-----|
| REJ-OBSERVEX | 本 crate 直接依赖 observex |
| REJ-BIZ | 业务状态机 / 领域校验进本 crate |

## CLOSED（agent-safe）

| ID | 说明 |
|----|------|
| RETRY-CORE-001 | RetryConfig + retry_fn §2 行为 |
| INSTR-LOCAL-001 | Instrumentation 本地 trait（无 contracts） |
| COV-100 | llvm-cov Lines 100% |
| WS-MEMBER | workspace 成员 `crates/resiliencx` |
