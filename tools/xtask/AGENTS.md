# AGENTS.md — xtask

> 完整行为准则与架构约束以仓库根 [AGENTS.md](../../AGENTS.md) 与 [CONSTITUTION.md](../../CONSTITUTION.md) 为准；两者冲突以 CONSTITUTION.md 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- 二进制 crate（无 lib.rs），不发布到 crates.io。
- `lint-deps` 的校验规则必须与 `docs/architecture/spec.md` §2 R1–R6 同步：修改规则时同步改 spec，反之亦然。
- 内部模块 `allowed_matrix` / `classify` / `lint_deps` 为私有，不对外暴露。
- 新增子命令遵循 `clap` derive 风格，保持与现有命令一致的输出格式。
