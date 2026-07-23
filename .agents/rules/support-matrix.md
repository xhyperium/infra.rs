# 支持矩阵（Support Matrix）

> **DEFER-6 处置**：Accept **Linux-only** 官方支持面。  
> 本文件声明 `infra.rs` 当前**官方支持**平台与工具链，作为 CI 与发布门禁的引用 SSOT。

---

## 官方支持（Official）

| 维度 | 声明 | 说明 |
|------|------|------|
| **操作系统** | **Linux** | 仅 Linux 为官方支持 OS |
| **CPU 架构** | **x86_64**（`x86_64-unknown-linux-gnu`） | CI 主跑手（`ubuntu-latest`） |
| **MSRV** | **Rust 1.85** | 见 workspace `Cargo.toml` `rust-version = "1.85"` 与 `ci-rust.yml` MSRV job |
| **Edition** | **2024** | workspace `edition = "2024"` |
| **CI 主轨** | GitHub Actions `ubuntu-latest` | 格式 / clippy / test / coverage / MSRV 均在此运行 |

**“官方支持”含义**：

- 对上述组合的回归、缺陷修复与 semver 承诺优先保障
- PR 合并门禁以 Linux x86_64 + MSRV 1.85 的绿为必要条件
- 公开 API / 覆盖率 / 专项门禁均默认假设该矩阵

---

## 非官方 / 尽力而为（Best-effort）

以下环境**可能**可构建或运行，但**不**构成发布 blocker，也**不**保证 API/行为一致：

| 环境 | 状态 | 备注 |
|------|------|------|
| Linux aarch64 | Best-effort | 无强制 CI 矩阵；欢迎社区补丁 |
| macOS（x86_64 / aarch64） | Best-effort | 开发者本地可用，无官方 runner 承诺 |
| Windows（MSVC / GNU） | Best-effort | 路径/锁/时间语义差异未纳入回归 |
| 其他 Unix | Best-effort | 未声明 |

在非官方环境发现问题：可开 issue，优先级低于官方矩阵；修复以不破坏 Linux x86_64 为准。

---

## 与版本 / API 门禁的关系

| 机制 | 路径 | 关系 |
|------|------|------|
| MSRV 构建 | `.github/workflows/ci-rust.yml`（job `msrv`） | 锁死 1.85 |
| 公开 API 快照 | `docs/api-baselines/` + `scripts/quality-gates/check-public-api.mjs` | 在官方矩阵上生成与比对 |
| 版本策略 | [`VERSIONING.md`](VERSIONING.md) | SemVer 与破坏性变更流程 |
| 生产签核模板 | [`prod-signoff-TEMPLATE.md`](prod-signoff-TEMPLATE.md) | L 级签核需注明支持矩阵 |

破坏性平台扩展（例如将 macOS 升为官方支持）须：

1. 更新本文件  
2. 扩展 CI matrix  
3. 在 PR / CHANGELOG 显式声明  
4. Maintainer 在签核包中确认  

---

## 变更记录

| 日期 | 变更 |
|------|------|
| 2026-07-21 | 初版：DEFER-6 Accept Linux-only；x86_64 + MSRV 1.85（PLAN-CORE-PROD-002 W5 / `infra-asa.6`） |
