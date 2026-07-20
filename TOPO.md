# 初始化后续待办（可选）

本文件记录独立仓库初始化后可继续推进的事项，使用 **UTF-8 / 中文**。

1. 若需覆盖远程旧历史：在确认后执行  
   `git push --force-with-lease origin main`
2. 按业务需要扩展 `crates/` 成员
3. ~~（可选）安装 LSP~~ **已完成**：`typescript-language-server@5.3.0` + `typescript@7`（全局）  
   配置见 `.lsp.json`（mjs / js / json）
4. （可选）接入 OpenSpec 规范驱动工作流 — **待你确认后再开**
5. 按需增减 `CLAUDE.md` 中的 Skill 路由表 — **待你给出增减清单**
