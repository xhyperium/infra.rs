# docs/ — 文档索引

本目录按**文档职责**严格分类。新增文档必须先归类，禁止在 `docs/` 根目录平铺内容文件（索引与 `.gitkeep` 除外）。

## 分类规则

| 目录 | 放什么 | 不放什么 |
|------|--------|----------|
| [`constitution/`](constitution/) | 工程宪章正文 SSOT（分章）+ 索引 | 落地细则、对齐矩阵、状态报告、DDR |
| [`governance/`](governance/) | 宪章落地细则、强制工程约定、领域规范 | 对齐审计矩阵、一次性状态报告、DDR |
| [`ssot/`](ssot/) | SSOT 镜像同步手册、域对齐矩阵、落地状态 | 通用治理条文、CI 配置快照 |
| [`plans/`](plans/) | 跨 crate 可执行修复/晋级计划（波次、验收、门禁） | 一次性审计报告、长期治理条文、对齐矩阵 |
| [`status/`](status/) | CI/仓库配置的**状态与验证记录**（可过期） | 长期有效的规则与策略 |
| [`decisions/`](decisions/) | 架构决策记录（DDR） | 日常状态报告、对齐矩阵 |
| [`api-baselines/`](api-baselines/) | 核心 crate 公开 API 文本快照（semver 门禁） | 叙事文档、一次性报告 |
| [`plans/`](plans/) | 执行计划副本、发布签核包 | 宪章正文、SSOT 对齐矩阵 |
| [`report/`](report/) | 只读审计/就绪度报告（可过期） | 可执行计划（→ `plans/`） |

**根目录只允许**：本 `README.md`、分类子目录、`.gitkeep`。

**与 crate 文档分层**（见 [`crates/AGENTS.md`](../crates/AGENTS.md)）：

| 层级 | 职责 |
|------|------|
| 仓库根 `docs/` | 跨 crate 治理、SSOT 对齐、工作流、DDR |
| `crates/<name>/docs/` | 单 crate 设计、API 契约、迁移 |

---

## 工程宪章 — [`constitution/`](constitution/)

| 文档 | 说明 |
|------|------|
| [README.md](constitution/README.md) | 宪章索引与常用条款速查（正文 SSOT 入口） |
| [01-mission.md](constitution/01-mission.md) … [08-amendments.md](constitution/08-amendments.md) | 分章正文 |
| [CONSTITUTION.md](../CONSTITUTION.md) | 仓库根兼容索引（指向本目录） |

## 治理与约定 — [`governance/`](governance/)

| 文档 | 说明 |
|------|------|
| [VERSIONING.md](governance/VERSIONING.md) | 统一版本管理（项目 / 宪章 / Crate） |
| [support-matrix.md](governance/support-matrix.md) | 官方支持矩阵（Linux x86_64 + MSRV 1.85；DEFER-6） |
| [prod-signoff-TEMPLATE.md](governance/prod-signoff-TEMPLATE.md) | 生产签核包模板（L1–L5；仅 Maintainer 签核） |
| [worktree-policy.md](governance/worktree-policy.md) | Git Worktree 强制开发策略（宪章 §6.0.5） |
| [编码与语言约定.md](governance/编码与语言约定.md) | UTF-8 与中文/英文文档语言约定 |
| [ASD-STE100.md](governance/ASD-STE100.md) | 英文技术文档受控语言（STE）落地指南 |
| [quant-dev-spec.md](governance/quant-dev-spec.md) | 量化开发领域专项规范 |

相关根文档：[CONSTITUTION.md](../CONSTITUTION.md)（兼容索引 → [`constitution/`](constitution/)）、[ARCHITECTURE.md](../ARCHITECTURE.md)。

公开 API baseline：[`api-baselines/`](api-baselines/)（DEFER-5；`scripts/quality-gates/check-public-api.mjs`）。

---

