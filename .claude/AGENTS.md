# Claude Code 代理配置

`infra.rs` 仓库的 Claude Code 代理配置。

> 项目治理与行为规范见仓库根 [AGENTS.md](../AGENTS.md) 与 [CLAUDE.md](../CLAUDE.md)。

## 目录结构

- `hooks/` — 生命周期钩子（branch-protect / edit-guard / session-review / version-guard 等）
- `skills/` — 技能定义（SSOT；可投影到 `.agents/skills/`）
- `settings.json` — 钩子激活配置（无凭据，可提交）
- `reviews/` — 会话审查日志（gitignore）
- `.edit-guard-state.json` — 编辑守卫状态（gitignore）
- `*.local.json` — 本地 Provider 配置（gitignore，含 Token）

## Harness

本仓库附带 Harness 工程系统（钩子 + 技能 + 审查循环 + Beads 任务板），详见根目录 `AGENTS.md`。

## 语言与编码

- 新增/修改的注释与文档使用**中文**
- 文本文件编码统一 **UTF-8（无 BOM）**
