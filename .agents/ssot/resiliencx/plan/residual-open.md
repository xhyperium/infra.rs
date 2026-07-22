# Residual Open — resiliencx（infra.rs）

| 字段 | 值 |
|------|-----|
| 更新 | 2026-07-23 · round-01 |
| Active | [spec/spec.md](../spec/spec.md) |

> OPEN 不是失败；无证据宣称 CLOSED / DONE 才是失败。

## OPEN / DEFER

| ID | 摘要 | 标签 |
|----|------|------|
| OPEN-RETRY-AFTER | 服务端 Retry-After 策略 | DEFER |
| OPEN-REPORT | 结构化 execution report | DEFER |
| OPEN-DISTRIBUTED | 分布式 budget / 熔断 / 限流 / 舱壁协调 | DEFER |
| OPEN-QUEUE | 公平排队、排队 deadline 与超载策略 | DEFER |
| DEFER-STABLE | package stable / crates.io | HUMAN_ONLY |

## REJECTED（持续禁止）

| ID | 项 |
|----|-----|
| REJ-OBSERVEX | 本 crate 直接依赖 observex |
| REJ-BIZ | 业务状态机 / 领域校验进入本 crate |
| REJ-CANCEL-UNDO | 宣称 future cancellation 会撤销已发生外部副作用 |
| REJ-ATTEMPT-HERD | 宣称 attempt-only jitter 具备抗群聚保证 |

## CLOSED（agent-safe）

| ID | 说明 |
|----|------|
| RETRY-CORE-001 | 有限同步/异步 retry + 可注入 wait/backoff |
| RETRY-SAFETY-001 | `RetryContext`、显式安全分类与生产安全入口 |
| RETRY-BUDGET-001 | sync/async budget parity 与统一耗尽错误 |
| RETRY-DEADLINE-001 | Tokio 整次 retry deadline |
| RETRY-JITTER-001 | caller seed 入口与 attempt-only 边界 |
| BULKHEAD-POISON-001 | poisoned inner 恢复且 permit 不泄漏容量 |
| INSTR-CONTRACTS-001 | re-export `contracts::Instrumentation` |
| WS-MEMBER | workspace 成员 `crates/resiliencx` |
