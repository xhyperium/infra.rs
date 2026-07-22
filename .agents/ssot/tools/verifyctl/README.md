# tools/verifyctl — 最小验证 CLI（本仓扩展）

> 实现：`tools/verifyctl`，workspace member `verifyctl` `0.1.0`。
> Active spec：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-verifyctl-complete-spec.md](spec/xhyper-verifyctl-complete-spec.md)。
> 当前能力：plan / execute / report 最小闭环。
> 裁定：**非生产 verifier**，不得把最小 CLI 或布局 COMPLETE 解释为完整验证控制面已闭合。

## 11 层映射

| 管线层 | 路径 | 当前状态 |
|--------|------|----------|
| Goal | [goal/goal.md](goal/goal.md) | 目标正文；未宣称全部 AC 闭合 |
| Spec | [spec/spec.md](spec/spec.md) | active 规格入口 |
| Design | [design/design.md](design/design.md) | 设计入口 |
| Plan | [plan/plan.md](plan/plan.md) | 计划入口 |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 任务入口 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | Prompt 入口 |
| **Code** | **`tools/verifyctl`** | workspace member；实现不在 SSOT 树 |
| Test | [test/test.md](test/test.md) | 证据入口 |
| Review | [review/review.md](review/review.md) | 不因实现存在自动 PASS |
| Release | [release/release.md](release/release.md) | 生产 verifier release 仍 BLOCKED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 复盘入口 |

横切制品位于 [matrix/](matrix/)、[gate/](gate/) 与 [evidence/](evidence/)。

## 可观察实现

- `plan`：由 Goal Contract 和 changed paths 生成 `verification-plan/v1`。
- `execute`：在指定 cwd 执行计划内检查，生成 `verification-run/v1`。
- `report`：聚合并可写出 RunResult。
- package 版本为 `0.1.0`，可选 `with-evidence` feature。

## OPEN / 禁止生产声明

- 输入 schema/digest/check 非空的完整 fail-closed 校验；
- 真正执行 `timeout_secs` 的取消与子进程清理；
- 远程 runner、签名证据链、完整 V0–V3 矩阵与稳定机器错误契约；
- evidence-required 模式的追加失败闭锁；
- package stable、Production Ready 或 Agent L5。

这些 OPEN 项未闭合前，`verifyctl` 只能称为最小 CLI，不能作为生产 verifier 或发布 Gate。

## 验证

```bash
cargo metadata --no-deps --format-version 1 | \
  jq -e '.packages[] | select(.name == "verifyctl" and .version == "0.1.0")'
cargo test -p verifyctl --all-targets
cmp .agents/ssot/tools/verifyctl/spec/spec.md \
  .agents/ssot/tools/verifyctl/spec/xhyper-verifyctl-complete-spec.md
```
