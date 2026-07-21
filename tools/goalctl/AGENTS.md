# AGENTS.md — goalctl

> 完整行为准则与架构约束以仓库根 [AGENTS.md](../../AGENTS.md) 与 [CONSTITUTION.md](../../CONSTITUTION.md) 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 feature branch 或 worktree。

## 本 crate 约束

- **Phase 1 只读**：`doctor` / `index` / `version`；不得写业务代码或改 Task Pack。
- **Authority**：rank 来自 `docs/goal/schema/authority-policy.yaml`，禁止硬编码 SSOT（PR-2 完整解析）。
- **控制面**：禁止创建或依赖 `.config/goal`。
- **state-dir**：`${XDG_STATE_HOME}/xhyper-goalctl/<repo-id>/` 或 `--state-dir`；禁止 Cargo target。
- **输出**：`--json` 时 stdout 仅 JSON；路径仓库相对；确定性序列化。
- **Diagnostic**：`GC-*` 不是 G0–G11 Gate。
- **`publish = false`**。
- 合同：`.agent/SSOT/tools/goalctl/contracts/**` 与 `schemas/**`。

## 验证

```bash
cargo test -p xhyper-goalctl
cargo clippy -p xhyper-goalctl --all-targets -- -D warnings
cargo xtl lint-deps
```
