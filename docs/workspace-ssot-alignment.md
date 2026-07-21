# Workspace SSOT 对齐总览

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 用途 | 一眼看清：**镜像有什么** vs **本仓落地了什么** |
| 权威 members | 根 `Cargo.toml` `[workspace.members]` + `cargo metadata --no-deps` |

## 当前 workspace members

| Package | 路径 | lib | 角色 | 对齐文档 |
|---------|------|-----|------|----------|
| `xhyper-kernel` | `crates/kernel/` | `kernel` | L0 语义信任根 | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
| `xhyper-testkit` | `crates/testkit/` | `testkit` | T0 test-support（仅 dev-dep） | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `xhyper-decimalx` | `crates/types/decimal/` | `decimalx` | `/types/` 十进制 / Money | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `xhyper-canonical` | `crates/types/canonical/` | `canonical` | `/types/` 跨层纯 DTO | [types-ssot-alignment.md](./types-ssot-alignment.md) |

> **已移除**：`infra-core`（不在 SSOT 三域 kernel/testkit/types 内；文档历史见根 `CHANGELOG` / DDR-003 撤销说明）。

## 依赖图

```text
                    ┌──────────────┐
                    │ xhyper-kernel│  L0
                    └──────▲───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
     ┌────────┴───┐  ┌─────┴─────┐     │
     │xhyper-     │  │xhyper-    │     │
     │decimalx    │  │testkit    │ dev-only
     └────────▲───┘  └───────────┘
              │
     ┌────────┴───┐
     │xhyper-     │
     │canonical   │
     └────────────┘
```

## 镜像 vs 落地（R7）

| 上游镜像域 | 镜像路径 | 本仓 crate | 状态 |
|------------|----------|------------|------|
| kernel | `.agents/ssot/kernel/` | `crates/kernel` | **已落地**；见 kernel 对齐文 |
| testkit | `.agents/ssot/testkit/` | `crates/testkit` | **core 已落地**；contract-testkit DEFER |
| types | `.agents/ssot/types/` | `crates/types/{decimal,canonical}` | **已落地**；wire/package stable OPEN |

规则：

1. 镜像写 COMPLETE / Stable ≠ 本仓可宣称 ship
2. 本仓完成声明必须以 **members + 源码 + 本仓测试输出** 为准
3. 禁止在 `.agents/ssot/**` 镜像内直接编辑；上游用 `cp -rf` 同步（见 [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md)）

## 验证入口

```bash
cargo metadata --no-deps --format-version 1
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 域专项
cargo test -p xhyper-kernel --all-targets
cargo test -p xhyper-testkit --all-targets
cargo test -p xhyper-decimalx --all-targets
cargo test -p xhyper-canonical --all-targets
node scripts/check-canonical-align.mjs
```

## 相关索引

| 文档 | 说明 |
|------|------|
| [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) | SPEC-KERNEL-002 本仓矩阵 |
| [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) | SPEC-TESTKIT-002 core 本仓矩阵 |
| [types-ssot-alignment.md](./types-ssot-alignment.md) | decimal + canonical 本仓状态 |
| [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md) | 镜像同步完整性（≠ 实现落地） |
| [crates/AGENTS.md](../crates/AGENTS.md) | crate 子模块标准布局 + 概览 |
| [.agents/ssot/SSOT.md](../.agents/ssot/SSOT.md) | R6/R7 规则 |
