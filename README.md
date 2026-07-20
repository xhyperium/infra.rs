# infra.rs

[![CI (Rust)](https://github.com/xhyperium/infra.rs/actions/workflows/ci-rust.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/ci-rust.yml)
[![质量](https://github.com/xhyperium/infra.rs/actions/workflows/quality.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/quality.yml)
[![校验](https://github.com/xhyperium/infra.rs/actions/workflows/validation.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/validation.yml)
[![安全](https://github.com/xhyperium/infra.rs/actions/workflows/security.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/security.yml)
[![Constitution](https://github.com/xhyperium/infra.rs/actions/workflows/constitution.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/constitution.yml)

独立的 Rust 基础设施工作区（workspace）。提供可复用的核心库、工程约定，以及 AI 编码助手的治理配置。

## 快速开始

```bash
# 构建
cargo build --workspace

# 测试
cargo test --workspace

# 格式 / Lint
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings

# 依赖安全
cargo deny check

# Harness 健康检查
node scripts/check.mjs
```

## 仓库结构

```text
infra.rs/
├── crates/               # Rust workspace crates
│   └── infra-core/       #   核心库
├── examples/             # 示例（按需添加）
├── tests/                # 集成测试（按需添加）
├── docs/                 # 项目文档
├── scripts/              # Harness 脚本（check / gc-scan / worktree-policy）
├── .cargo/               # Cargo 配置与本地 target/cache
├── .claude/              # Claude Code：skills / hooks / settings
├── .codex/               # Codex：agents / hooks
├── .github/              # CI/CD、Issue/PR 模板
├── Cargo.toml            # Workspace 根
├── AGENTS.md             # 全 Agent 共享治理
├── CLAUDE.md             # Claude Code 专属指令
├── deny.toml             # cargo-deny
├── rustfmt.toml          # rustfmt
└── README.md
```

## Workspace

| Crate | 说明 |
| ------- | ------ |
| `infra-core` | 核心基础设施库（起点脚手架） |

Rust edition `2024`，MSRV 见 `Cargo.toml` 中 `rust-version`。

## 工程约定

- 构建产物：`.cargo/target/`（已 gitignore）
- 工具缓存：`.cargo/cache/<tool>/`
- 提交信息：Conventional Commits
- 分支：`main` 受保护；功能开发走 feature 分支 + PR
- 任务板：Beads（`bd`），本地 DB 不入库

## 文档

完整索引见 [docs/README.md](docs/README.md)。

| 文档 | 说明 |
| ------ | ------ |
| [CONSTITUTION.md](CONSTITUTION.md) | 工程宪章 — 核心价值观、架构原则、代码标准 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 架构文档 — 结构、Crate、CI/CD、治理 |
| [CI_STATUS_REPORT.md](docs/CI_STATUS_REPORT.md) | CI 工作流矩阵、触发条件与运行统计 |
| [ASD-STE100.md](docs/ASD-STE100.md) | 英文技术文档规范 |
| [ADR-001-009.md](docs/decisions/ADR-001-009.md) | 架构决策记录（DDR-001 ~ DDR-009） |
| [编码与语言约定.md](docs/编码与语言约定.md) | 中文编码与文档语言约定 |

## AI 协作

| 系统 | 角色 |
| ------ | ------ |
| Claude Code | 主执行：编码、审查、交付 |
| Codex | 编排与派工 |
| Copilot | 补充建议 |

技能 SSOT：`.claude/skills/`。任务跟踪：`bd`。

## 许可

MIT

## 语言与编码

- 注释与文档使用**中文**；标识符英文
- 全文 **UTF-8（无 BOM）**，见 [docs/编码与语言约定.md](./docs/编码与语言约定.md)
