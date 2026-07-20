# infra.rs

[![CI (Rust)](https://github.com/xhyperium/infra.rs/actions/workflows/ci-rust.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/ci-rust.yml)
[![质量](https://github.com/xhyperium/infra.rs/actions/workflows/quality.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/quality.yml)
[![校验](https://github.com/xhyperium/infra.rs/actions/workflows/validation.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/validation.yml)
[![安全](https://github.com/xhyperium/infra.rs/actions/workflows/security.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/security.yml)
[![Constitution](https://github.com/xhyperium/infra.rs/actions/workflows/constitution.yml/badge.svg)](https://github.com/xhyperium/infra.rs/actions/workflows/constitution.yml)

**infra.rs** 是 [xhyper.rs](https://github.com/xhyperium/xhyper.rs) 项目的基础设施与治理仓库，承载以下职责：

- 可复用的 Rust 核心库（`infra-core`）
- 统一的工程约定、工具链配置与 CI/CD 流水线
- 多 AI 编码助手（Claude Code / Codex / Copilot）的共享治理配置

## 快速开始

### 前置条件

- [Rust](https://rustup.rs) >= 1.85（MSRV）
- [Node.js](https://nodejs.org) >= 18（Harness 脚本）

```bash
# 克隆仓库
git clone https://github.com/xhyperium/infra.rs.git
cd infra.rs

# 构建全部 crate
cargo build --workspace

# 运行测试
cargo test --workspace

# 完整 CI 模拟（格式 + Lint + 测试 + 安全审计）
make ci
```

### 使用 Make 快捷命令

```bash
make build       # 编译
make test        # 测试
make fmt-check   # 格式检查
make lint        # Clippy 静态分析
make deny        # 依赖安全审计
make ci          # 本地完整 CI
make check       # 宪章合规性验证
```

### 作为依赖引入

在 `Cargo.toml` 中引用本仓库的 crate：

```toml
[dependencies]
infra-core = { git = "https://github.com/xhyperium/infra.rs.git" }
```

或通过 crates.io（待发布）：

```toml
[dependencies]
infra-core = "0.1"
```

### 初始化 Harness

```bash
# 健康检查（验证 hooks / skills / beads 就绪）
node scripts/check.mjs

# 初始化 Beads 任务板
bd init && bd prime
```

## Git Worktree（强制）

所有活跃开发须在独立的 [Git Worktree](https://git-scm.com/docs/git-worktree) 中进行，保持 `main` 工作区干净。

```bash
# 加载快捷命令（建议写入 ~/.bashrc）
eval "$(node scripts/worktree-activate.mjs)"

# 创建 worktree 并切换
wt feat/my-feature

# 回到 main 工作区
wt main

# 列出所有 worktree
wt

# 管理命令
./scripts/worktree.mjs create feat/my-feature   # 创建
./scripts/worktree.mjs remove feat/my-feature   # 删除
./scripts/worktree.mjs prune                    # 清理
```

路径约定：`.worktrees/<branch>`，分支 `/` 保留为目录分隔符。

## Workspace

| Crate | 说明 |
|-------|------|
| `infra-core` | 核心基础设施库 — 错误类型、Result 别名、serde 序列化 |

Rust edition `2024`，MSRV `1.85`。完整结构见 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 工程约定

| 类别 | 约定 |
|------|------|
| 构建 | `.cargo/target/`（gitignored），`cargo build --workspace` |
| 格式 | `cargo fmt --all`，`max_width = 100` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| 测试 | `cargo nextest` + `cargo llvm-cov`，覆盖率 >= 80% |
| 安全 | `cargo deny check`，禁止 `unsafe` |
| 提交 | Conventional Commits |
| 分支 | `main` 受保护，feature 分支 + PR，squash merge |
| 任务 | Beads（`bd`），本地 Dolt DB |

## 文档

完整索引见 [docs/README.md](docs/README.md)。

| 文档 | 说明 |
|------|------|
| [CONSTITUTION.md](CONSTITUTION.md) | 工程宪章 — 核心价值观、架构原则、代码标准 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 架构文档 — 结构、Crate、CI/CD、治理 |
| [ASD-STE100.md](docs/ASD-STE100.md) | 英文技术文档规范（ASD-STE100 / STE） |
| [编码与语言约定.md](docs/编码与语言约定.md) | 中文编码与文档语言约定 |
| [CI_STATUS_REPORT.md](docs/CI_STATUS_REPORT.md) | CI 工作流矩阵、触发条件与运行统计 |
| [CONFIG_SUMMARY.md](docs/CONFIG_SUMMARY.md) | CI 配置、分支保护规则、测试验证记录 |
| [DDR 索引](docs/decisions/) | 架构决策记录（DDR-001 ~ DDR-009） |

## AI 协作

| 系统 | 角色 |
|------|------|
| Claude Code | 主执行：编码、审查、交付 |
| Codex | 编排与派工 |
| Copilot | 补充建议 |

技能 SSOT：`.claude/skills/`。任务跟踪：`bd`。

## 许可

MIT © 2026 [xhyperium](https://github.com/xhyperium)
test
