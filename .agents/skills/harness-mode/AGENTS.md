# .agent/skills/harness-mode/AGENTS.md

`harness-mode` 技能：切换 Harness 工作 mode（full/hotfix/tweak）与开发 phase（design/build/fix）。

- 状态持久化在 `.agent/.harness-state`（`{"phase","mode","since"}`）。
- `hotfix` 模式跳过行数/文件数检查，自动设 `phase=fix`；`tweak` 模式仅保护 `.env`，自动设 `phase=design`。

详见 `SKILL.md`。
