# status/ — 状态与验证记录

## 职责

存放**会过期**的工程状态快照：CI 矩阵、配置检查、一次性验证记录。

## 收录标准

**应放入本目录：**

- CI 工作流矩阵与运行/迁移验证报告
- 仓库配置与分支保护检查总览
- 阶段性环境验证笔记

**不应放入本目录：**

- 长期有效的规则与策略 → `docs/governance/`
- SSOT 对齐矩阵 → `docs/ssot/`
- 架构决策 → `docs/decisions/`

## 文档

| 文档 | 说明 |
|------|------|
| [CI_WORKFLOW_MATRIX.generated.md](CI_WORKFLOW_MATRIX.generated.md) | **自动生成**工作流矩阵（`node scripts/docs/gen-docs-status.mjs`） |
| [CI_STATUS_REPORT.md](CI_STATUS_REPORT.md) | 人工叙事：CI 工作流说明与迁移验证 |
| [CONFIG_SUMMARY.md](CONFIG_SUMMARY.md) | 人工叙事：CI 配置、分支保护、测试验证总览 |

## 自动生成

```bash
node scripts/docs/gen-docs-status.mjs          # 从 .github/workflows 重写矩阵
node scripts/docs/gen-docs-status.mjs --check  # 校验已提交矩阵是否过期
```

工作流 YAML 变更后应重跑生成脚本，并提交 `CI_WORKFLOW_MATRIX.generated.md`。

上级索引：[docs/README.md](../README.md)。
