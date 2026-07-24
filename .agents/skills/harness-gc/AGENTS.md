# .agent/skills/harness-gc/AGENTS.md

`harness-gc` 技能：Garbage Collection Agent，定期扫描项目健康状态，检测文档/代码一致性漂移，自动发起修复提案。

- 依赖 `scripts/gc-scan.mjs` 作为外部确定性验证门（Sniff 模式），不完全依赖 AI 判断。
- 连续 3 次扫描无改善即触发熔断，停止自动重试。
- 所有修复建议必须经用户审查后才能应用，不得自行合并。

详见 `SKILL.md`。
