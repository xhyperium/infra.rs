# AGENTS.md — marketd

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md) 与 [`../../CONSTITUTION.md`](../../CONSTITUTION.md)；冲突时以 CONSTITUTION.md 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- 保持二进制组合根：领域能力放在 `market_data`，此处只做装配与进程生命周期管理。
- 真实 provider、外部凭据和生产部署变更须单独审批，不以 fixture 替代生产实现。
- 关闭和检查点语义变更必须保留进程级重启测试。
