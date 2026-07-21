# constitution/ — 工程宪章

## 职责

本目录是 **工程宪章** 在 `docs/` 下的归类入口：索引宪章正文、条款导航，以及仅服务宪章阅读与修订的附属材料。

**宪章正文 SSOT** 仍位于仓库根：[`CONSTITUTION.md`](../../CONSTITUTION.md)。  
本目录不复制正文；修订宪章时改根文件，并在本目录同步索引（如有条款表变更）。

## 收录标准

**应放入本目录：**

- 宪章条款导航 / 章节索引
- 宪章阅读指南、修订说明、与治理文档的映射表
- 仅解释宪章本身（不展开落地细则）的材料

**不应放入本目录：**

- 宪章条款的**实施细则**（worktree、语言约定、STE、版本号规则等）→ [`docs/governance/`](../governance/)
- SSOT 对齐矩阵 / 同步报告 → [`docs/ssot/`](../ssot/)
- CI 状态、配置快照 → [`docs/status/`](../status/)
- 架构决策（DDR）→ [`docs/decisions/`](../decisions/)

## 宪章正文

| 文档 | 说明 |
|------|------|
| [CONSTITUTION.md](../../CONSTITUTION.md) | 工程宪章正文（仓库根 SSOT） |

## 条款导航

| 章节 | 主题 |
|------|------|
| 一 | 使命 |
| 二 | 核心价值观（安全 / 可观测 / 可验证 / 自动化 / 简单） |
| 三 | 架构原则（模块边界 / 接口 / 类型驱动 / 错误处理） |
| 四 | 代码标准（格式 / Lint / 命名 / 测试 / 语言编码 / STE / ESM） |
| 五 | 质量门禁 |
| 六 | 治理（Git Main First / 变更流程 / 版本 / 所有权） |
| 七 | AI 代理章程 |
| 八 | 修订 |

## 落地细则（不在本目录）

宪章条款的可执行细则见 [`docs/governance/`](../governance/)：

| 细则 | 宪章锚点 |
|------|----------|
| [VERSIONING.md](../governance/VERSIONING.md) | §6.2 / 版本策略 |
| [worktree-policy.md](../governance/worktree-policy.md) | §6.0.5 |
| [编码与语言约定.md](../governance/编码与语言约定.md) | §4.5 |
| [ASD-STE100.md](../governance/ASD-STE100.md) | §4.6 |
| [quant-dev-spec.md](../governance/quant-dev-spec.md) | 领域扩展 |

上级索引：[docs/README.md](../README.md)。
