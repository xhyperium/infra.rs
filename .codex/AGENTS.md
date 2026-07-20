# Codex Agents

xhyper.rs 仓库的 Codex 代理配置。

## 目录结构

- `config.toml` — Codex 特性与多代理并发配置（hooks 保持启用）
- `hooks.json` — 单一 SessionStart 钩子；只读注入 Beads 持久记忆，不推断 Git 权限
- `agents/` — 官方项目级自定义 agent；使用复数目录，包含通用角色与 Goal 十一阶段 `goal-*` 投影

## Skill 统一到 `.agent/`（SSOT）

| 角色 | 路径 |
|------|------|
| **Skill 唯一真相** | **`.agent/skills/<name>/`**（与 harness skills 同树） |
| Advisor 正文 | `.agent/skills/advisor/SKILL.md` |
| Advisor 元数据 | `.agent/skills/advisor/agents/openai.yaml` |
| Codex 发现投影 | `.agents/skills/<name>`（由 SSOT 生成的 managed entity，禁止手工编辑） |

说明：

- Codex CLI **硬编码**扫描 `$REPO_ROOT/.agents/skills`（及 cwd 向上的 `.agents/skills`），**不支持**把发现根改成 `.agent/`。
- 因此本仓规则是：**人写 / 审阅 / 编辑只改 `.agent/skills/`**；`.agents/skills/` 只做兼容投影，不得成为第二事实源。
- 新增 Skill：先落盘 `.agent/skills/<name>/`，再运行 `node scripts/agents/project-skills.mjs --write`；
  `scripts/check.mjs` 会验证 `.agents` / `.github` / `.claude` 三套 managed entity 投影与 SSOT 的内容和 mode 一致。
- Advisor 通过 `$advisor` 显式调用，也可从 `/skills` 选择或由触发描述隐式启用；裸 `/advisor` 不是 Codex 内建命令，不得声称可用。

## 说明

Codex 的代理配置已清理，移除了不属于本仓库的 FoundationX 治理体系配置。如需恢复，参见 git 历史。

`executor` 必须通过 `node scripts/grok-executor.mjs` 调用固定摘要的本机 Grok CLI。Grok 只返回候选补丁，不直接修改工作树；Codex 负责审查和应用，root 负责最终验证与交付。

**Beads 任务板**：Grok / OpenCode **不**写 beads。派工前 root 用 `bd update --claim`（可 `BEADS_ACTOR=codex`）；验证后 root `bd close` 或 `bd create` follow-up。OpenCode 仅只读意见，吸收后再改 beads。协议全文见根 `AGENTS.md`「多模型 beads 接入」。

独立任务优先使用 `node scripts/grok-executor-batch.mjs <manifest.json>`；默认并发 8、硬上限 16。Manifest 必须提供 token 或 cost 总预算，并为每项声明输入/输出 token 上界与互斥 `allowed_paths`。CLI 无法强制 provider 单次 token cap 时，batch 只能用保守预留和实际 usage 阻止新任务，不得伪称能中止 provider 内部生成。

用户请求的历史别名只做语义映射：`sol` 使用 `gpt-5.6-sol`，`luna` 使用 `gpt-5.6-terra` 且 reasoning effort 为 low。agent 配置不得写不存在的模型 slug。

Goal 主流程对应 `goal-definer` → `goal-spec` → `goal-designer` → `goal-planner` → `goal-task-splitter` → `goal-prompt-builder` → `goal-coder` → `goal-tester` → `goal-reviewer` → `goal-release` → `goal-retrospective`。这些文件只投影 `docs/goal/`，不成为新的规则源；Matrix 仍是横切制品，不是第十二阶段。

`ai-goal`、`ai-research`、`ai-spec`、`ai-planner`、`ai-executor`、`ai-verifier`、`ai-reviewer`、`ai-release`、`ai-learning` 是「AI / 自动化介入位置」的横切能力角色，不新增或替换上述 canonical 阶段。`ai-*` 输出必须交回对应阶段与 root 裁决；不得用横切角色自批 Goal、Gate、Review、Release 或 Controlled RSI。

文件名使用 `goal-*.toml`，TOML `name` 使用对应 snake_case（如 `goal-definer.toml` → `goal_definer`）。依赖 Codex v2 目录发现，不在 `.codex/config.toml` 添加 legacy `[agents.<role>] config_file` 注册。

其中 9 个分析、审查与治理阶段显式只读；Code 是唯一交付文件 writer，在 root 分配的互斥完整文件内同时完成代码与测试变更。Test 使用 `workspace-write` 仅写构建产物与临时文件，并显式允许仓库固定的 `.cargo/target`，不得修改交付文件。`sandbox_mode` 不提供文件级 allowed_paths 硬隔离，root 仍须通过任务 envelope、互斥所有权和独立 diff review 强制边界。Release 只生成 Manifest 草案并审查 readiness，禁止 commit、tag、push、publish 或 deploy；Retrospective 缺最终 Release 与 Metrics 证据时必须阻断。

横切 `ai-*` 中 Goal / Research / Spec / Planner / Verifier / Reviewer / Learning 只读；Executor 是任务 envelope 内的交付 writer；Verifier 把需要写构建产物的 command plan 交给 root 在可丢弃的隔离 worktree 执行，再独立裁决原始 Evidence；Release 仅在独立 Review PASS 后修改明确分配的 CHANGELOG / manifest / 本地 artifact。Release 仍禁止 commit、tag、push、publish、deploy 或自批 Gate；Learning 只提 CR、修订建议与 AutoResearch 候选，不直接改受保护资产。

Kimi CLI 当前不可用，不创建伪 adapter；研究任务回退到 `researcher` 的 `gpt-5.6-terra` low。OpenCode 1.17.18 仅可由 `architecture_advisor` 在 root 准备的隔离只读 context 目录上以 pure/plan 模式获取可选第二意见，禁止 `--auto` 和可写业务 worktree。
