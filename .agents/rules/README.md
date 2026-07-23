# `.agents/rules/` — 本仓规则 SSOT

本目录是 **infra.rs 项目级规则文档** 的统一入口（与 `.agents/skills/`、`.agents/ssot/` 并列）。

组织级规则（Agent 全局加载）仍在 `~/.claude/rules/`（`xhyperium/.github` → `rulesets/`），**不**复制到本目录。

## 分层

```text
组织 rulesets/          # 全局 P0（language / rust / agent-*）— 本机 ~/.claude/rules
        ↓ 可加严，不可削弱
.agents/rules/          # 本仓规则 SSOT（本目录）
        ↓ 人类索引兼容
docs/governance/        # 仅重定向 stub + 本目录说明（非正文）
docs/constitution/      # 工程宪章正文（独立层级，高于规则落地细则）
```

## 收录标准

**应放入本目录：**

- 强制工程约定与开发规则（worktree、版本、提交、复用）
- 本仓对组织规范的加严落地（Rust、语言、STE）
- 跨 crate 的领域专项规则（量化、存储适配器配置）
- Agent / CI / 人类共同引用的长期有效规则

**不应放入本目录：**

- 工程宪章正文 → `docs/constitution/`
- 域规格 SSOT → `.agents/ssot/`
- Agent skills → `.agents/skills/`
- 一次性审计报告 → `docs/report/`
- 架构决策（DDR）→ `docs/decisions/`

## 文档索引

| 文档 | 宪章锚点 | 说明 |
|------|----------|------|
| [项目开发规则.md](./项目开发规则.md) | — | 开发规则总览（文档/工程/复用/门禁/提交） |
| [rust-dev-rules.md](./rust-dev-rules.md) | §4.0 | Rust 开发规则补充（本仓落地与加严） |
| [编码与语言约定.md](./编码与语言约定.md) | §4.5 | UTF-8、强制中文 |
| [ASD-STE100.md](./ASD-STE100.md) | §4.6 | 英文交付可选指南（须书面豁免） |
| [文档组织约定.md](./文档组织约定.md) | — | 文档目录与报告路径 |
| [worktree-policy.md](./worktree-policy.md) | §6.0.5 | Git Worktree 强制策略 |
| [VERSIONING.md](./VERSIONING.md) | §6.2 | 版本策略（crate 独立版本 / PATCH +1） |
| [commit-template.md](./commit-template.md) | §4.3.3 | `.gitmessage` 提交模板 |
| [support-matrix.md](./support-matrix.md) | §5 | 官方支持矩阵 |
| [prod-signoff-TEMPLATE.md](./prod-signoff-TEMPLATE.md) | §6.1 | L1–L5 生产签核模板 |
| [storage-adapter-config.md](./storage-adapter-config.md) | — | storage×7 环境变量与本地库规范 |
| [credential-baseline.md](./credential-baseline.md) | — | 凭据复杂度基线 |
| [quant-dev-spec.md](./quant-dev-spec.md) | §3.3 | 量化金融领域专项 |

## 组织上位（外链，不入库副本）

| 文档 | 说明 |
|------|------|
| [language.md](https://github.com/xhyperium/.github/blob/main/rulesets/language.md) | 组织语言政策（P0） |
| [Rust RULES.md](https://github.com/xhyperium/.github/blob/main/rulesets/rust/RULES.md) | Rust 全局编码规范完整版 |
| [setup-global-rules.sh](https://github.com/xhyperium/.github/blob/main/scripts/setup-global-rules.sh) | 分发到 `~/.claude/rules/` |

## 兼容路径

历史路径 `docs/governance/<name>.md` 保留 **stub 重定向**，正文只在本目录维护。新增规则请直接写入 `.agents/rules/`。

## 变更日志

| 日期 | 变更 |
|------|------|
| 2026-07-23 | 自 `docs/governance/` 迁入；确立本目录为项目规则 SSOT |
