# governance/ — 治理与约定

## 职责

存放**长期有效**的工程约定与宪章落地细则。内容应可被 Agent、CI 与人类共同引用。

## 收录标准

**应放入本目录：**

- 宪章条款的实施细则（语言、版本、worktree 等）
- 跨 crate 的强制规范
- 领域专项规范（如量化）

**不应放入本目录：**

- 工程宪章正文 / 条款导航 → `docs/constitution/`（根 `CONSTITUTION.md` 仅为兼容索引）
- SSOT 对齐矩阵 / 同步报告 → `docs/ssot/`
- CI 运行快照、配置检查记录 → `docs/status/`
- 架构决策（DDR）→ `docs/decisions/`
- 单 crate API/设计 → `crates/<name>/docs/`

## 文档

| 文档 | 宪章锚点 | 说明 |
|------|----------|------|
| [VERSIONING.md](VERSIONING.md) | 版本策略 | 项目 / 宪章 / Crate 版本规则 |
| [support-matrix.md](support-matrix.md) | 发布 / CI | 官方支持矩阵（Linux x86_64 + MSRV 1.85） |
| [prod-signoff-TEMPLATE.md](prod-signoff-TEMPLATE.md) | 发布签核 | L1–L5 模板；**仅 Maintainer 签核** |
| [worktree-policy.md](worktree-policy.md) | §6.0.5 | Git Worktree 强制策略 |
| [编码与语言约定.md](编码与语言约定.md) | §4.5 | UTF-8、中文注释、文档语言 |
| [ASD-STE100.md](ASD-STE100.md) | §4.6 | 英文技术文档 STE 落地指南 |
| [quant-dev-spec.md](quant-dev-spec.md) | 领域扩展 | 量化金融专项要求 |

上级索引：[docs/README.md](../README.md)。  
API baseline：[`../api-baselines/`](../api-baselines/)。  
发布签核副本目录：[`../plans/releases/`](../plans/releases/)。
