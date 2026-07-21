# Workspace SSOT 对齐总览

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| 跟进 | P0/P1 **#98**；PLAN-CORE-PROD-002 W0–W5 + L5 **0.3.0-signoff**；四包内部 GO **#159** · tag **`v0.3.0-four-crates`**；**不**宣称 workspace 整体 Production Ready |
| 用途 | 一眼看清：**镜像有什么** vs **本仓落地了什么** |
| 权威 members | 根 `Cargo.toml` `[workspace.members]` + `cargo metadata --no-deps`（**package 名以 metadata 为准**） |

## 当前 workspace members

| Package（`cargo -p`） | 路径 | lib | 角色 | 对齐文档 |
|---------|------|-----|------|----------|
| `kernel` | `crates/kernel/` | `kernel` | L0 语义信任根 · **L1+L4** | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
| `testkit` | `crates/testkit/` | `testkit` | T0 test-support（仅 dev-dep）· **L1** | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `configx` | `crates/configx/` | `configx` | L1 内存字符串 KV（非多源热更新） | [configx-ssot-alignment.md](./configx-ssot-alignment.md) |
| `schedulex` | `crates/schedulex/` | `schedulex` | L1 任务 ID 登记表（registry only） | [schedulex-ssot-alignment.md](./schedulex-ssot-alignment.md) |
| `bootstrap` | `crates/bootstrap/` | `bootstrap` | L1 唯一组合根（ADR-016） | [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md) |
| `evidence` | `crates/evidence/` | `evidence` | L1 审计证据追加面 | [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) |
| `resiliencx` | `crates/resiliencx/` | `resiliencx` | L1 重试 + 熔断 + 限流 | [resiliencx-ssot-alignment.md](./resiliencx-ssot-alignment.md) |
| `decimalx` | `crates/types/decimal/` | `decimalx` | `/types/` 十进制 / Money · **L1** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `canonical` | `crates/types/canonical/` | `canonical` | `/types/` 跨层纯 DTO · **L2 wire 子集** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `contracts` | `crates/contracts/` | `contracts` | adapter trait 出口（#43）；Tx/消息可测 | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| `binancex` | `crates/adapters/exchange/binance/` | `binancex` | exchange adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `okxx` | `crates/adapters/exchange/okx/` | `okxx` | exchange adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | `clickhousex` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `kafkax` | `crates/adapters/storage/kafka/` | `kafkax` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `natsx` | `crates/adapters/storage/nats/` | `natsx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `ossx` | `crates/adapters/storage/oss/` | `ossx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `postgresx` | `crates/adapters/storage/postgres/` | `postgresx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `redisx` | `crates/adapters/storage/redis/` | `redisx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `taosx` | `crates/adapters/storage/taos/` | `taosx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `transportx` | `crates/transport/` | `transportx` | L1 HTTP/WS 传输 | [transport-ssot-alignment.md](./transport-ssot-alignment.md) |

> **已移除**：`infra-core`（不在 SSOT 三域 kernel/testkit/types 内；文档历史见根 `CHANGELOG` / DDR-003 撤销说明）。

## 依赖图

```text
                         ┌──────────────┐
                         │    kernel    │  L0 · L1+L4 internal GO
                         └──────▲───────┘
                                │
     ┌───────────┬──────────────┼──────────────┬───────────┐
     │           │              │              │           │
┌────┴────┐ ┌────┴────┐  ┌──────┴─────┐ ┌──────┴─────┐ ┌───┴────┐ ┌───────────┐
│decimalx │ │configx  │  │bootstrap   │ │resiliencx  │ │testkit │ │transportx │
│  L1     │ └─────────┘  └────────────┘ └────────────┘ │  L1    │ └───────────┘
└────▲────┘                                            │dev-only│
     │                                                 └────────┘
┌────┴────┐
│canonical│  L2 committed wire subset
└─────────┘

  adapters/* (scaffold) ── thiserror（+ contracts/decimalx when implementing）
  contracts ── serde + thiserror + decimalx（trait 出口；#43）
  （kernel/types MUST NOT depend on adapters）
```

## 镜像 vs 落地（R7）

