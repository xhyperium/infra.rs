# Codex 代理配置

`infra.rs` 仓库的 Codex 代理配置。

## 目录结构

- `config.toml` — Codex 特性与多代理并发配置（本地，可能被 gitignore）
- `hooks.json` — SessionStart 等钩子（本地，可能被 gitignore）
- `agents/` — 项目级自定义 agent 角色定义

## 技能 SSOT

| 角色 | 路径 |
|------|------|
| **技能唯一真相** | **`.claude/skills/<name>/`** |
| Codex 发现投影 | `.agents/skills/<name>`（投影目录，禁止手工分叉） |

说明：

- Codex CLI 默认扫描 `$REPO_ROOT/.agents/skills`
- 人工编辑只改 `.claude/skills/`，再按需同步投影
- Beads：派工前 `bd update --claim`，验证后 `bd close` 或创建 follow-up

## Worktree 安全约束

Codex 在本仓工作时**必须**遵守 worktree 策略（宪章 §6.0.5）：

| 约束 | 说明 |
|------|------|
| **必须 worktree 开发** | `node scripts/worktree/worktree.mjs create <type>/<id>-<slug>` → `cd .worktrees/…` |
| **禁止主仓 Write/Edit** | 已跟踪文件的编辑必须在 worktree 内 |
| **禁止 main 上 commit** | HEAD 为 main 时不得 `git commit` |
| **禁止 force push** | 任何分支都不得 `git push --force` |
| **禁止 bypass** | 不得自行设置 `INFRA_WORKTREE_BYPASS=1`（仅 maintainer 应急） |
| **危险命令** | `rm -rf`、`git reset --hard`、`git clean -fd` 仅限 `.worktrees/` 或 `/tmp/` 内 |

SessionStart 钩子 `codex-worktree-guard.mjs` 会注入当前 worktree 状态；
UserPromptSubmit 钩子会扫描危险命令模式并告警。

> Codex 不支持 PreToolUse block，上述约束为 advisory 级别。
> Codex 的 sandbox 权限（workspace-write / read-only）在 `config.toml` 和 agent 定义中控制。

## 语言与编码

- 本仓库新增注释、文档优先使用**中文**
- 文本文件统一 **UTF-8**
