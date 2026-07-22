# configx — Goal 管线契约

> 实现 / 代码唯一位置：`crates/configx`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-configx-complete-spec.md](spec/xhyper-configx-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — **未宣称闭合**（无证据不得标 Done）  
> **布局**：对齐 [`.agents/ssot/kernel/`](../kernel/)（[AGENTS.md](../../../AGENTS.md) §2）  
> **当前轮次**：[round-03-findings.md](plan/round-03-findings.md) — 候选已重冻；本地 reviewer 完成，verifier 技术/证据初验完成
> **状态**：`0.1.2` 进程内手动 reload 合同已加固；发布门禁 BLOCKED

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | 入口存在 · 未宣称 AC 闭合 |
| Spec | [spec/spec.md](spec/spec.md) | **SSOT 入口**（布局迁移） |
| Design | [design/design.md](design/design.md) | 当前原子提交 / watch 设计 |
| Plan | [plan/plan.md](plan/plan.md) | Round 01/02/03 findings 已记录 |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 入口 / 占位 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 / 占位 |
| **Code** | **`crates/configx`** | 实现不在 `.agents/ssot/` |
| Test | [test/test.md](test/test.md) | 确定性加强与本地证据审查完成；GitHub CI artifact pending |
| Review | [review/review.md](review/review.md) | 本地独立 reviewer 完成；verifier 技术/证据初验完成 |
| Release | [release/release.md](release/release.md) | `0.1.2` 候选；BLOCKED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 阶段性复盘 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md)（BLOCKED） |
| Evidence | [evidence/](evidence/)（Round 01/02/03） |

## 硬限制

1. 无证据不得宣称 Done / 全闭合 / 5/5 / Spec Approved（除非既有战役文件已证明）。
2. 本树禁止 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
3. 布局迁移 **≠** 实现完成 **≠** package stable。
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-configx-complete-spec.md` 须 `cmp` 同构。
5. “reload”仅指调用方显式触发的进程内替换；禁止扩写为自动 watcher 或远端配置中心。

## 验证

```bash
cmp .agents/ssot/configx/spec/spec.md \
    .agents/ssot/configx/spec/xhyper-configx-complete-spec.md
# 结构：README + 11 层目录 + evidence/ 横切
test -f .agents/ssot/configx/README.md
test -f .agents/ssot/configx/spec/spec.md
```

**布局对齐：是 · Round 03：本地 reviewer 完成、verifier 技术/证据初验完成 · GitHub CI / 交付 gate pending · 战役全闭合：未宣称。**
