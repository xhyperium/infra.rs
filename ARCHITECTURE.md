# infra.rs 架构文档

## 概述

**infra.rs** 是 [xhyper.rs](https://github.com/xhyperium/xhyper.rs) Rust HTTP 框架的基础设施与治理仓库，同时承载所有 AI 编码助手的共享配置、CI/CD 流水线与工程约定。

### 项目身份

| 属性 | 值 |
|------|-----|
| 类型 | Rust Cargo workspace |
| 版号 | 2024 |
| MSRV | 1.85 |
| 许可证 | MIT |
| 仓库 | https://github.com/xhyperium/infra.rs |

### 非目标

- 不是 xhyper.rs 的镜像或子模块
- 不包含产品级运行时代码 — 仅基础设施与工具链
- 不替代上游运营 — 治理与 CI 约束通过配置声明而非强制同步

---

## 仓库结构

```
infra.rs/
├── Cargo.toml                 # Workspace 根清单
├── Cargo.lock                 # 锁文件（v4）
├── clippy.toml                # Clippy 行为阈值配置
├── deny.toml                  # cargo-deny 安全审计规则
├── rustfmt.toml               # Rustfmt 格式化规则
├── rust-toolchain.toml        # Rust 工具链声明
├── .editorconfig              # 编辑器通用配置
├── Makefile                   # 快捷命令入口
├── LICENSE                    # MIT
│
├── crates/                    # Rust workspace 成员
│   ├── kernel/                # L0 语义信任根
│   ├── testkit/               # 测试支持（dev-only）
│   └── types/
│       ├── decimal/           # 十进制数值 / Money
│       └── canonical/         # 跨层共享 DTO
│
├── scripts/                   # Harness 脚本
│   ├── check.mjs              # 健康检查
│   ├── check-constitution.mjs  # 宪章合规验证
│   ├── gc-scan.mjs            # 垃圾回收扫描
│   └── worktree-policy.mjs    # Worktree 策略
│
├── .github/                   # GitHub 配置
│   └── workflows/             # CI/CD 工作流（6 个）
│
├── .claude/                   # Claude Code 配置
│   ├── settings.json          # 生命周期钩子注册
│   ├── hooks/                 # 12 个钩子脚本
│   └── skills/                # 46 个技能定义
│
├── .codex/                    # Codex CLI 配置
│   └── agents/                # 28 个代理定义
│
├── docs/                      # 项目文档
│   ├── CI_STATUS_REPORT.md    # CI 工作流矩阵与运行统计
│   ├── 编码与语言约定.md       # 编码与文档语言约定
│   └── ...
│
├── examples/                  # 示例代码
└── tests/                     # 集成测试
```

---

## Crate 架构

### 层次模型

```
┌────────────────────────────────────────────┐
│  tests/ / examples/                        │  跨 crate 集成与示例
├────────────────────────────────────────────┤
│  crates/testkit/                           │  T0 test-support（仅 dev-dep）
│  └── ManualClock 族                        │
├────────────────────────────────────────────┤
│  crates/types/canonical/                   │  跨层共享纯 DTO
│  crates/types/decimal/                     │  Decimal / Money
├────────────────────────────────────────────┤
│  crates/kernel/                            │  L0 语义信任根
│  └── clock / lifecycle                     │
└────────────────────────────────────────────┘
```

### 依赖规则

```
canonical  →  decimalx  →  kernel
testkit    →  kernel          # 仅 [dev-dependencies]
```

- 所有 crate 通过 `[workspace.dependencies]` 统一管理版本
- 禁止循环依赖
- L0 / types 层不依赖外部运行时或平台特定代码
- 新增 crate 须在 `crates/` 下注册，遵循单一职责与标准布局

---

## CI/CD 架构

### 工作流矩阵

```
                     push/PR 到 main
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   ┌────▼────┐      ┌─────▼─────┐     ┌─────▼─────┐
   │ CI(Rust) │      │   质量     │     │   校验     │
   │ build    │      │ rustfmt   │     │ yamllint  │
   │ test     │      │ clippy    │     │ taplo     │
   │ msrv     │      │ cargo doc │     │ md lint   │
   │ coverage │      │           │     │ codespell │
   └──────────┘      └───────────┘     │ lychee    │
                                       │ harness   │
   ┌──────────┐      ┌───────────┐     └───────────┘
   │   安全    │      │Constitution│
   │ deny     │      │ full check │
   │ audit    │      │           │
   └──────────┘      └───────────┘
```

### 触发策略

| 工作流 | 触发条件 | 定时 |
|--------|---------|------|
| CI (Rust) | Cargo / crate / rust-toolchain 变更 | — |
| 质量 | .rs / rustfmt.toml / clippy.toml 变更 | — |
| 校验 | 全部 push / PR | — |
| 安全 | Cargo / deny.toml 变更 | 每周一 02:00 |
| Constitution | Rust / config / CONSTITUTION.md 变更 | — |

详见 [CI_STATUS_REPORT.md](CI_STATUS_REPORT.md)。

### 构建缓存

- `Swatinem/rust-cache@v2` 在所有 Rust 编译 Job 中使用
- 构建产物统一输出到 `.cargo/target/`（gitignored）

---

## 治理系统

### 宪章（CONSTITUTION.md）

五大核心价值观决定了所有技术决策：

| 价值 | 约束 |
|------|------|
| 安全优先 | 变更不得降低安全标准；依赖须通过 cargo-deny 审计 |
| 可观测 | 关键路径有追踪；错误可追溯 |
| 可验证 | `check` / `test` / `fmt --check` 为门禁底线；覆盖率 ≥ 80% |
| 自动化优先 | CI 是唯一仲裁者；机器保证的不依赖人工 |
| 简单优于灵活 | YAGNI；每加一层间接必须有可论证收益 |

### AI 代理角色

```
┌─────────────────────────────────────────────┐
│              AGENTS.md                       │
│           共享治理（SSOT）                    │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │Claude Code│  │  Codex   │  │ Copilot  │  │
│  │ 主执行    │  │ 编排派工 │  │ 补充建议 │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │         │
│  12 hooks      28 agents      auto-manage  │
│  46 skills     .agents/skills (投影)        │
│  .claude/      .codex/         .copilot/    │
└─────────────────────────────────────────────┘
```

SSOT 规则：所有技能定义以 `.claude/skills/` 为唯一事实源；`.codex/.agents/skills/` 为自动投影，禁止手工编辑。

### 钩子生命周期

```
SessionStart ──► PreToolUse ──► PostToolUse ──► PreCompact ──► Stop
     │               │              │               │            │
 session-       pre-tool-     post-tool-      pre-        session-
 context        check         check           compact     review
 bd prime       edit-guard    edit-guard-                 version-
                count-guard   reset                      guard
                             link-check                  branch-
                                                         protect
```

---

## 工程约定

### 语言与编码

| 范围 | 语言 | 编码 |
|------|------|------|
| 代码注释 / 文档 | 中文（技术术语保留英文） | UTF-8 无 BOM |
| 用户可见错误 | 中文 | UTF-8 无 BOM |
| 标识符 | 英文（Rust 惯例） | ASCII |
| 许可证 | 英文原文 | UTF-8 |
| 换行符 | — | LF |

### Git 规范

- `main` 唯一主干，受保护
- 开发走 feature 分支 + PR，合并 squash
- Conventional Commits
- 禁止 force push / `--no-verify`

### 本地开发

```bash
make build     # 编译
make test      # 测试
make fmt-check # 格式检查
make lint      # Clippy
make deny      # 安全审计
make ci        # 完整 CI 模拟（fmt + lint + test + deny）
```

---

## 设计决策记录

详细内容见 [docs/decisions/](docs/decisions/)（DDR-001 ~ DDR-009）。

| DDR | 决策 | 理由 |
|-----|------|------|
| 001 | Rust 2024 + MSRV 1.85 | 采用最新稳定版号，利用新版特性 |
| 002 | thiserror v2 | 成熟稳定的错误派生方案 |
| 003 | serde 手动实现 Error 序列化 | 完整保留 IO 错误链跨序列化边界 |
| 004 | 中文优先注释 | 团队母语，降低认知负担 |
| 005 | 统一 target-dir 到 .cargo/ | 避免多位置 target 缓存碎片 |
| 006 | Beads（bd）任务跟踪 | 跨 AI 模型可协作，本地 DB 不入版本库 |
| 007 | Squash Merge | 保持 main 线性历史 |
| 008 | AI 治理三层分层 | SSOT 技能源，钩子全生命周期覆盖 |
| 009 | taiki-e/install-action 统一安装 | 统一 CI 工具安装接口 |
