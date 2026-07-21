# plans/ — 跨 crate 执行计划

| 目录职责 | 说明 |
|----------|------|
| 放什么 | 可执行的多 crate / 多波次修复与晋级计划（含验收与门禁） |
| 不放什么 | 一次性审计报告（→ `docs/report/`）、长期治理条文（→ `docs/governance/`）、域对齐矩阵（→ `docs/ssot/`） |

## 索引

| 计划 | 状态 | 说明 |
|------|------|------|
| [2026-07-21-core-crates-production-readiness.md](./2026-07-21-core-crates-production-readiness.md) | Draft | 五核心 crate 生产级修复（W0–W5）；输入审计 `docs/report/2026-07-21/` |

## 约定

- 计划 ID 形如 `PLAN-<AREA>-<NNN>`
- 完成后在计划文内更新状态与变更记录；不删除历史波次
- 实现必须在 `.worktrees/<branch>` 内进行；本目录变更同样走 PR
