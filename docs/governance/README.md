# governance/ — 治理与约定

## 职责

存放**长期有效**的工程约定与宪章落地细则。内容应可被 Agent、CI 与人类共同引用。

## 收录标准

**应放入本目录：**

- 宪章条款的实施细则（语言、版本、worktree 等）
- 跨 crate 的强制规范
- 领域专项规范（如量化）

**不应放入本目录：**

- 工程宪章正文 / 条款导航 → `docs/constitution/`（根 `CONSTITUTION.md` 仅为兼容索引）
- SSOT 对齐矩阵 / 同步报告 → `docs/ssot/`
- CI 运行快照、配置检查记录 → `docs/status/`
- 架构决策（DDR）→ `docs/decisions/`
- 单 crate API/设计 → `crates/<name>/docs/`

## 文档

| 文档 | 宪章锚点 | 说明 |
|------|----------|------|
| [组织 Rust 编码规范完整版 v2.0](https://github.com/xhyperium/.github/blob/main/rulesets/rust/RULES.md) | §4.0 | Rust 全局编码规范（强制上位）— SSOT（`xhyperium/.github` `rulesets/rust/`；上游镜像 bytechainx）；本仓可加严不可削弱 |
| [setup-global-rules.sh](https://github.com/xhyperium/.github/blob/main/scripts/setup-global-rules.sh) | §4.0 | Rust 全局编码规范 — Agent 机器一键分发 `~/.claude/rules/rust.md` |
| [ci-rust-standard / foundation](https://github.com/xhyperium/.github/tree/main/workflows) | §5 | 质量门禁 — 组织可复用 Rust CI（本仓见 `.github/workflows/ci-rust-org.yml`） |
| [VERSIONING.md](VERSIONING.md) | §6.2 | 版本策略 — 项目 / 宪章 / Crate 版本规则 |
| [support-matrix.md](support-matrix.md) | §5 | 质量门禁 — 官方支持矩阵（Linux x86_64 + MSRV 1.85） |
| [prod-signoff-TEMPLATE.md](prod-signoff-TEMPLATE.md) | §6.1 | 变更流程 — L1–L5 模板；**仅 Maintainer 签核** |
| [worktree-policy.md](worktree-policy.md) | §6.0.5 | Git Worktree 强制 — 隔离开发环境策略 |
| [编码与语言约定.md](编码与语言约定.md) | §4.5 | 语言与编码（强制）— UTF-8、中文注释、文档语言 |
| [ASD-STE100.md](ASD-STE100.md) | §4.6 | 文档标准：ASD-STE100（强制）— 英文技术文档 STE 落地指南 |
| [commit-template.md](commit-template.md) | §4.3.3 | 分支与标签 — `.gitmessage` 提交信息模板使用指南 |
| [quant-dev-spec.md](quant-dev-spec.md) | — | — 量化金融专项要求 |

上级索引：[docs/README.md](../README.md)。  
API baseline：[`../api-baselines/`](../api-baselines/)。  
发布签核副本目录：[`../plans/releases/`](../plans/releases/)。
