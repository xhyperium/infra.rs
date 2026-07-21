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
| `xhyper-configx` | `crates/configx/` | `configx` | L1 内存字符串 KV（非多源热更新） | [configx-ssot-alignment.md](./configx-ssot-alignment.md) |
| `xhyper-schedulex` | `crates/schedulex/` | `schedulex` | L1 任务 ID 登记表（registry only） | [schedulex-ssot-alignment.md](./schedulex-ssot-alignment.md) |
| `xhyper-bootstrap` | `crates/bootstrap/` | `bootstrap` | L1 唯一组合根（ADR-016） | [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md) |
| `xhyper-resiliencx` | `crates/resiliencx/` | `resiliencx` | L1 重试 | [resiliencx-ssot-alignment.md](./resiliencx-ssot-alignment.md) |
| `xhyper-decimalx` | `crates/types/decimal/` | `decimalx` | `/types/` 十进制 / Money | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `xhyper-canonical` | `crates/types/canonical/` | `canonical` | `/types/` 跨层纯 DTO | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `xhyper-contracts` | `crates/contracts/` | `contracts` | adapter trait 出口（#43） | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `binancex` | `crates/adapters/exchange/binance/` | `binancex` | exchange adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `okxx` | `crates/adapters/exchange/okx/` | `okxx` | exchange adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | `clickhousex` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `kafkax` | `crates/adapters/storage/kafka/` | `kafkax` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `natsx` | `crates/adapters/storage/nats/` | `natsx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `ossx` | `crates/adapters/storage/oss/` | `ossx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `postgresx` | `crates/adapters/storage/postgres/` | `postgresx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `redisx` | `crates/adapters/storage/redis/` | `redisx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `taosx` | `crates/adapters/storage/taos/` | `taosx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `xhyper-transportx` | `crates/transport/` | `transportx` | L1 HTTP/WS 传输 | [transport-ssot-alignment.md](./transport-ssot-alignment.md) |

> **已移除**：`infra-core`（不在 SSOT 三域 kernel/testkit/types 内；文档历史见根 `CHANGELOG` / DDR-003 撤销说明）。

## 依赖图

```text
                         ┌──────────────┐
                         │ xhyper-kernel│  L0
                         └──────▲───────┘
                                │
     ┌───────────┬──────────────┼──────────────┬───────────┐
     │           │              │              │           │
┌────┴────┐ ┌────┴────┐  ┌──────┴─────┐ ┌──────┴─────┐ ┌───┴────┐ ┌───────────┐
│decimalx │ │configx  │  │bootstrap   │ │resiliencx  │ │testkit │ │transportx │
└────▲────┘ └─────────┘  └────────────┘ └────────────┘ └───┬────┘ └───────────┘
     │                                                      dev-only
┌────┴────┐
│canonical│
└─────────┘

  adapters/* (scaffold) ── thiserror（+ contracts/decimalx when implementing）
  xhyper-contracts ── serde + thiserror + decimalx（trait 出口；#43）
  （kernel/types MUST NOT depend on adapters）
```

## 镜像 vs 落地（R7）

| 上游镜像域 | 镜像路径 | 本仓 crate | 状态 |
|------------|----------|------------|------|
| kernel | `.agents/ssot/kernel/` | `crates/kernel` | **已落地**；见 kernel 对齐文 |
| testkit | `.agents/ssot/testkit/` | `crates/testkit` | **core 已落地**；contract-testkit DEFER |
| schedulex | `.agents/ssot/infra/schedulex/` | `crates/schedulex` | **registry 已落地**（active SSOT 最小合同） |
| types | `.agents/ssot/types/` | `crates/types/{decimal,canonical}` | **已落地**；wire/package stable OPEN |
| infra/configx | `.agents/ssot/infra/configx/` | `crates/configx` | **0.1.0 内存 KV 已落地**；多源/热更新 DEFER |
| infra/bootstrap | `.agents/ssot/infra/bootstrap/` | `crates/bootstrap` | **组合根已落地**；contracts/observex/evidence 全量 **DEFER** |
| infra/resiliencx | `.agents/ssot/infra/resiliencx/` | `crates/resiliencx` | **重试已落地**；熔断/限流 DEFER |
| infra 其余域 | `.agents/ssot/infra/*` | — | **仅镜像**，未宣称 crate 落地 |
| adapters | `.agents/ssot/adapters/` | `crates/adapters/**`（9 package） | **镜像已注册**；crate 为 **scaffold**，未宣称实现 |
| （本仓）contracts | —（无独立上游 SSOT 域） | `crates/contracts` | **trait 出口已注册**（#43）；非业务实现 |
| infra/bootstrap | `.agents/ssot/infra/bootstrap/` | `crates/bootstrap` | **组合根已落地**（可移植 trait 替面）；contracts/observex/evidence 全量 **DEFER** |
| infra/transport | `.agents/ssot/infra/transport/` | `crates/transport` | **active 合同已落地**（未达 M3）；见 transport 对齐文 |

规则：

1. 镜像写 COMPLETE / Stable ≠ 本仓可宣称 ship
2. 本仓完成声明必须以 **members + 源码 + 本仓测试输出** 为准
3. 禁止在 `.agents/ssot/**` 镜像内直接编辑；上游用删除感知同步（见 [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md)）
4. 保留上游层级：`infra/`、`adapters/` 勿展平到 `.agents/ssot/` 根

## 验证入口

```bash
cargo metadata --no-deps --format-version 1
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 域专项
cargo test -p xhyper-kernel --all-targets
cargo test -p xhyper-testkit --all-targets
cargo test -p xhyper-configx --all-targets
cargo test -p xhyper-schedulex --all-targets
cargo test -p xhyper-decimalx --all-targets
cargo test -p xhyper-canonical --all-targets
cargo test -p xhyper-bootstrap --all-targets
cargo test -p xhyper-transportx --all-targets
node scripts/check-canonical-align.mjs

# adapters / contracts scaffold
cargo check -p xhyper-contracts -p binancex -p okxx -p redisx -p kafkax \
  -p natsx -p postgresx -p taosx -p ossx -p clickhousex
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/adapters .agents/ssot/adapters
cargo test -p okxx --all-targets
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/contracts .agents/ssot/contracts
```

## 相关索引

| 文档 | 说明 |
|------|------|
| [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) | SPEC-KERNEL-002 本仓矩阵 |
| [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) | SPEC-TESTKIT-002 core 本仓矩阵 |
| [configx-ssot-alignment.md](./configx-ssot-alignment.md) | configx 0.1.0 本仓矩阵 |
| [schedulex-ssot-alignment.md](./schedulex-ssot-alignment.md) | schedulex active registry 本仓矩阵 |
| [types-ssot-alignment.md](./types-ssot-alignment.md) | decimal + canonical 本仓状态 |
| [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md) | bootstrap 组合根本仓矩阵 |
| [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) | adapters 九域镜像 + scaffold 状态 |
| [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) | contracts 镜像 + trait 落地 |
| [transport-ssot-alignment.md](./transport-ssot-alignment.md) | transportx 本仓矩阵 |
| [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md) | 镜像同步完整性（≠ 实现落地） |
| [crates/AGENTS.md](../crates/AGENTS.md) | crate 子模块标准布局 + 概览 |
| [.agents/ssot/SSOT.md](../.agents/ssot/SSOT.md) | R6/R7 规则 |
