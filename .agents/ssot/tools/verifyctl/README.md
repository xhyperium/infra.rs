# tools/verifyctl — Goal 管线契约（本仓扩展）

> 实现 / 代码唯一位置：`tools/verifyctl`（**本仓尚未创建**）  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-verifyctl-complete-spec.md](spec/xhyper-verifyctl-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — **未宣称闭合**（无证据不得标 Done）  
> **布局**：对齐 [`.agents/ssot/kernel/`](../../kernel/)（[AGENTS.md](../../../../AGENTS.md) §2）  
> **状态**：布局已对齐 kernel · **本仓扩展域**（上游 `tools/` 无 verifyctl）· 战役内容未宣称闭合

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | 生产级 Goal 正文已入树 · 未宣称 AC 闭合 |
| Spec | [spec/spec.md](spec/spec.md) | **SSOT 入口** |
| Design | [design/design.md](design/design.md) | 入口 / 占位 |
| Plan | [plan/plan.md](plan/plan.md) | 入口 / 占位 |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 入口 / 占位 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 / 占位 |
| **Code** | **`tools/verifyctl`** | 实现不在 `.agents/ssot/`；**本仓无 member** |
| Test | [test/test.md](test/test.md) | 入口 / 占位 |
| Review | [review/review.md](review/review.md) | 默认 NOT PASS |
| Release | [release/release.md](release/release.md) | 默认 BLOCKED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 入口 / 占位 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/](evidence/) |

## 硬限制

1. 无证据不得宣称 Done / 全闭合 / Spec Approved。
2. 本树禁止 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
3. 布局迁移 **≠** 实现完成 **≠** package stable。
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-verifyctl-complete-spec.md` 须 `cmp` 同构。
5. 全量 `rsync --delete` 上游 `tools/` 会删除本域；同步后须从 git 恢复或重跑本地化步骤。

## 验证

```bash
cmp .agents/ssot/tools/verifyctl/spec/spec.md \
    .agents/ssot/tools/verifyctl/spec/xhyper-verifyctl-complete-spec.md
test -f .agents/ssot/tools/verifyctl/README.md
test -f .agents/ssot/tools/verifyctl/goal/goal.md
test -f .agents/ssot/tools/verifyctl/spec/spec.md
# 本仓无 crate：以下应失败直至落地
# cargo metadata --no-deps | jq -e '.packages[] | select(.name=="verifyctl")'
```

**布局对齐：是 · 战役全闭合：未宣称 · 禁止假 Done。**
