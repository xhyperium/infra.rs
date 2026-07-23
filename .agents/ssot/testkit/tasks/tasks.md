# Tasks — GOAL-DETERMINISTIC-TEST-SUPPORT

| 字段 | 值 |
|------|-----|
| Campaign | **COMPLETE** · ship 2026-07-14 · main@testkit-v0.1.1 |
| Residual OPEN | **0 阻塞**（DEF-001…010 全 CLOSED · 1 OPTIONAL branch cov） |

## 终态（W0–W9）

| Wave / Task | 状态 | 证据 |
|-------------|------|------|
| W0 计划/冻结/计划 10× | **done** | PR [#247](https://github.com/xhyperium/infra.rs/pull/247) |
| W1 ManualClock V2 | **done** | PR [#254](https://github.com/xhyperium/infra.rs/pull/254) |
| W2 时钟消费者迁移 | **done** | PR #254 |
| W3 删宏/FixtureBuilder | **done** | PR #254 |
| W4 contract-testkit | **done** | PR #254 · package `contract-testkit` |
| W5 layer=test-support / SSOT 对齐 | **done** | PR #254 · `workspace.toml` |
| W6 test-graph-check / 图隔离 CI | **done** | PR #254 · `cargo xtl test-graph-check`（**archgate OOS**：本仓不移植） |
| W7 十轮实现验收 | **done** | fail_rounds=0 · mutants missed=0 · Miri PASS |
| W8 人审（A1–A10） | **done** | [plan/approval-packet.md](../plan/approval-packet.md) |
| W9 §24 闭合 / Stable 决策 | **done** | Stable CLAIMED 2026-07-14 |
| 0.1.1 release | **done** | PR [#255](https://github.com/xhyperium/infra.rs/pull/255) · tag `testkit-v0.1.1` |

> 完整任务明细见 [plan/tasks.md](../plan/tasks.md)（按 Wave 分块的原子任务表 + v1.1/v1.2 补丁）。

## Residual OPEN

**无阻塞。** DEF-001…010 全 CLOSED。仅 1 OPTIONAL（branch cov ≥90%，line≥95% 已强制），见 [plan/residual-open.md](../plan/residual-open.md)。

**Campaign COMPLETE。破坏性改动走新 spec 版本。**

## Maintenance M-CTK-012（2026-07-23）

| Task | 状态 |
|------|------|
| 14 trait reference suites + 15 broken cases | implementation complete；待独立复审 |
| FixtureNamespace / smoke-observed 语义边界 | implementation complete；待复审 |
| cargo metadata production graph gate | implementation complete；待 CI |
| public API baseline / crate version / SSOT | implementation complete；待最终 main 重放 |
| PR / 人工批准 / 合并 / cleanup | pending |
