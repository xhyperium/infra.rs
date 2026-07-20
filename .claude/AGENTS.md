# Claude Code Agents

xhyper.rs 仓库的 Claude Code 代理配置。

> 项目治理与行为规范详见仓库根 [AGENTS.md](../../AGENTS.md) 和 [CLAUDE.md](../../CLAUDE.md)。

## 目录结构

- `hooks/` — 生命周期钩子（branch-protect / edit-guard / session-review / version-guard 等）
- `skills/` — 技能（SSOT，Codex 投影到 `.agents/skills/`）
- `providers/` — LLM provider 配置（gitignored）
- `settings.json` — 钩子激活配置（无凭据，可提交）
- `reviews/` — 会话审查日志（gitignored）
- `.edit-guard-state.json` — 编辑守卫状态（gitignored）

## Harness 系统

本仓库附带 Harness 工程系统（钩子 + 技能 + 审查循环 + Beads 任务板接入），详见仓库根 `AGENTS.md`。
