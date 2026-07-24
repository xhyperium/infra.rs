# transport — Goal 管线契约

> 实现 / 代码唯一位置：`crates/infra/transport`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-transportx-complete-spec.md](spec/xhyper-transportx-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — `0.1.4` **IMPLEMENTED CANDIDATE**
> **布局**：对齐 [`.agents/ssot/kernel/`](../kernel/)（[AGENTS.md](../../../AGENTS.md) §2）  
> **状态**：实现与本地 workspace 门禁已运行；固定代码证据由
> [`manifest.json`](../../../evidence/testkit/2026-07-23-infra-2d9.10/manifest.json) 绑定；
> PR CI、独立终审、人工批准与 merge 均为 OPEN

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | 候选目标 |
| Spec | [spec/spec.md](spec/spec.md) | **active SSOT** |
| Design | [design/design.md](design/design.md) | 候选设计 |
| Plan | [plan/plan.md](plan/plan.md) | 三轮收敛计划 |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 本地完成 + 交付待办 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 最终交付上下文 |
| **Code** | **`crates/infra/transport`** | 实现不在 `.agents/ssot/` |
| Test | [test/test.md](test/test.md) | 本地证据由 manifest 绑定 |
| Review | [review/review.md](review/review.md) | PENDING |
| Release | [release/release.md](release/release.md) | NOT RELEASED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 候选阶段复盘 |

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
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-transportx-complete-spec.md` 须 `cmp` 同构。

## 验证

```bash
cmp .agents/ssot/infra/transport/spec/spec.md \
    .agents/ssot/infra/transport/spec/xhyper-transportx-complete-spec.md
# 结构：README + 11 层目录 + evidence/ 横切
test -f .agents/ssot/infra/transport/README.md
test -f .agents/ssot/infra/transport/spec/spec.md
```

**布局对齐：是 · 战役全闭合：未宣称 · 禁止假 Done。**
