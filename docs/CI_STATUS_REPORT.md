# infra.rs CI Status Report

**Branch**: `main` | **Date**: 2026-07-21 | **Last validated**: 2026-07-21 `.sh` → `.mjs` migration

## 工作流矩阵

### CI（Rust）

**触发**: Rust 源码变更 (`Cargo.toml`, `Cargo.lock`, `**/Cargo.toml`, `crates/**`, `rust-toolchain.toml`, workflow self)

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| 检测 Rust 源码 | 判断是否需要执行后续 Rust 任务 | 2m |
| 构建 | `cargo build --workspace --all-features` | 15m |
| 测试 | `cargo nextest run --workspace --all-features` | 15m |
| 最低版本兼容 (MSRV 1.85) | 使用 Rust 1.85 构建 | 15m |
| 覆盖率 | `cargo llvm-cov --lcov` → Codecov | 15m |

### 质量

**触发**: Rust 源文件 / 格式化配置变更 (`**/*.rs`, `rustfmt.toml`, `clippy.toml`, workflow self)

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| 检测 Rust 源码 | 判断是否有 `.rs` 文件 | 2m |
| 格式检查 (rustfmt) | `cargo fmt --all --check` | 10m |
| Clippy 静态检查 | `cargo clippy --workspace --all-features --all-targets -- -D warnings` | 15m |
| 文档检查 (cargo doc) | `RUSTDOCFLAGS=-D warnings cargo doc --no-deps --document-private-items` | 10m |

### 校验

**触发**: 全部 push / PR

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| YAML 检查 | `yamllint` | 5m |
| TOML 检查 | `taplo-cli fmt --check` | 5m |
| Markdown 检查 | `markdownlint` (README/AGENTS/CLAUDE/docs) | 5m |
| 拼写检查 | `codespell` (排除 `ser`) | 5m |
| 链接检查 | `lychee` (fail: false) | 10m |
| Harness 健康检查 | `node scripts/check.mjs` | 5m |

### 安全

**触发**: Cargo / deny.toml / rust-toolchain 变更 + 每周一 02:00 UTC

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| 检测 Rust 源码 | 判断 `Cargo.toml` 是否存在 | 2m |
| cargo-deny 依赖策略 | `cargo deny check` (许可证 / 安全公告 / 依赖策略) | 10m |
| cargo-audit 漏洞扫描 | `cargo audit` (仅 schedule / workflow_dispatch) | 10m |

### Constitution

**触发**: Rust 源文件 / 配置文件 / CONSTITUTION.md 变更

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| Constitution Check | `./scripts/check-constitution.mjs` (rustfmt + clippy + test + doc + cargo-deny + unsafe/unwrap/naming 审计) | 15m |

### CodeQL

**触发**: 自动 (GitHub 原生)

| Job | 说明 | 超时 |
| ----- | ------ | ------ |
| 代码安全分析 | GitHub CodeQL 安全扫描 | ~2m |

## 统计

| 指标 | 值 |
| ------ | ----- |
| 总工作流 | 6 (5 自定义 + 1 GitHub 原生) |
| 总 Job | 18 |
| 覆盖率上报 | Codecov (`fail_ci_if_error: false`) |
| 定时任务 | cargo-audit 每周一 02:00 UTC |
| 构建缓存 | `Swatinem/rust-cache@v2` |
| 工具安装 | `taiki-e/install-action@v2` (nextest / llvm-cov / cargo-deny / taplo-cli) |

## 最近运行

| 时间 | 工作流 | 状态 |
| ------ | -------- | ------ |
| 2026-07-20 17:01 | 校验 | success |
| 2026-07-20 17:01 | Constitution | success |
| 2026-07-20 16:58 | 质量 | success |
| 2026-07-20 16:58 | CI（Rust） | success |
| 2026-07-20 16:58 | 安全 | success |
| 2026-07-20 16:58 | 校验 | success |
| 2026-07-20 16:58 | CodeQL | success |

## Markdownlint 配置

`.markdownlint.json`:

```json
{
  "MD013": false,
  "MD024": false
}
```

- **MD013** (line-length): 关闭 — 部分行为自动生成的块和长 URL
- **MD024** (no-duplicate-heading): 关闭 — Beads 集成块有重复标题

## Codespell 配置

排除词: `ser`（Rust serde 序列化的常用缩写）

---

## `.sh` → `.mjs` 迁移验证 (2026-07-21)

### 迁移清单

| 旧脚本 (.sh) | 新脚本 (.mjs) | 状态 |
|-------------|-------------|------|
| `scripts/check-constitution.sh` | `scripts/check-constitution.mjs` | ✓ |
| `scripts/check-pr-template.sh` | `scripts/check-pr-template.mjs` | ✓ |
| `scripts/worktree.sh` | `scripts/worktree.mjs` | ✓ |
| `scripts/worktree-activate.sh` | `scripts/worktree-activate.mjs` | ✓ |
| `scripts/starship-wt.sh` | `scripts/starship-wt.mjs` | ✓ |
| `scripts/migrate-worktrees.sh` | `scripts/migrate-worktrees.mjs` | ✓ |

### CI 工作流验证

| 工作流 | 脚本引用 | 解释器 | 路径过滤 | 状态 |
|--------|---------|--------|---------|------|
| `constitution.yml` | `./scripts/check-constitution.mjs` | shebang (`#!/usr/bin/env node`) | `scripts/check-constitution.mjs` | ✓ |
| `pr-template-check.yml` | `node scripts/check-pr-template.mjs` | `node` (显式) | (全部 PR) | ✓ |
| `validation.yml` | `node scripts/check.mjs` | `node` (显式) | — | ✓ |
| `ci-rust.yml` | Cargo only | Cargo | Rust 源码 | ✓ |
| `quality.yml` | Cargo only | Cargo | Rust 源码 | ✓ |
| `security.yml` | Cargo only | Cargo | Rust 源码 | ✓ |

### 本地测试 (2026-07-21)

```
check-constitution.mjs --quick    4 pass / 0 fail / 1 skip
check-pr-template.mjs             5 pass / 0 fail
worktree.mjs list                 1 worktree (main)
starship-wt.mjs                   "main"
make check-quick                  exit 0
```

### 校验清单

- [x] 全部 `.sh` 文件已删除 (`scripts/` 下无残留)
- [x] CI 工作流脚本引用全部指向 `.mjs`
- [x] 解释器全部为 `node`（shebang 或显式）
- [x] 路径过滤器无 `.sh` 残留
- [x] 14 个引用文件已更新（docs、config、Makefile、nix）
- [x] CONSTITUTION.md §4.8 已更新（移除 shell 例外）
- [x] 本地脚本全部通过测试
