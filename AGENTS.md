# AGENTS.md — AI Agent Governance for infra.rs

本仓库是**独立的 Rust 基础设施工作区**。所有 AI 编码助手（Claude Code、Codex、Copilot 等）共享本文件中的治理约定。

## Rust 编码规范（强制）

- **上位全局标准**：《[Rust 编码规范（完整版）v2.0](https://github.com/bytechainx/.github/blob/main/rulesets/rust/RULES.md)》——组织 SSOT：`bytechainx/.github` → `rulesets/rust/`
- **Agent 加载**：`~/.claude/rules/rust.md`（symlink）；专项见同目录 `security` / `async-runtime` / `api-design` 等
- **本仓关系**：宪章 [§4.0](./docs/constitution/04-code-standards.md#40-rust-全局编码规范强制上位) 采纳上位标准；项目细则可**加严**、**不可削弱** P0
- 提交前：`cargo fmt` + `clippy -D warnings` + `test`（与完整版 / §5 门禁一致）

## 语言与编码（强制）

- **字符编码**：全部文本文件使用 **UTF-8（无 BOM）**，换行 **LF**
- **注释 / 中文治理文档**：统一使用**中文**（技术术语可保留英文）
- **英文技术文档**：**ASD-STE100（Simplified Technical English）**（宪章 §4.6）
- **用户可见错误信息**：中文
- **标识符**：英文（Rust 惯例）
- **LICENSE**：保留英文许可证原文
- 细则见 [docs/constitution/04-code-standards.md §4.0 / §4.5 / §4.6](./docs/constitution/04-code-standards.md)、[docs/governance/编码与语言约定.md](./docs/governance/编码与语言约定.md)、[docs/governance/ASD-STE100.md](./docs/governance/ASD-STE100.md)；索引 [docs/constitution/](./docs/constitution/)

## 项目身份

- **类型**：Rust Cargo workspace
- **根 crate 示例**：`crates/kernel`（`xhyper-kernel`）
- **非目标**：不是其他产品的元仓库镜像；本地即为源码与约定的 SSOT

## 上游 SSOT 镜像与本仓落地

- `.agents/ssot/{kernel,testkit,types,infra,adapters,contracts,tools}/` 是 `xhyper.rs/.agent/SSOT/` 的**只读镜像**（见 `.agents/ssot/SSOT.md` R6）
  - `infra/` 下含 bootstrap / configx / gate / observex / resiliencx / schedulex / testkitx / transport
  - `adapters/` 下含 exchange（binance/okx）与 storage（clickhouse/kafka/nats/oss/postgres/redis/taos）
  - `tools/` 下含 evidence / goalctl / xtask（+ 本仓扩展 `verifyctl`）
  - **保留 `adapters/`、`tools/` 层级**（勿展平到 `.agents/ssot/` 根；`infra/` 已展平）
- **镜像文档写 COMPLETE / Stable ≠ 本仓已有对应实现**；必须以 `crates/` + `cargo metadata` 为准
- **当前 workspace members**（无 `infra-core`）：
  - `crates/kernel` → `xhyper-kernel`（L0）
  - `crates/testkit` → `xhyper-testkit`（core ManualClock；仅 dev-dep）
  - `crates/configx` → `xhyper-configx`（L1 内存字符串 KV；非多源热更新）
  - `crates/schedulex` → `xhyper-schedulex`（L1 任务 ID 登记表；active SSOT registry only）
  - `crates/bootstrap` → `xhyper-bootstrap`（L1 组合根；已注入 contracts/observex/evidence）
  - `crates/evidence` → `xhyper-evidence`（L1 审计证据追加面）
  - `crates/observex` → `xhyper-observex`（L1 TracingInstrumentation 最小面）
  - `crates/resiliencx` → `xhyper-resiliencx`（L1 重试 + 熔断 + 限流）
  - `crates/transport` → `xhyper-transportx`（L1 HTTP/WS）
  - `crates/types/decimal` → `xhyper-decimalx`
  - `crates/types/canonical` → `xhyper-canonical`
  - `crates/contracts` → `xhyper-contracts`（adapter trait 出口；#43）
  - `crates/adapters/**` → 9 个 adapter package（**scaffold**；见 adapters 对齐文）
- `contract-testkit` **未**移植；**infra 其余域**（gate 等）当前仅镜像，未宣称本仓实现
- **adapters**：镜像已本地化；crate 为 scaffold，**未**宣称业务实现 / package stable
- **tools**：镜像已本地化；仅 `crates/evidence` 最小面落地；goalctl / xtask / verifyctl **未**宣称落地
- 禁止在 `.agents/ssot/**` 镜像内直接编辑；上游变更用 **删除感知**同步（`rsync -a --delete`，见 SSOT.md R6）
- 对齐审计总览：[docs/ssot/workspace-ssot-alignment.md](./docs/ssot/workspace-ssot-alignment.md)
  - kernel：[docs/ssot/kernel-ssot-alignment.md](./docs/ssot/kernel-ssot-alignment.md)
  - testkit：[docs/ssot/testkit-ssot-alignment.md](./docs/ssot/testkit-ssot-alignment.md)
  - types：[docs/ssot/types-ssot-alignment.md](./docs/ssot/types-ssot-alignment.md)
  - configx：[docs/ssot/configx-ssot-alignment.md](./docs/ssot/configx-ssot-alignment.md)
  - schedulex：[docs/ssot/schedulex-ssot-alignment.md](./docs/ssot/schedulex-ssot-alignment.md)
  - bootstrap：[docs/ssot/bootstrap-ssot-alignment.md](./docs/ssot/bootstrap-ssot-alignment.md)
  - evidence：[docs/ssot/evidence-ssot-alignment.md](./docs/ssot/evidence-ssot-alignment.md)
  - tools：[docs/ssot/tools-ssot-alignment.md](./docs/ssot/tools-ssot-alignment.md)
  - observex：[docs/ssot/observex-ssot-alignment.md](./docs/ssot/observex-ssot-alignment.md)
  - resiliencx：[docs/ssot/resiliencx-ssot-alignment.md](./docs/ssot/resiliencx-ssot-alignment.md)
  - transport：[docs/ssot/transport-ssot-alignment.md](./docs/ssot/transport-ssot-alignment.md)
  - contracts：[docs/ssot/contracts-ssot-alignment.md](./docs/ssot/contracts-ssot-alignment.md)
  - adapters：[docs/ssot/adapters-ssot-alignment.md](./docs/ssot/adapters-ssot-alignment.md)

## 仓库结构

```text
infra.rs/
├── crates/           # Rust workspace members
├── examples/         # 示例
├── tests/            # 集成测试
├── docs/             # 文档（governance/ssot/status/decisions）
├── scripts/          # Harness 健康检查 / GC / worktree 策略
├── .cargo/           # Cargo 配置、target-dir、工具缓存约定
├── .claude/          # Claude Code：skills / hooks / settings
├── .codex/           # Codex：agents / hooks
├── .github/          # CI/CD 与协作模板
├── AGENTS.md         # 本文件
├── CLAUDE.md         # Claude 专属指令
├── Cargo.toml        # Workspace 根
└── README.md
```

## 代理角色

| 系统 | 角色 | 技能来源 |
| ------ | ------ | ---------- |
| **Claude Code** | 主执行代理：编码、审查、交付 | `.claude/skills/` |
| **Codex** | 多模型编排与派工 | `.claude/skills/`（可投影到 `.agents/skills/`） |
| **Copilot** | 补充建议 | 自行管理 |

**SSOT**：技能定义以 `.claude/skills/` 为准；禁止在投影目录手工分叉维护。

## Beads 任务板

跨模型任务通过 Beads（`bd`）协作：

- **Claude Code**：SessionStart `bd prime` 只读注入；需要时 `bd update --claim` / `bd close`
- **Codex**：派工前 claim，验证后 close 或创建 follow-up
- **Copilot**：只读消费，不写 beads

本地 `.beads/` 已 gitignore，不进入版本库。

## 钩子系统

`.claude/settings.json` 生命周期钩子：

| 事件 | 钩子 | 用途 |
| ------ | ------ | ------ |
| SessionStart | `session-context.mjs` + `bd prime` | 上下文与任务记忆 |
| PreToolUse | `pre-tool-check` / `edit-guard` / `count-guard` | 编辑前校验 |
| PostToolUse | `post-tool-check` / `edit-guard-reset` / `link-check` | 编辑后检查 |
| PreCompact | `pre-compact.mjs` | 压缩前保留状态 |
| Stop | `session-review` / `version-guard` / `branch-protect` | 会话审查与护栏 |

## 构建与质量

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check
node scripts/quality-gates/check.mjs
```

- **target-dir**：`.cargo/target/`
- **alias**：`cargo xtask` → `infra-xtask`（crate 尚未添加时可忽略）
- **突变测试输出**：`.cargo/cache/mutants/`

## Git Worktree（强制）

完整细则见 [docs/governance/worktree-policy.md](./docs/governance/worktree-policy.md) 与 [docs/constitution/06-governance.md §6.0.5](./docs/constitution/06-governance.md#605-git-worktree-强制)。

- **所有活跃开发**在 `.worktrees/<branch-name>` 内进行
- 开工：`node scripts/worktree/worktree.mjs create <type>/<id>-<slug>` → `cd` 进入 worktree
- `pre-tool-check` 硬拦截主仓 Write/Edit 与主仓功能分支切换
- 禁止 Agent 使用 `INFRA_WORKTREE_BYPASS=1`

## Git 规范（Main First）

完整条款见 [docs/constitution/06-governance.md §6.0](./docs/constitution/06-governance.md#60-git-main-first强制)。摘要：

- **`main` 唯一主干**：工作必须收敛到 `main`，禁止长期并行主线
- **禁止在 `main` 上直接开发 / 推送**；路径：`分支 → PR → 审查 → CI → 合并 main`
- 从最新 `origin/main` 建支；合并默认 squash；合并后清理分支
- 禁止对 `main` force push；禁止 `--no-verify` 绕过钩子
- Conventional Commits

## 安全

- 不提交 `.claude/*.local.json`、证书、密钥、`.env`
- 不在日志/对话中回显完整 Token
- CI 使用 GitHub Secrets

## 常用 Skill

| 技能 | 场景 |
| ------ | ------ |
| `code-review` | 分支/PR 审查 |
| `codebase-design` / `design-an-interface` | 模块与 API 设计 |
| `diagnosing-bugs` | 故障与回归诊断 |
| `tdd` | 测试先行 |
| `domain-modeling` | 领域语言与 ADR |
| `harness-init` / `harness-mode` / `harness-gc` | Harness 管理 |
| `beads` | 任务板操作 |

## 任务处理流程

### 生命周期

```text
接收 → 分析 → 分解 → 执行 → 验证 → 交付
  │      │      │      │       │       │
  │      │      │      │       │       └─ 更新 beads / 提交 / PR
  │      │      │      │       └─ cargo test + clippy + fmt
  │      │      │      └─ 逐个实现子任务
  │      │      └─ 复杂任务拆分为 beads follow-up
  │      └─ 判读范围：代码 / 配置 / 文档 / CI
  └─ 用户输入或 beads ready
```

### 优先级规则

| 优先级 | 触发条件 | 示例 |
|--------|---------|------|
| P0 | 阻塞性安全/构建/CI 修复 | CVE 修复、CI 红改绿 |
| P1 | 用户显式请求 | 新功能、审查、重构 |
| P2 | beads ready / 依赖 P1 的 follow-up | 子任务、文档补全 |
| P3 | 代码质量改进 | clippy 警告、dead code 清理 |

### 任务边界

- **单任务单会话**：每个 `bd claim` 只在一个会话中执行
- **原子交付**：任务完成后立即 `bd close`，不跨会话堆积
- **子任务委托**：大任务拆分为 `bd create` 子项，不自我膨胀
- **不清楚先问**：范围不明确时先确认，不做假设

### 执行检查清单

每项任务完成前：

- [ ] `cargo fmt --all --check` 通过
- [ ] `cargo clippy --workspace -- -D warnings` 通过
- [ ] `cargo test --workspace` 通过
- [ ] 文档已更新（API doc / CHANGELOG / CONSTITUTION）
- [ ] 关联 beads 状态已更新
- [ ] 提交信息遵循 Conventional Commits

### 委托与接力

- **Codex 审查**：PR 提交后 `codex review --base main`
- **人工审批**：AI 不可 self-approve（§7.1），需 `@xhyperium/maintainers`
- **失败处理**：3 次尝试后仍失败 → 记录原因 → 创建 follow-up → 移交给人类

<!-- BEGIN BEADS INTEGRATION v:1.0 profile:minimal hash:970c3bf2 -->
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
   bd dolt push
   git push
   git status
   ```

5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**

- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

<!-- BEGIN BEADS CODEX SETUP: generated by bd setup codex -->
## Beads Issue Tracker

Use Beads (`bd`) for durable task tracking in repositories that include it. Use the `beads` skill at `.agents/skills/beads/SKILL.md` (project install) or `~/.agents/skills/beads/SKILL.md` (global install) for Beads workflow guidance, then use the `bd` CLI for issue operations.

### Quick Reference

```bash
bd ready                # Find available work
bd show <id>            # View issue details
bd update <id> --claim  # Claim work
bd close <id>           # Complete work
bd prime                # Refresh Beads context
```

### Rules

- Use `bd` for all task tracking; do not create markdown TODO lists.
- Run `bd prime` when Beads context is missing or stale. Codex 0.129.0+ can load Beads context automatically through native hooks; use `/hooks` to inspect or toggle them.
- Keep persistent project memory in Beads via `bd remember`; do not create ad hoc memory files.

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See [SYNC_CONCEPTS.md](https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md) for details and anti-patterns.
<!-- END BEADS CODEX SETUP -->
