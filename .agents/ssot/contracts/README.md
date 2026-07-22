# contracts — Goal 管线契约

> 实现 / 代码唯一位置：`crates/contracts`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-contracts-complete-spec.md](spec/xhyper-contracts-complete-spec.md)
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — maintenance IN PROGRESS
> **布局**：对齐 [`.agents/ssot/kernel/`](../kernel/)（[AGENTS.md](../../AGENTS.md) §2）  
> **状态**：`0.1.2` maintenance 执行中；整体 Production Ready 未宣称

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | 入口存在 · 未宣称 AC 闭合 |
| Spec | [spec/spec.md](spec/spec.md) | **SSOT 入口**（布局迁移） |
| Design | [design/design.md](design/design.md) | maintenance 设计 |
| Plan | [plan/plan.md](plan/plan.md) | 入口；战役文件可并列于 plan/ |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 入口 / 占位 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 / 占位 |
| **Code** | **`crates/contracts`** | 实现不在 `.agents/ssot/` |
| Test | [test/test.md](test/test.md) | maintenance 测试策略 |
| Review | [review/review.md](review/review.md) | 验证前 NOT PASS |
| Release | [release/release.md](release/release.md) | 验证前 BLOCKED |
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
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-contracts-complete-spec.md` 须 `cmp` 同构。

## 验证

```bash
cmp .agents/ssot/contracts/spec/spec.md \
    .agents/ssot/contracts/spec/xhyper-contracts-complete-spec.md
# 结构：README + 11 层目录 + evidence/ 横切
test -f .agents/ssot/contracts/README.md
test -f .agents/ssot/contracts/spec/spec.md
```

**布局对齐：是 · 战役全闭合：未宣称 · 禁止假 Done。**
