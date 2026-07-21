# CHANGELOG

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **`xhyper-schedulex` 0.1.0**（`crates/schedulex`）：active SSOT 最小任务 ID 登记表；std-only；line cov 100%
- **`docs/schedulex-ssot-alignment.md`**：schedulex 本仓对齐矩阵
- **storage adapters 收敛到 `xhyper-contracts`**：redisx→KeyValueStore/PubSub；kafkax/natsx→EventBus；postgresx→Repository/TxRunner；ossx→ObjectStore；clickhousex→AnalyticsSink；taosx→TimeSeriesStore（均内存 scaffold）
- **`natsx` / `postgresx` / `clickhousex` / `ossx` / `taosx`**：StorageAdapter 内存 scaffold（与 redisx/kafkax 同模式）
- **`redisx` / `kafkax`**：实现 `StorageAdapter`（进程内 KV scaffold + 单元测试）
- **`okxx`**：实现 `ExchangeAdapter` trait（占位 ticker；5 单元测试）
- **contracts SSOT 镜像**：`.agents/ssot/contracts/`（16 文件，与上游 0 diff）；对齐文 `docs/contracts-ssot-alignment.md`
- **`xhyper-bootstrap`**（lib `bootstrap`）：L1 唯一组合根（ADR-016）；typed `PlatformContext` / `AppContext` / `BootstrappedApp`；可移植 trait 替面；对齐文档 `docs/bootstrap-ssot-alignment.md`
- **`xhyper-configx`（lib `configx`）**：落地 active SSOT 0.1.0 内存字符串 KV（`ConfigStore`）；生产依赖仅 `xhyper-kernel`；对齐文 `docs/configx-ssot-alignment.md`
- **Crate 子模块标准布局**（`crates/AGENTS.md`）：强制 `src/`、`tests/`、`examples/`、`docs/`、`README.md`、`AGENTS.md`、`CHANGELOG.md`
- **`kernel` 等 crate 骨架补齐**：README、CHANGELOG、AGENTS、空目录 `.gitkeep`
- **adapters SSOT 镜像**（只读）：`exchange/{binance,okx}` + `storage/{clickhouse,kafka,nats,oss,postgres,redis,taos}`（144 文件，与上游 0 diff）
- **对齐文档**：[docs/adapters-ssot-alignment.md](./docs/adapters-ssot-alignment.md)（镜像 vs scaffold 状态矩阵）
- **adapters / contracts 标准布局补齐**：9 adapter crate + `xhyper-contracts` 补齐七项骨架；各 crate 显式 `publish = false`

### Removed

- **`infra-core` crate**：从 workspace 移除；L0 基础层以 `xhyper-kernel` 为准，类型层以 `types/*` 为准

### Changed

- workspace members 增加 `xhyper-schedulex`；总览/对齐文档同步
- 同步更新 `README` / `ARCHITECTURE` / `CONSTITUTION` / `crates/AGENTS.md` / `docs/VERSIONING.md` 中的 crate 地图
- DDR-002 适用范围改为 workspace 库 crate；DDR-003 标记为已撤销（依赖已删除的 `infra_core::Error`）
- 对齐文档：新增 `docs/workspace-ssot-alignment.md`、`docs/types-ssot-alignment.md`、`docs/configx-ssot-alignment.md`、`docs/bootstrap-ssot-alignment.md`；更新 `SSOT_SYNC_REPORT` / `docs/README` / 根 `AGENTS.md` / `CLAUDE.md` 入口
- **adapters SSOT 本地化**：注册 `.agents/ssot/adapters/` 镜像；更新 R6/R7、`workspace`/`SSOT_SYNC` 总览与 `crates/AGENTS` 概览

---

## [0.3.0] — 2026-07-21

### Added

- **文档系统**: `docs/README.md` 文档索引，`docs/decisions/` ADR 目录与模板
- **配置总结**: `docs/CONFIG_SUMMARY.md` — CI 工作流、分支保护规则、验证记录总览
- **主 README 文档链接**: 项目 README 新增文档章节，链接到全部 4 份文档
- **ASD-STE100 指南** (`docs/ASD-STE100.md`): 英文技术文档受控语言规范
- **中文编码约定** (`docs/编码与语言约定.md`): UTF-8 与编码策略

---

## [0.2.0] — 2026-07-21

### Added

- **工程宪章** (`CONSTITUTION.md`): 核心价值观、架构原则、代码标准、质量门禁、AI 代理章程
- **CI 工作流**: `validation.yml`, `quality.yml`, `ci-rust.yml`, `security.yml`, `constitution.yml`
- **PR 模板** (`.github/PULL_REQUEST_TEMPLATE.md`): 标准化格式（类型、Issue、宪章检查清单、验证方式）
- **PR 模板校验** (`scripts/check-pr-template.mjs` + `.github/workflows/pr-template-check.yml`)
- **宪章合规性验证** (`scripts/check-constitution.mjs`): 一键运行全部强制门禁
- **Makefile**: `make check`, `make ci`, `make fmt`, `make test` 等 16 条快捷命令
- **Git Main First** (§6.0): 强制执行主干集成与分支保护
- **分支保护规则**: PR 强制、1 人 approve、CODEOWNERS、required status checks、线性历史
- **Auto-merge**: 启用，PR 通过全部检查后自动 squash merge
- **`clippy.toml`**: 复杂度阈值与 lint 行为配置

### Changed

- PR 模板增强：新增变更类型标签、宪章合规性检查清单、验证方式代码块

### Security

- **CVE-2024-48908**: 升级 `lychee-action` v1 → v2，修复代码注入漏洞

### Removed

- **Dependabot**: 移除 `dependabot.yml`，禁用自动依赖更新

---

## [0.1.0] — 2026-07-20

### Added

- **项目初始化**: 独立 Rust workspace (`Cargo.toml`)
- **`infra-core` crate**: 核心基础设施库
  - 错误类型 `Error` (I/O, Config, InvalidArgument, Internal) with `thiserror`
  - `Result<T>` 类型别名
  - 自定义 `serde::Serialize` / `Deserialize` 实现
  - `io::Error` source 链保留机制（`ChainNode`）
- **工具配置**: `rustfmt.toml`, `deny.toml`, `.cargo/config.toml`
- **CI 基础**: `.github/workflows/`, `.github/ISSUE_TEMPLATE/`, `CODEOWNERS`
- **Claude Code**: hooks, skills, session 治理 (`.claude/`)

---

[0.3.0]: https://github.com/xhyperium/infra.rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/xhyperium/infra.rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/xhyperium/infra.rs/releases/tag/v0.1.0
- E2E complete flow test
