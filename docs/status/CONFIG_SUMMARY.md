# CONFIG_SUMMARY.md — 配置与测试记录

> **更新日期**: 2026-07-24  
> **版本**: v2.0  
> **仓库**: [xhyperium/infra.rs](https://github.com/xhyperium/infra.rs)  
> **权威源**: 以 GitHub Ruleset 为准；本文件仅记录配置基线快照。

---

## CI 工作流概览

> **workflow 清单不再在此手工维护**——以 `.github/workflows/` 目录实际文件为准。  
> 运行 `ls .github/workflows/*.yml` 获取当前完整列表。

### 按类别

| 类别 | 说明 | required? |
|------|------|-----------|
| **核心门禁** | constitution / quality / ci-rust-org / security / validation / pr-template-check | Ruleset required: **Constitution Check** + **Template Validation**；组织 ruleset 另强制 rust-fmt/clippy/test |
| **per-crate coverage** | kernel / testkit / configx / observex / resiliencx / schedulex / contracts / canonical / decimal / evidence | 信号（非阻断） |
| **MIRI / Loom / Mutants** | kernel / testkit / decimal | 定时 + PR paths 触发（非阻断） |
| **Live 集成** | redisx-live / contracts-live / exchange-live-readonly | paths/manual（非阻断） |
| **治理辅助** | contract-compliance / public-api / secrets-lint / self-test / ssot-dual-specs / workflow-security / ci-summary | 信号（部分 required by org） |
| **发布/协作** | release / rebase-on-push / rebase-on-label / beads-e2e / beads-test | event 触发 |

### required checks（合入硬门槛）

Ruleset `main-ai-first` required（名称须完全一致）：

1. **`Constitution Check`**（`constitution.yml` job name）
2. **`Template Validation`**（`pr-template-check.yml` job name）

组织级 ruleset 另外强制（来自 `xhyperium/.github` 可复用 workflow `ci-rust-org.yml`）：`rust-fmt` / `rust-clippy` / `rust-test`。

其余 workflow 多为 **信号 / 推荐**；应尽量绿，但**以仓库 Ruleset 配置为准**。

---

## 分支保护规则（Ruleset）

> **2026-07-21 迁移**：经典 Branch Protection 已删除；`main` 由仓库 Ruleset **`main-ai-first`**（id `19250230`）强制。  
> UI：<https://github.com/xhyperium/infra.rs/rules/19250230>

| 规则 | 值 |
| ------ | ----- |
| 机制 | GitHub Ruleset（非 classic branch protection） |
| Target | `refs/heads/main` |
| Enforcement | `active` |
| 合并前需 PR | 启用 |
| 最少 approving reviews | 1 |
| CODEOWNERS 审查 | 强制 |
| 过时 PR 自动 dismiss | 启用 |
| `require_last_push_approval` | **启用**（最后 pusher 不能当唯一批准；走 `pr-auto-approve` / 第二身份） |
| Conversation resolution | 未强制（AI First：避免评论挂死） |
| Required status checks | **`Constitution Check`**、**`Template Validation`**（须与 check_run 名一致，勿加 workflow 前缀） |
| Status check strict | 启用（分支须与 base 同步） |
| 线性历史 | 启用 |
| Force push | 禁止（`non_fast_forward`） |
| 删除 `main` | 禁止（`deletion`） |
| 允许的 merge 方法 | **仅 squash**（ruleset + 仓库设置） |
| CODEOWNERS | `@ZoneCNH @liukongqiang5`（team 需 write 后可改回 `@xhyperium/maintainers`） |
| Bypass | team `maintainers`（`pull_request` 模式，应急） |
| 合并后删除分支 | 启用（仓库设置） |
| Auto-merge | 启用（仓库设置） |

### AI First 合入路径

```bash
# 1) 开 PR（作者 ZoneCNH）
# 2) 等 required: Constitution Check + Template Validation 绿
# 3) 第二身份批准（默认自动识别当前仓库，无需 export PR_AUTO_APPROVE_REPO）
bash .claude/skills/pr-auto-approve/scripts/approve.sh <pr> "checks green; AI-first path."
# 4) 自动合入
gh pr merge <pr> --squash --auto
```

---

## 直推 main 拦截验证

- **测试时间**: 2026-07-21（Ruleset 迁移后复测）
- **测试方法**: 空 commit 直推 `origin main`
- **结果**: 推送被拒绝（Ruleset only；classic 已删除）

```text
remote: error: GH013: Repository rule violations found for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - 2 of 2 required status checks are expected.
 ! [remote rejected] HEAD -> main (push declined due to repository rule violations)
```

- **应急**: team `maintainers` 可在 PR 路径下 bypass；须在 PR 记录原因
- **来源**: `docs/constitution/06-governance.md` §6.0 + Ruleset `main-ai-first`

---

## 仓库配置

| 配置 | 值 |
| ------ | ----- |
| 默认分支 | `main` |
| 合并方式 | squash merge only |
| Auto-merge | 启用 |
| 合并后删除分支 | 启用 |
| Secret scanning | 启用 + push protection |
| Dependabot security updates | 禁用（依赖 `security.yml` 周一 audit） |

---

## 变更日志

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-07-24 | v2.0 | 不再手工维护 workflow 清单（指向 `.github/workflows/` 实际文件）；修正 `ci-rust.yml` → `ci-rust-org.yml`；添加 workflow 分类概览 |
| 2026-07-21 | v1.1 | 初始版本 |
