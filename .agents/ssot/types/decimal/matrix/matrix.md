# types/decimal — Matrix

> **状态**：Active SSOT agent-safe 对账矩阵已维护 · **not** Goal Achieved · **not** Spec Approved  
> 追溯边横切；非 pipeline_state。

## 追溯边（2026-07-21）

| 边 | 从 | 到 | 状态 |
|----|----|----|------|
| Goal → Spec | `goal/goal.md` | `spec/spec.md` | Active 入口绑定 |
| Spec → Code | `spec/spec.md` | `crates/types/decimal` | 公开 API / 行为对齐 |
| Spec dual | `spec/spec.md` | `spec/xhyper-decimalx-complete-spec.md` | `cmp` 同构 |
| Plan residual | `plan/residual-open.md` | T-HUM / T-DEF / T-POL | **仍开放** |
| Review | `review/review.md` | 门禁 + residual | 默认 NOT PASS（人审） |
| Wire | `crates/types/decimal/docs/WIRE.md` | serde/text/DB | **非** stable |

## 条款摘要

| 条款族 | 实现 | 晋级 |
|--------|------|------|
| DEC-LAYER / REP / ROUND / FLOAT | ALIGNED | Active 验收 |
| MAX_SCALE 强制（代码） | ALIGNED | 取值批准 = HUMAN_ONLY |
| DEC-LIMIT / ERR / WIRE 治理 | OPEN | HUMAN_ONLY |
| DEC-DIV target_scale / 全 i128 oracle | OPEN | DEFERRED |

完整一一矩阵见本回合 scratch：`decimal-ssot-alignment.md`（会话证据；非 Goal Achieved）。

## 禁止

- 空目录批量标 DONE / READY / PASS
- 将 HUMAN_ONLY / DEFERRED / POLICY 伪标完成
