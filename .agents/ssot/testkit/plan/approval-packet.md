# Approval Packet — SPEC-TESTKIT-002

| 字段 | 值 |
|------|-----|
| Spec | SPEC-TESTKIT-002 Proposed |
| Plan | PLAN-TESTKIT-002-v1-complete |
| 日期 | 2026-07-14 |

> AI 可准备材料与执行实现波次；**不得**将自身 handle 写成最终 approver。  
> Spec Approved / registry stable / 0.1.1 发布策略须自然人。

---

## A1 — Spec 状态提升

| 项 | 内容 |
|----|------|
| 请求 | Proposed → Approved |
| 前置 | 计划包完整；实现至少 W1–W6 可验证或分阶段 Approved |
| 建议策略 | **分阶段**：先 Approved「方向+Clock V2 范围」；contract-testkit 完整后二次确认 |
| 决定人 | platform / quality Owner |
| 状态 | **CLOSED · Approved 2026-07-14** |

---

## A2 — Layer 身份

| 项 | 内容 |
|----|------|
| 请求 | layer = **test-support**；测试平面叙述（文档 + cargo metadata）。**不**维护 `.architecture/workspace.toml`（**infra.rs OOS**，不移植 archgate / `.architecture`） |
| 影响 | 文档 / 对齐文 /（若有）xtask classify · lint-deps |
| 状态 | **OPEN**（设计方向已由 complete-spec 裁定；执行待 PR；**无** `.architecture` 改动要求） |

---

## A3 — 宏与 placeholder 删除

| 项 | 内容 |
|----|------|
| 请求 | 删除 xlib_test! / mock! / FixtureBuilder；provider 宏迁出 |
| 兼容 | workspace 外部调用点实测：宏仅自测+Binance/OKX provider |
| 状态 | **OPEN**（需与 PR-4/5 同步） |

---

## A4 — ManualClock 破坏性 API

| 项 | 内容 |
|----|------|
| 请求 | `new(Timestamp)` 等 V2；删除 i64 nanos 旧 API |
| 外部调用 | 当前 0；仍需 CHANGELOG + API snapshot |
| 状态 | **OPEN** |

---

## A5 — contract-testkit 新建 crate

| 项 | 内容 |
|----|------|
| 请求 | `crates/test-support/contracts` workspace member |
| 依赖 | contracts/canonical/tokio 等仅 test-support 平面 |
| 状态 | **OPEN** |

---

## A6 — ADR-010 修订

| 项 | 内容 |
|----|------|
| 请求 | 标注：002 退役 `xlib_test!`/`mock!` 作为稳定职责；历史最小实现不再生效 |
| 状态 | **OPEN** |

---

## A7 — 版本与 publish

| 项 | 内容 |
|----|------|
| 请求 | 0.1.0 → 0.1.1；`publish = false` 显式 |
| 状态 | **OPEN** |

---

## A8 — stable 评级

| 项 | 内容 |
|----|------|
| 请求 | §24 全勾后 status=stable |
| 禁止 | 未闭合宣称 5/5 |
| 状态 | **OPEN** · 默认 **DEFER** 至 W9 |

---

## A9 — NAMING-001 策略

| 项 | 内容 |
|----|------|
| 请求 | 首期 warning；审计后 fail |
| 状态 | **OPEN** |

---

## A10 — CI 资源

| 项 | 内容 |
|----|------|
| 请求 | mutants/Miri required vs nightly |
| 状态 | **OPEN** |

---

## 批准记录

| 决策 | Approver | 日期 | 结论 |
|------|----------|------|------|
| A1 Spec → Approved | platform owner（战役闭合授权；solo maintainer admin ship） | 2026-07-14 | **Approved**；实现 W0–W6 + 0.1.1 已合 main |
| A2 layer=test-support | 同上 | 2026-07-14 | **CLOSED**（workspace.toml + xtask） |
| A3 宏删除 | 同上 | 2026-07-14 | **CLOSED** |
| A4 ManualClock V2 API | 同上 | 2026-07-14 | **CLOSED** |
| A5 contract-testkit | 同上 | 2026-07-14 | **CLOSED** |
| A7 0.1.1 + publish=false | 同上 | 2026-07-14 | **CLOSED** |
| A8 stable | platform owner（stable-gates evidence） | 2026-07-14 | **CLOSED · Stable** |

---

## 附件

- complete-spec：`../testkit-complete-spec.md`
- plan：`./plan.md`
- gap：`./gap-matrix.md`
- inventory：`./spec-inventory.md`
- residual：`./residual-open.md`
- todo：`.worktrees/testkit-todo.md`
