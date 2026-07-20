# types/decimal — Goal 管线契约

> 实现 / 代码唯一位置：`crates/types/decimal`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-decimalx-complete-spec.md](spec/xhyper-decimalx-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — agent-safe 对账完成 · **未** Goal Achieved  
> **布局**：对齐 [`.agents/ssot/`](../../)（[AGENTS.md](../../../../AGENTS.md)）  
> **状态**：布局已对齐 · agent-safe 对账完成 · **未** Spec Approved

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | agent-safe 对账完成 · **未** Goal Achieved |
| Spec | [spec/spec.md](spec/spec.md) | **SSOT 入口**（Active 验收合同） |
| Design | [design/design.md](design/design.md) | 入口 / 占位 |
| Plan | [plan/plan.md](plan/plan.md) | agent-safe 战役 DONE · residual 仍开放 |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 入口 / 占位 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 / 占位 |
| **Code** | **`crates/types/decimal`** | 实现不在 `.agents/ssot/` |
| Test | [test/test.md](test/test.md) | 入口 / 占位 |
| Review | [review/review.md](review/review.md) | 默认 NOT PASS（人审 residual） |
| Release | [release/release.md](release/release.md) | 默认 BLOCKED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 入口 / 占位 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/](evidence/) |

## 硬限制

1. 无证据不得宣称 Done / 全闭合 / 5/5 / Spec Approved（除非既有战役文件已证明）。
2. 本树禁止 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
3. 布局迁移 **≠** 实现完成 **≠** package stable。
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-decimalx-complete-spec.md` 须 `cmp` 同构。

## 验证

```bash
cmp .agents/ssot/types/decimal/spec/spec.md \
    .agents/ssot/types/decimal/spec/xhyper-decimalx-complete-spec.md
# 结构：README + 11 层目录 + evidence/ 横切
test -f .agents/ssot/types/decimal/README.md
test -f .agents/ssot/types/decimal/spec/spec.md
cargo test -p xhyper-decimalx
```

**布局对齐：是 · agent-safe 对账：是 · Goal Achieved：否 · Spec Approved：否 · 禁止假 Done。**
