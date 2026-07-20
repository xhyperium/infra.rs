# AGENTS.md — AI Agent Governance for infra.rs

本仓库是**独立的 Rust 基础设施工作区**。所有 AI 编码助手（Claude Code、Codex、Copilot 等）共享本文件中的治理约定。

## 语言与编码（强制）

- **字符编码**：全部文本文件使用 **UTF-8（无 BOM）**，换行 **LF**
- **注释 / 文档**：统一使用**中文**（技术术语可保留英文）
- **用户可见错误信息**：中文
- **标识符**：英文（Rust 惯例）
- **LICENSE**：保留英文许可证原文
- 细则见 [docs/编码与语言约定.md](./docs/编码与语言约定.md)

## 项目身份

- **类型**：Rust Cargo workspace
- **根 crate 示例**：`crates/infra-core`
- **非目标**：不是其他产品的元仓库镜像；本地即为源码与约定的 SSOT

## 仓库结构

```
infra.rs/
├── crates/           # Rust workspace members
├── examples/         # 示例
├── tests/            # 集成测试
├── docs/             # 文档
├── scripts/          # Harness 健康检查 / GC / worktree 策略
├── .cargo/           # Cargo 配置、target-dir、工具缓存约定
├── .claude/          # Claude Code：skills / hooks / settings
├── .codex/           # Codex：agents / hooks
├── .github/          # CI/CD 与协作模板
├── AGENTS.md         # 本文件
├── CLAUDE.md         # Claude 专属指令
├── Cargo.toml        # Workspace 根
└── README.md
```

## 代理角色

| 系统 | 角色 | 技能来源 |
|------|------|----------|
| **Claude Code** | 主执行代理：编码、审查、交付 | `.claude/skills/` |
| **Codex** | 多模型编排与派工 | `.claude/skills/`（可投影到 `.agents/skills/`） |
| **Copilot** | 补充建议 | 自行管理 |

**SSOT**：技能定义以 `.claude/skills/` 为准；禁止在投影目录手工分叉维护。

## Beads 任务板

跨模型任务通过 Beads（`bd`）协作：

- **Claude Code**：SessionStart `bd prime` 只读注入；需要时 `bd update --claim` / `bd close`
- **Codex**：派工前 claim，验证后 close 或创建 follow-up
- **Copilot**：只读消费，不写 beads

本地 `.beads/` 已 gitignore，不进入版本库。

## 钩子系统

`.claude/settings.json` 生命周期钩子：

| 事件 | 钩子 | 用途 |
|------|------|------|
| SessionStart | `session-context.mjs` + `bd prime` | 上下文与任务记忆 |
| PreToolUse | `pre-tool-check` / `edit-guard` / `count-guard` | 编辑前校验 |
| PostToolUse | `post-tool-check` / `edit-guard-reset` / `link-check` | 编辑后检查 |
| PreCompact | `pre-compact.mjs` | 压缩前保留状态 |
| Stop | `session-review` / `version-guard` / `branch-protect` | 会话审查与护栏 |

## 构建与质量

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check
node scripts/check.mjs
```

- **target-dir**：`.cargo/target/`
- **alias**：`cargo xtask` → `infra-xtask`（crate 尚未添加时可忽略）
- **突变测试输出**：`.cargo/cache/mutants/`

## Git 规范

- 不在 `main` 上直接开发；走 feature 分支 + PR
- 禁止 `git push --force`、`git push --no-verify`（历史重写仅在用户明确要求且单独确认后执行）
- Conventional Commits

## 安全

- 不提交 `.claude/*.local.json`、证书、密钥、`.env`
- 不在日志/对话中回显完整 Token
- CI 使用 GitHub Secrets

## 常用 Skill

| 技能 | 场景 |
|------|------|
| `code-review` | 分支/PR 审查 |
| `codebase-design` / `design-an-interface` | 模块与 API 设计 |
| `diagnosing-bugs` | 故障与回归诊断 |
| `tdd` | 测试先行 |
| `domain-modeling` | 领域语言与 ADR |
| `harness-init` / `harness-mode` / `harness-gc` | Harness 管理 |
| `beads` | 任务板操作 |

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:970c3bf2 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See [SYNC_CONCEPTS.md](https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md) for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   bd dolt push
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

<!-- BEGIN BEADS CODEX SETUP: generated by bd setup codex -->
## Beads Issue Tracker

Use Beads (`bd`) for durable task tracking in repositories that include it. Use the `beads` skill at `.agents/skills/beads/SKILL.md` (project install) or `~/.agents/skills/beads/SKILL.md` (global install) for Beads workflow guidance, then use the `bd` CLI for issue operations.

### Quick Reference

```bash
bd ready                # Find available work
bd show <id>            # View issue details
bd update <id> --claim  # Claim work
bd close <id>           # Complete work
bd prime                # Refresh Beads context
```

### Rules

- Use `bd` for all task tracking; do not create markdown TODO lists.
- Run `bd prime` when Beads context is missing or stale. Codex 0.129.0+ can load Beads context automatically through native hooks; use `/hooks` to inspect or toggle them.
- Keep persistent project memory in Beads via `bd remember`; do not create ad hoc memory files.

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See [SYNC_CONCEPTS.md](https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md) for details and anti-patterns.
<!-- END BEADS CODEX SETUP -->
