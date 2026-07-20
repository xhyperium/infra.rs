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

## 语言与编码

- 本仓库新增注释、文档优先使用**中文**
- 文本文件统一 **UTF-8**
