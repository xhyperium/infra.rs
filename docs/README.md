# docs/ — 文档索引

本目录按**文档职责**严格分类。新增文档必须先归类，禁止在 `docs/` 根目录平铺内容文件（索引与 `.gitkeep` 除外）。

## 分类规则

| 目录 | 放什么 | 不放什么 |
|------|--------|----------|
| [`governance/`](governance/) | 宪章落地细则、强制工程约定、领域规范 | 对齐审计矩阵、一次性状态报告、DDR |
| [`ssot/`](ssot/) | SSOT 镜像同步手册、域对齐矩阵、落地状态 | 通用治理条文、CI 配置快照 |
| [`status/`](status/) | CI/仓库配置的**状态与验证记录**（可过期） | 长期有效的规则与策略 |
| [`decisions/`](decisions/) | 架构决策记录（DDR） | 日常状态报告、对齐矩阵 |

**根目录只允许**：本 `README.md`、分类子目录、`.gitkeep`。

**与 crate 文档分层**（见 [`crates/AGENTS.md`](../crates/AGENTS.md)）：

| 层级 | 职责 |
|------|------|
| 仓库根 `docs/` | 跨 crate 治理、SSOT 对齐、工作流、DDR |
| `crates/<name>/docs/` | 单 crate 设计、API 契约、迁移 |

---

## 治理与约定 — [`governance/`](governance/)

| 文档 | 说明 |
|------|------|
| [VERSIONING.md](governance/VERSIONING.md) | 统一版本管理（项目 / 宪章 / Crate） |
| [worktree-policy.md](governance/worktree-policy.md) | Git Worktree 强制开发策略（宪章 §6.0.5） |
| [编码与语言约定.md](governance/编码与语言约定.md) | UTF-8 与中文/英文文档语言约定 |
| [ASD-STE100.md](governance/ASD-STE100.md) | 英文技术文档受控语言（STE）落地指南 |
| [quant-dev-spec.md](governance/quant-dev-spec.md) | 量化开发领域专项规范 |

相关根文档（不在 `docs/` 内）：[CONSTITUTION.md](../CONSTITUTION.md)、[ARCHITECTURE.md](../ARCHITECTURE.md)。

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

## 状态与验证记录 — [`status/`](status/)

| 文档 | 说明 |
|------|------|
| [CI_WORKFLOW_MATRIX.generated.md](status/CI_WORKFLOW_MATRIX.generated.md) | 自动生成：工作流文件 / jobs 矩阵 |
| [CI_STATUS_REPORT.md](status/CI_STATUS_REPORT.md) | 人工叙事：CI 说明与迁移验证 |
| [CONFIG_SUMMARY.md](status/CONFIG_SUMMARY.md) | 人工叙事：配置、分支保护、测试验证总览 |

> 状态类文档会随时间过期；规则类文档请放在 `governance/`。

---

## 架构决策记录 — [`decisions/`](decisions/)

目录：[decisions/](decisions/) — 格式与模板见 [decisions/README.md](decisions/README.md)（DDR-001 ~ DDR-009）。