| 上游镜像域 | 镜像路径 | 本仓 crate | 状态 |
|------------|----------|------------|------|
| kernel | `.agents/ssot/kernel/` | `crates/kernel` | **已落地**；ClockDomain + loom CI + wait_timeout；见 kernel 对齐文 |
| testkit | `.agents/ssot/testkit/` | `crates/testkit` | **ManualClock core 已落地**（含 domain）；独立 contract-testkit crate **DEFER** |
| schedulex | `.agents/ssot/schedulex/` | `crates/schedulex` | **registry 已落地**（active SSOT 最小合同） |
| types | `.agents/ssot/types/` | `crates/types/{decimal,canonical}` | **已落地**；decimal **L1**；canonical **L2** committed v1–v1.3；package stable **OPEN** |
| configx | `.agents/ssot/configx/` | `crates/configx` | **0.1.0 内存 KV 已落地**；多源/热更新 DEFER |
| bootstrap | `.agents/ssot/bootstrap/` | `crates/bootstrap` | **组合根已落地**；`Bounded*` 有界面 + Instrumentation/Evidence 注入；全量 async contracts **DEFER** |
| resiliencx | `.agents/ssot/resiliencx/` | `crates/resiliencx` | **重试已落地**；熔断/限流 DEFER |
| infra 其余域 | `.agents/ssot/{gate,observex,testkitx,transport}` | — | **仅镜像** 或见分域对齐文；勿把镜像 COMPLETE 当本仓 ship |
| adapters | `.agents/ssot/adapters/` | `crates/adapters/**`（9 package） | **镜像已注册**；crate 为 **scaffold**，未宣称业务实现 |
| （本仓）contracts | `.agents/ssot/contracts/`（若有） | `crates/contracts` | **trait 出口已注册**；Tx/消息可测 + 最小 Fake testkit；真实后端 **DEFER** |
| transport | `.agents/ssot/transport/` | `crates/transport` | **active 合同已落地**（未达 M3）；见 transport 对齐文 |
| tools | `.agents/ssot/tools/` | `crates/evidence`（仅 evidence） | **镜像已本地化**；evidence 最小面落地；goalctl/xtask/verifyctl **未**落地 |

规则：

1. 镜像写 COMPLETE / Stable ≠ 本仓可宣称 ship
2. 本仓完成声明必须以 **members + 源码 + 本仓测试输出** 为准
3. 禁止在 `.agents/ssot/**` 镜像内直接编辑；上游用删除感知同步（见 [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md)）
4. 保留上游层级：`adapters/`、`tools/` 勿展平到 `.agents/ssot/` 根（`infra/` 已展平）

## 验证入口

```bash
cargo metadata --no-deps --format-version 1
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 域专项（package 名 = cargo metadata name）
cargo test -p kernel --all-targets
cargo test -p testkit --all-targets
cargo test -p configx --all-targets
cargo test -p schedulex --all-targets
cargo test -p decimalx --all-targets
cargo test -p canonical --all-targets
cargo test -p contracts --all-targets
cargo test -p bootstrap --all-targets
cargo test -p transportx --all-targets
node scripts/quality-gates/check-canonical-align.mjs
node scripts/quality-gates/check-public-api.mjs
# kernel loom 持续门禁
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
# 或: node scripts/quality-gates/run-kernel-loom.mjs

# adapters / contracts scaffold
cargo check -p contracts -p binancex -p okxx -p redisx -p kafkax \
  -p natsx -p postgresx -p taosx -p ossx -p clickhousex
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/adapters .agents/ssot/adapters
cargo test -p okxx --all-targets
diff -rq /home/workspace/xhyper.rs/.agent/SSOT/contracts .agents/ssot/contracts

# tools（本地化后与上游非零 diff 为预期；校验目录存在）
test -d .agents/ssot/tools/evidence
test -d .agents/ssot/tools/goalctl
test -d .agents/ssot/tools/xtask
test -d .agents/ssot/tools/verifyctl
cargo test -p evidence --all-targets
```

## 核心 crate 生产就绪快照（2026-07-21 · 更新）

| crate | 本仓判定（分层） | 权威细节 |
|-------|------------------|----------|
| `kernel` | **L1 + L4 内部 GO**（#159） | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
| `testkit` | **L1 ManualClock test-support**（非 runtime） | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `decimalx` | **L1 Internal Ready** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `canonical` | **L2 committed wire subset**（v1–v1.3） | 同上 |
| `contracts` | **部分就绪**（不在四包 GO 范围） | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |

- 四包证据：[`../plans/releases/2026-07-21-four-crates-internal-release.md`](../plans/releases/2026-07-21-four-crates-internal-release.md)
- L5 权威：[`../plans/releases/0.3.0-signoff.md`](../plans/releases/0.3.0-signoff.md) · GO-with-Accepts
- 内部 tag：`v0.3.0-four-crates`
- 审计与 DEFER：[`../report/2026-07-21/core-crates-production-readiness.md`](../report/2026-07-21/core-crates-production-readiness.md) §11  

**禁止**将上表误读为「五 crate / workspace 整体 Production Ready」或 crates.io 已发布。

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
| [tools-ssot-alignment.md](./tools-ssot-alignment.md) | tools 四域镜像 + 本地化状态 |
| [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) | evidence crate 落地矩阵 |
| [SSOT_SYNC_REPORT.md](./SSOT_SYNC_REPORT.md) | 镜像同步完整性（≠ 实现落地） |
| [crates/AGENTS.md](../../crates/AGENTS.md) | crate 子模块标准布局 + 概览 |
| [.agents/ssot/SSOT.md](../../.agents/ssot/SSOT.md) | R6/R7 规则 |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | PR #98 合入：五 crate 生产就绪快照与验证入口（loom/align 路径）写入总览 |
| 2026-07-21 | 四包内部 GO：members 表 package 名对齐 Cargo metadata；分层 L1/L2/L4；#159 · tag `v0.3.0-four-crates` |
