# Alignment — decimalx 2026-07-17

| 维度 | 对齐结论 |
|------|----------|
| Active SSOT | `.agents/ssot/types/decimal/decimalx-spec.md` 仍为实现验收权威 |
| Candidate Draft | `.agents/ssot/types/decimal/20260717/*` · Status **Draft** · 不覆盖 Active |
| ADR | ADR-006 舍入/对齐/checked；ADR-007 层/唯一定义点 — 代码一致 |
| Package | `xhyper-decimalx` 0.1.0 · lib `decimalx` · path `crates/types/decimal` |
| Plan | `PLAN-TYPES-DECIMALX-002-agent-safe-v1` · agent-safe 闭合 |
| Todo | `.agents/ssot/types/decimal/todo.md` · disposition 台账 |
| Evidence | `plan/evidence/m0-consumer-inventory-2026-07-17.txt` · SCRATCH 测试/10x 日志 |
| Wire | serde 字段 shape = **当前事实** · **非**跨版本稳定承诺 |
| Goal/Spec 晋级 | **未**标 Achieved/Approved |
| PR / Approval | [#507](https://github.com/xhyperium/infra.rs/pull/507) · tip-bound SSOT = **SCRATCH** `approval-readback.json` · plan/evidence 仅为 POINTER_NOT_TIP_BOUND · **≠** Goal Achieved |
| Draft→Active 链接 | `20260717/*` 使用 `../decimalx-spec.md`（归位后相对路径） |

## 指针

- Goal: `20260717/decimalx-complete-goal.md`
- Spec: `20260717/decimalx-complete-spec.md`
- Gap: `plan/gap-matrix.md`
- Residual: `plan/residual-open.md`
- 10x: `plan/decimalx-plan-10x-verdict.md`

## 非宣称

不得从本文推出：package stable、金融 wire 已稳定、consumer 迁移完成、Draft=Approved。