## SSOT 对齐与同步 — [`ssot/`](ssot/)

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](ssot/workspace-ssot-alignment.md) | Workspace members 与 SSOT 域落地**总览** |
| [SSOT_SYNC_OPS.md](ssot/SSOT_SYNC_OPS.md) | 上游镜像同步操作手册 |
| [SSOT_SYNC_REPORT.md](ssot/SSOT_SYNC_REPORT.md) | 镜像同步完整性报告（≠ 实现落地） |
| [kernel-ssot-alignment.md](ssot/kernel-ssot-alignment.md) | kernel |
| [testkit-ssot-alignment.md](ssot/testkit-ssot-alignment.md) | testkit |
| [configx-ssot-alignment.md](ssot/configx-ssot-alignment.md) | configx |
| [schedulex-ssot-alignment.md](ssot/schedulex-ssot-alignment.md) | schedulex |
| [types-ssot-alignment.md](ssot/types-ssot-alignment.md) | types（decimal + canonical） |
| [bootstrap-ssot-alignment.md](ssot/bootstrap-ssot-alignment.md) | bootstrap |
| [adapters-ssot-alignment.md](ssot/adapters-ssot-alignment.md) | adapters 九域 |
| [contracts-ssot-alignment.md](ssot/contracts-ssot-alignment.md) | contracts |
| [observex-ssot-alignment.md](ssot/observex-ssot-alignment.md) | observex |
| [resiliencx-ssot-alignment.md](ssot/resiliencx-ssot-alignment.md) | resiliencx |
| [transport-ssot-alignment.md](ssot/transport-ssot-alignment.md) | transport |
| [evidence-ssot-alignment.md](ssot/evidence-ssot-alignment.md) | evidence |
| [tools-ssot-alignment.md](ssot/tools-ssot-alignment.md) | tools（evidence / goalctl / xtask / verifyctl） |

---

## 执行计划 — [`plans/`](plans/)

| 文档 | 说明 |
|------|------|
| [README.md](plans/README.md) | 计划目录约定与索引 |
| [2026-07-21-core-crates-production-readiness.md](plans/2026-07-21-core-crates-production-readiness.md) | 五核心 crate 生产级修复方案（W0–W5 · DONE） |
| [2026-07-21-status-modules-prod-followup.md](plans/2026-07-21-status-modules-prod-followup.md) | STATUS 全模块生产就绪 follow-up（Beads `infra-s9t`） |

> 计划是**可执行**的；审计结论见 [`report/`](report/)。

## 审计报告 — [`report/`](report/)

| 文档 | 说明 |
|------|------|
| [2026-07-21/status-modules-production-readiness.md](report/2026-07-21/status-modules-production-readiness.md) | STATUS 全 21 模块生产级就绪度（Agent Team） |
| [2026-07-21/core-crates-production-readiness.md](report/2026-07-21/core-crates-production-readiness.md) | 五核心 crate 生产就绪审计 |
| [2026-07-21/README.md](report/2026-07-21/README.md) | 当日报告索引 |

## 状态与验证记录 — [`status/`](status/)

| 文档 | 说明 |
|------|------|
| [CI_WORKFLOW_MATRIX.generated.md](status/CI_WORKFLOW_MATRIX.generated.md) | 自动生成：工作流文件 / jobs 矩阵 |
| [CI_STATUS_REPORT.md](status/CI_STATUS_REPORT.md) | 人工叙事：CI 说明与迁移验证 |
| [CONFIG_SUMMARY.md](status/CONFIG_SUMMARY.md) | 人工叙事：配置、分支保护、测试验证总览 |
| [../STATUS.md](../STATUS.md) | **自动生成**：crates 子模块进度/完成度看板（`make status`） |

> 状态类文档会随时间过期；规则类文档请放在 `governance/`。  
> crates 进度看板在仓库根 `STATUS.md`（由 `scripts/docs/gen-crate-status.mjs` 生成，支持 `--watch` 自动监控）。

---

## 架构决策记录 — [`decisions/`](decisions/)

目录：[decisions/](decisions/) — 格式与模板见 [decisions/README.md](decisions/README.md)（DDR-001 ~ DDR-009）。
