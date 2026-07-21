# CLAUDE.md — Claude Code Agent Instructions

本仓库 `infra.rs` 是**独立的 Rust 基础设施工作区**。Claude Code 担任主执行代理。

详细治理见 [AGENTS.md](./AGENTS.md)。

## 项目身份

- 这是 **Rust 源码仓库**（Cargo workspace），不是外部项目的配置镜像
- 主要代码在 `crates/`
- 职责：实现库代码、维护 CI/工程约定、维护 Agent skills/hooks

## 上游 SSOT 与本仓落地

- `.agents/ssot/{kernel,testkit,types,infra,adapters}` 是上游只读镜像；**不要**把镜像 COMPLETE 当成「本仓可宣称 ship」
  - infra 平面在 `.agents/ssot/infra/{bootstrap,configx,gate,…}`（保留 `infra/` 层级）
  - adapters 平面在 `.agents/ssot/adapters/{exchange,storage}/…`（保留 `adapters/` 层级）
- 本仓 members：`kernel` / `testkit` / `configx` / `bootstrap` / `resiliencx` / `types/*` / `contracts` / `adapters/**`（adapters 为 scaffold；**无** `infra-core`；infra 其余域未落地）
- 验证：`cargo test --workspace`；专项见对齐文档
- 总览：[docs/workspace-ssot-alignment.md](./docs/workspace-ssot-alignment.md)
- 域文档：
  - [docs/kernel-ssot-alignment.md](./docs/kernel-ssot-alignment.md)
  - [docs/testkit-ssot-alignment.md](./docs/testkit-ssot-alignment.md)
  - [docs/types-ssot-alignment.md](./docs/types-ssot-alignment.md)
  - [docs/bootstrap-ssot-alignment.md](./docs/bootstrap-ssot-alignment.md)
  - [docs/adapters-ssot-alignment.md](./docs/adapters-ssot-alignment.md)

## Hook 感知

`.claude/settings.json` 已注册完整钩子链：

1. **启动**：`session-context.mjs` + `bd prime`
2. **工具前**：`count-guard` / `edit-guard` / `pre-tool-check`（含 **worktree 硬门禁**）
3. **编辑后**：`post-tool-check` / `link-check` / `edit-guard-reset`
4. **结束**：`session-review` / `version-guard` / `branch-protect`

钩子报错时应修复问题，不要绕过。

## Git Worktree（强制开工）

**禁止在主仓 `main` 工作区直接改代码。** 细则：[docs/worktree-policy.md](./docs/worktree-policy.md)

```bash
node scripts/worktree.mjs create feat/<id>-<slug>
cd .worktrees/feat/<id>-<slug>
# 在该目录内编码 / 测试 / 提交 / 开 PR
```

- `pre-tool-check` **BLOCK**：主仓**已跟踪**文件 Write/Edit、主仓 `checkout -b`/`switch` 功能分支、`main` 上 commit
- **例外**：`.gitignore` 匹配路径（如 `.beads/`、`.claude/*.local.json`、`target/`）可在主仓编辑；`.env` 仍单独拦截
- 路径规范：`.worktrees/<branch-name>`（`/` 保留为目录分隔符）
- 禁止设置 `INFRA_WORKTREE_BYPASS=1` 绕过（仅人工 maintainer 应急）

## 安全红线

- 不提交 `.claude/*.local.json`、密钥、证书
- 不在对话中回显完整 API Token
- 不削弱 `.gitignore` 对敏感路径的排除
- 不执行 `git push --force`、`git push --no-verify`、删除 `main`（除非用户明确要求且已确认风险）
- **Git Main First**（宪章 §6.0）：不在 `main` 上直接开发；worktree 建支 → PR → 合并 main

## 代码行为准则

- **中文优先**：代码注释、中文治理文档、用户可见错误信息使用中文；标识符保持英文
- **ASD-STE100**：英文技术文档使用简化技术英语（宪章 §4.6；见 `docs/ASD-STE100.md`）
- **UTF-8**：所有文本文件 UTF-8 无 BOM，换行 LF（宪章 §4.5；见 `docs/编码与语言约定.md` 与 `.editorconfig`）
- **宪章约束**：语言/编码/文档标准见 `CONSTITUTION.md` §4.5、§4.6，不可削弱

- **最小变更**：只做明确要求的事
- **保守编辑**：优先小步修改现有文件
- **SSOT**：技能以 `.claude/skills/` 为准
- **Rust 质量门禁**：提交前 `fmt` + `clippy -D warnings` + `test`
- 禁止无上下文的 `unwrap()`；库代码优先 `thiserror`，应用侧可用 `anyhow`
- 日志用 `tracing`，不用 `println!`（示例/bin 除外）

## Karpathy 原则

- Think Before Coding
- 消除信息差
- 讨论与执行分离
- Simplicity First
- Surgical Changes
- Goal-Driven Execution

## 自动审查闭环

```bash
node scripts/check.mjs
node scripts/gc-scan.mjs
cargo test --workspace
bd ready
```

## Skill 路由

| 任务类型 | Skill | 触发条件 |
| --------- | ------- | --------- |
| Harness 初始化 / 复位 | harness-init / harness-start | 初始化、检查 Harness |
| 模式切换 | harness-mode | full / hotfix / tweak |
| 健康扫描 | harness-gc | 漂移与一致性 |
| 代码审查 | code-review | PR / diff |
| 接口设计 | codebase-design / design-an-interface | 模块边界、API |
| Bug 诊断 | diagnosing-bugs | 失败、回归 |
| 领域建模 | domain-modeling | 术语、ADR |
| TDD | tdd | 测试先行 |
| Beads | beads | 任务认领与关闭 |
| Git 护栏 | git-guardrails-claude-code | 危险 git 操作防护 |

## 与 Codex 协作

- Codex：编排派工
- Claude：实现与交付
- 冲突时以本仓库 `AGENTS.md` / `CLAUDE.md` 为准

## 本地 Provider

`.claude/*.local.json` 含 API 配置（gitignored），由 Claude Code `--settings` 切换，会话内无需管理。

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:6cd5cc61 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See [SYNC_CONCEPTS.md](https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md) for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:

   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```

5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**

- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->
