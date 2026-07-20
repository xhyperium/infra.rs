# CONFIG_SUMMARY.md — 配置与测试记录

> 生成日期: 2026-07-21  
> 版本: v1.1  
> 仓库: [xhyperium/infra.rs](https://github.com/xhyperium/infra.rs)

---

## CI 工作流 (6 个)

| 工作流 | 文件 | 触发 | 状态 |
|--------|------|------|------|
| Validation | `validation.yml` | PR + main push | YAML/TOML/MD lint + spell + link check |
| Quality | `quality.yml` | PR + Rust 文件变更 | rustfmt + clippy |
| CI (Rust) | `ci-rust.yml` | PR + Rust 文件变更 | build → test → coverage |
| Security | `security.yml` | PR + 定时 (周一) | cargo-deny + cargo-audit |
| Constitution | `constitution.yml` | PR + main push | 宪章合规性验证 |
| PR Template Check | `pr-template-check.yml` | PR (opened/edited/sync) | 模板字段校验 |

---

## 分支保护规则

| 规则 | 值 |
|------|-----|
| 合并前需 PR | 启用 |
| 最少 approving reviews | 1 |
| CODEOWNERS 审查 | 强制 |
| 过时 PR 自动 dismiss | 启用 |
| Required status checks | Constitution Check, PR Template Check |
| Status check strict | 启用 (分支须与 main 同步) |
| 线性历史 | 启用 |
| Force push | 禁止 |
| Branch deletion | 禁止 |
| Squash merge only | 启用 |
| 合并后删除分支 | 启用 |
| Auto-merge | 启用 |

---

## enforce_admins 测试验证

- **测试时间**: 2026-07-21
- **测试方法**: 开启 `enforce_admins: true`，管理员直接在 main 提交并推送
- **结果**: 推送被拒绝

```
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - 2 of 2 required status checks are expected.
 ! [remote rejected] main -> main (protected branch hook declined)
```

- **生产策略**: `enforce_admins: false`，管理员可应急绕过，须在 PR 记录原因
- **来源**: `CONSTITUTION.md §6.0.5`

---

## Dependabot

| 项目 | 状态 |
|------|------|
| Dependabot 配置 | 已移除 (`dependabot.yml` 删除) |
| 最后漏洞告警 | CVE-2024-48908 (lychee-action v1 → v2，已修复) |
| 告警状态 | fixed (自动) |

---

## 宪章合规性验证

全部强制门禁 (8/8) 已通过：

```
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

## 仓库配���

| 配置 | 值 |
|------|-----|
| 默认分支 | `main` |
| 合并方式 | squash merge only |
| Auto-merge | 启用 |
| 合并后删除分支 | 启用 |
| Secret scanning | 启用 + push protection |
| Dependabot security updates | 禁用 |
