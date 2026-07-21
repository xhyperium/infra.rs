# AGENTS.md — archgate

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md) 与 [`../../CONSTITUTION.md`](../../CONSTITUTION.md)；冲突时以 CONSTITUTION.md 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- 门禁保持只读、确定性；同一提交的输出必须稳定。
- 新增阻断规则必须有已批准的架构依据，并同时提供正反例测试。
- 例外必须显式记录；不得通过扩大 allowlist 隐藏未知违规。
