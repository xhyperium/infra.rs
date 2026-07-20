# types/decimal — Goal

> **状态**：Active 合同 agent-safe 基线已落地并对账 · **未** Goal Achieved · **未** Spec Approved  
> Source Goal 候选：`20260717/xhyper-decimalx-complete-goal.md`（Draft，非权威）

## 目标摘要

在 `crates/types/decimal`（package `xhyper-decimalx`）落地 ADR-006/007 十进制数值类型，
并以 Active SSOT `spec/spec.md` 为验收合同。

## 当前判定（2026-07-21 / infra.rs）

| 维度 | 判定 |
|------|------|
| 实现路径 | `crates/types/decimal` · lib `decimalx` · `0.1.0` |
| Active 合同可行使 | **是**（表示、五策略、checked 四则/rescale、Eq/Ord/Hash、Currency/Money、serde 字段 shape、MAX_SCALE 生产强制） |
| agent-safe 对账 | **完成**（见 `plan/CURRENT-STATE.md`、scratch alignment 矩阵） |
| Goal Achieved | **否**（T-HUM-005；含人审 residual） |
| Spec Approved | **否** |
| Wire stable | **否**（`docs/WIRE.md`） |

## 禁止

- 将 Draft / residual 人审项伪标 Achieved / Approved / READY
- 无门禁证据宣称 Done
