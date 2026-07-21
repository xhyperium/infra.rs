# CONFIG_SUMMARY.md — 配置与测试记录

> 生成日期: 2026-07-21  
> 版本: v1.1  
> 仓库: [xhyperium/infra.rs](https://github.com/xhyperium/infra.rs)

---

## CI 工作流 (6 个)

| 工作流 | 文件 | 触发 | 状态 |
| -------- | ------ | ------ | ------ |
| Validation | `validation.yml` | PR + main push | YAML/TOML/MD lint + spell + link check |
| Quality | `quality.yml` | PR + Rust 文件变更 | rustfmt + clippy |
| CI (Rust) | `ci-rust.yml` | PR + Rust 文件变更 | build → test → coverage |
| Security | `security.yml` | PR + 定时 (周一) | cargo-deny + cargo-audit |
| Constitution | `constitution.yml` | 全部 PR；main push（Rust/配置路径） | 宪章合规；docs-only PR 快速绿 |
| PR Template Check | `pr-template-check.yml` | PR (opened/edited/sync) | 模板字段校验 |

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

## Dependabot

| 项目 | 状态 |
| ------ | ------ |
| Dependabot 配置 | 已移除 (`dependabot.yml` 删除) |
| 最后漏洞告警 | CVE-2024-48908 (lychee-action v1 → v2，已修复) |
| 告警状态 | fixed (自动) |

---

## 宪章合规性验证

全部强制门禁 (8/8) 已通过：

```text
rustfmt          ✓
clippy           ✓
unit + doc tests ✓  (18 tests)
unsafe 审计       ✓  (0 处)
unwrap/expect    ✓  (clippy lint 控制)
命名规范         ✓  (snake_case)
文档             ✓  (cargo doc + doc-test)
cargo-deny       ✓
```

快捷命令: `make check` / `make check-quick` / `make check-json`

---

## 仓库配置

| 配置 | 值 |
| ------ | ----- |
| 默认分支 | `main` |
| 合并方式 | squash merge only |
| Auto-merge | 启用 |
| 合并后删除分支 | 启用 |
| Secret scanning | 启用 + push protection |
| Dependabot security updates | 禁用 |
