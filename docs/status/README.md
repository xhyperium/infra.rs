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
| [../../STATUS.md](../../STATUS.md) | **自动生成** crates 子模块进度/完成度看板（`node scripts/docs/gen-crate-status.mjs`） |

## 自动生成

```bash
# CI 工作流矩阵
node scripts/docs/gen-docs-status.mjs          # 从 .github/workflows 重写矩阵
node scripts/docs/gen-docs-status.mjs --check  # 校验已提交矩阵是否过期

# crates 进度看板
node scripts/docs/gen-crate-status.mjs              # 始终写本地副本；非 main 写入库 STATUS.md
node scripts/docs/gen-crate-status.mjs --local-only # 只写本地副本（主仓推荐）
node scripts/docs/gen-crate-status.mjs --tracked    # 强制写入库 STATUS.md
node scripts/docs/gen-crate-status.mjs --check      # CI：校验入库 STATUS.md
node scripts/docs/gen-crate-status.mjs --watch 30   # 自动监控
```

### 双写策略（减轻 worktree 摩擦）

| 产物 | 路径 | 何时写 | 是否入库 |
|------|------|--------|----------|
| **本地实时副本** | `docs/status/CRATES_STATUS.local.md` | 每次 `make status` | **否**（gitignore） |
| **入库看板** | 根目录 `STATUS.md` | 非 `main` 默认写；`main` 需 `--tracked` | **是** |

**不必**「同步一次就单独开 PR」。日常在主仓 `make status` 只刷本地副本；  
改了 `Cargo.toml` members / 标准布局时，在 **同一 feature worktree PR** 里顺带 `make status` 更新 `STATUS.md` 即可。

**SessionStart**：`.claude/hooks/session-context.mjs` 会自动 `gen-crate-status --local-only` 刷新本地副本，并打印一行平均完成度；**不**写入库 `STATUS.md`、**不**阻断会话。

`STATUS.md` 完成度是**结构进度**（布局七项 · 测试 · 源码实质），**不是** Production Ready 签字。

上级索引：[docs/README.md](../README.md)。
