# ssot/ — SSOT 对齐与同步

## 职责

记录本仓对上游 SSOT 镜像（`.agents/ssot/**`）的**同步状态**与**实现落地矩阵**。

> 镜像写 COMPLETE / Stable **≠** 本仓已宣称可 ship。以 `crates/` + `cargo metadata` 为准。

## 收录标准

**应放入本目录：**

- 各域 `*-ssot-alignment.md` 对齐矩阵
- 工作区总览 `workspace-ssot-alignment.md`
- 同步操作手册与同步完整性报告

**不应放入本目录：**

- 通用治理条文 → `docs/governance/`
- CI/配置状态快照 → `docs/status/`
- 上游镜像正文 → `.agents/ssot/`（只读镜像，禁止手改）

## 阅读顺序

1. [workspace-ssot-alignment.md](workspace-ssot-alignment.md) — members 地图与总览
2. [SSOT_SYNC_OPS.md](SSOT_SYNC_OPS.md) — 如何从上游同步
3. [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) — 镜像是否完整（≠ 实现落地）
4. 各域 `*-ssot-alignment.md` — 本仓落地差距
5. 生产就绪审计（五核心 crate）：[../report/2026-07-21/core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md)

> **2026-07-21 跟进（PR #98）**：decimal/canonical/contracts/kernel/testkit 已做 P0/P1 闭合并回写对齐文；镜像 COMPLETE **仍不等于** 本仓 Production Ready。

## 文档

| 文档 | 说明 |
|------|------|
| [workspace-ssot-alignment.md](workspace-ssot-alignment.md) | Workspace 总览 |
| [SSOT_SYNC_OPS.md](SSOT_SYNC_OPS.md) | 同步操作手册 |
| [SSOT_SYNC_REPORT.md](SSOT_SYNC_REPORT.md) | 同步完整性报告 |
| [kernel-ssot-alignment.md](kernel-ssot-alignment.md) | kernel |
| [testkit-ssot-alignment.md](testkit-ssot-alignment.md) | testkit |
| [configx-ssot-alignment.md](configx-ssot-alignment.md) | configx |
| [schedulex-ssot-alignment.md](schedulex-ssot-alignment.md) | schedulex |
| [types-ssot-alignment.md](types-ssot-alignment.md) | types |
| [bootstrap-ssot-alignment.md](bootstrap-ssot-alignment.md) | bootstrap |
| [adapters-ssot-alignment.md](adapters-ssot-alignment.md) | adapters |
| [contracts-ssot-alignment.md](contracts-ssot-alignment.md) | contracts |
| [observex-ssot-alignment.md](observex-ssot-alignment.md) | observex |
| [resiliencx-ssot-alignment.md](resiliencx-ssot-alignment.md) | resiliencx |
| [transport-ssot-alignment.md](transport-ssot-alignment.md) | transport |
| [evidence-ssot-alignment.md](evidence-ssot-alignment.md) | evidence |
| [tools-ssot-alignment.md](tools-ssot-alignment.md) | tools（evidence/goalctl/xtask/verifyctl） |

上级索引：[docs/README.md](../README.md)。
