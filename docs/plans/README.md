# plans/ — 跨 crate 执行计划

| 目录职责 | 说明 |
|----------|------|
| 放什么 | 可执行的多 crate / 多波次修复与晋级计划（含验收与门禁）；发布签核包 |
| 不放什么 | 一次性审计报告（→ `docs/report/`）、长期治理条文（→ `.agents/rules/`）、域对齐矩阵（→ `docs/ssot/`） |

## 索引

| 计划 | 状态 | 说明 |
|------|------|------|
| [2026-07-21-core-crates-production-readiness.md](./2026-07-21-core-crates-production-readiness.md) | W0–W5 已合入 · **L5 GO-with-Accepts** · 计划 DONE | 五核心 crate 生产级修复；PR #120–#128；输入审计 `docs/report/2026-07-21/`（含 §12 post-W5） |
| [releases/0.3.0-signoff.md](./releases/0.3.0-signoff.md) | **SIGNED · GO-with-Accepts** | 正式签核包（DEFER-7 已闭；`@ZoneCNH` · 2026-07-21） |
| [releases/2026-07-21-four-crates-internal-release.md](./releases/2026-07-21-four-crates-internal-release.md) | **DRAFT · GO for declared tiers** | kernel/testkit/decimalx/canonical 四包内部生产证据（L1/L2/L4）；非 crates.io |
| [releases/2026-07-21-FOUR-CRATES-MAINTAINER-CHECKLIST.md](./releases/2026-07-21-FOUR-CRATES-MAINTAINER-CHECKLIST.md) | Maintainer 收尾清单 | tag `v0.3.0-four-crates` 已推；可选手签仍仅人工 |
| [releases/2026-07-21-signoff-DRAFT.md](./releases/2026-07-21-signoff-DRAFT.md) | **SUPERSEDED** | 草稿归档；权威见 `0.3.0-signoff.md` |
| [releases/2026-07-21-MAINTAINER-HANDOFF.md](./releases/2026-07-21-MAINTAINER-HANDOFF.md) | 交接一页纸（历史） | 签核前步骤说明；签核后以正式包为准 |

## 冻结产物（W0）

| 文件 | 说明 |
|------|------|
| [artifacts/prod-trait-inventory.md](./artifacts/prod-trait-inventory.md) | 首批 / 二期生产 trait |
| [artifacts/wire-promotion-candidates.md](./artifacts/wire-promotion-candidates.md) | committed wire 升格批次 |
| [artifacts/support-matrix.md](./artifacts/support-matrix.md) | OS/MSRV（Accept 仅 Linux） |
| [artifacts/defer-disposition.md](./artifacts/defer-disposition.md) | DEFER-1…8 分类 |

## 发布签核

| 路径 | 用途 |
|------|------|
| [`releases/`](releases/) | 按版本填写的生产签核包（从模板复制） |
| 治理模板 | [`../governance/prod-signoff-TEMPLATE.md`](../../.agents/rules/prod-signoff-TEMPLATE.md) |

**规则**：签核文件中的 `Signed-off-by` **仅 Maintainer**；Agent 不得代签。正式权威：`releases/<version>-signoff.md`（非 `*DRAFT*`）。

## 约定

- 计划 ID 形如 `PLAN-<AREA>-<NNN>`
- 完成后在计划文内更新状态与变更记录；不删除历史波次
- 实现必须在 `.worktrees/<branch>` 内进行；本目录变更同样走 PR
