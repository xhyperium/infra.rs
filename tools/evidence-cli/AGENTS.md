# AGENTS.md — evidence-cli

> 完整行为准则与架构约束以仓库根 [AGENTS.md](../../AGENTS.md) 与 [CONSTITUTION.md](../../CONSTITUTION.md) 为准；两者冲突以 CONSTITUTION.md 为准。

## 分支纪律

严禁在 `main` 上开发；编辑前确认位于 worktree 或 feature branch。

## 本 crate 约束

- **SPEC-EVIDENCE-002 §25**：只读 evidence chain CLI。
- **依赖白名单**：`clap` / `kernel` / `evidence` / `evidence_file` / `serde` / `serde_json`；不得引入 `tokio` / `sqlx` / 其他 adapter。
- **默认只读**：`verify` / `head` / `inspect` / `export` / `vectors` 命令不修改存储。
- **repair-tail** 需 `--confirm` 显式确认；缺 `--confirm` 时拒绝执行。
- **退出码**：0 success/valid · 2 invalid arguments · 3 chain invalid · 4 checkpoint/signature invalid · 5 storage unavailable · 6 unsupported version · 7 repair required（incomplete tail）。
- `publish = false`：当前仅内部工具，未发布 crates.io。

## 验证

```bash
cargo fmt -p xhyper-evidence-cli
cargo test -p xhyper-evidence-cli
cargo clippy -p xhyper-evidence-cli --all-targets -- -D warnings
```
