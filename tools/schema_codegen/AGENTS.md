# AGENTS.md — schema_codegen

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md) 与 [`../../CONSTITUTION.md`](../../CONSTITUTION.md)；冲突时以 CONSTITUTION.md 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- 生成保持确定性，相同输入必须产生逐字一致的标准输出。
- 命令只读取输入并输出源码，不直接覆盖仓库文件。
- 修改映射规则时为对应输入格式补充最小 fixture 测试。
