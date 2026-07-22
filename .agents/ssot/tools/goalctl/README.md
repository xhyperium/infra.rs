# tools/goalctl — 最小 Goal→Contract CLI

> 实现：`tools/goalctl`，workspace member `goalctl` `0.2.0`。
> Active spec：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-goalctl-complete-spec.md](spec/xhyper-goalctl-complete-spec.md)。
> 当前裁定：最小 CLI 已实现；非完整 authority plane、Agent 或 required CI cutover。

## 可观察实现

| 表面 | 当前行为 |
|------|----------|
| `doctor` | 输出版本、当前支持命令与状态信息 |
| `validate <goal>` | 解析 YAML/JSON GoalDocument 并执行最小 fail-closed 校验 |
| `compile <goal> -o <path>` | 先校验，再输出 `goal-contract/v1` 与稳定 digest |
| library | `compile_goal` / `compile_goal_str` / `validate_goal` / `VERSION` |

实现和测试位于 `tools/goalctl/{src,tests}`；SSOT 树不保存 Rust 源码副本。

## 证据与历史材料的关系

本目录内 PR-0A、DECISION-PACK、contracts/schemas 与 Phase 文档是规划/历史战役输入。它们可以约束后续设计，但不能把当前 `0.2.0` 最小 CLI 扩大解释为 resolve/reconcile/index、完整 monorepo authority、native required Gate 或 package stable 已实现。

## OPEN / 禁止声明

- 完整 authority rank/引用闭包与跨模块 reconcile；
- unknown fields、引用循环、全 schema registry 兼容矩阵的生产闭锁；
- Agent daemon、required CI cutover、远程状态与签名发布链；
- package stable、Production Ready 或 Agent L5。

## 验证

```bash
cargo metadata --no-deps --format-version 1 | \
  jq -e '.packages[] | select(.name == "goalctl" and .version == "0.2.0")'
cargo test -p goalctl --all-targets
cargo run -p goalctl -- doctor
cargo run -p goalctl -- validate tools/goalctl/tests/fixtures/good_goal.yaml
cargo run -p goalctl -- compile tools/goalctl/tests/fixtures/good_goal.yaml -o /tmp/contract.json
cmp .agents/ssot/tools/goalctl/spec/spec.md \
  .agents/ssot/tools/goalctl/spec/xhyper-goalctl-complete-spec.md
```

完成条件只覆盖上述最小表面；历史 Approved/COMPLETE 不自动改变 current-state。
