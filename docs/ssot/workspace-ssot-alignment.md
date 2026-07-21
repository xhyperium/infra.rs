# Workspace SSOT 对齐总览

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-22 |
| 跟进 | P0/P1 **#98**；L5 **0.3.0-signoff**；四包 GO **#159** · **`v0.3.0-four-crates`**；kernel **#163**；**`infra-s9t` 18/18 closed**（#166–#168 · #172）· 对齐 **#174** · closeout **#175**；**contract-testkit #178**；**不**宣称 workspace Production Ready / L5 |
| 用途 | 一眼看清：**镜像有什么** vs **本仓落地了什么** |
| 权威 members | 根 `Cargo.toml` `[workspace.members]` + `cargo metadata --no-deps`（**package 名以 metadata 为准**） |

## 当前 workspace members

| Package（`cargo -p`） | 路径 | lib | 角色 | 对齐文档 |
|---------|------|-----|------|----------|
| `kernel` | `crates/kernel/` | `kernel` | L0 语义信任根 · **L1+L4 已内部发布** | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) |
| `testkit` | `crates/testkit/` | `testkit` | T0 test-support（仅 dev-dep）· **L1** ManualClock | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `contract-testkit` | `crates/test-support/contracts/` | `contract_testkit` | T0 Fake + per-trait suite（仅 dev-dep）· #178 | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) · [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| `configx` | `crates/configx/` | `configx` | L1 内存字符串 KV（非多源热更新） | [configx-ssot-alignment.md](./configx-ssot-alignment.md) |
| `schedulex` | `crates/schedulex/` | `schedulex` | L1 任务 ID 登记表（registry only） | [schedulex-ssot-alignment.md](./schedulex-ssot-alignment.md) |
| `bootstrap` | `crates/bootstrap/` | `bootstrap` | L1 唯一组合根（ADR-016） | [bootstrap-ssot-alignment.md](./bootstrap-ssot-alignment.md) |
| `evidence` | `crates/evidence/` | `evidence` | L1 审计证据追加面 | [evidence-ssot-alignment.md](./evidence-ssot-alignment.md) |
| `observex` | `crates/observex/` | `observex` | L1 TracingInstrumentation（L3 Instr 入口） | [observex-ssot-alignment.md](./observex-ssot-alignment.md) |
| `resiliencx` | `crates/resiliencx/` | `resiliencx` | L1 重试（含 async）+ 熔断 + 限流 + 舱壁 | [resiliencx-ssot-alignment.md](./resiliencx-ssot-alignment.md) |
| `decimalx` | `crates/types/decimal/` | `decimalx` | `/types/` 十进制 / Money · **L1** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `canonical` | `crates/types/canonical/` | `canonical` | `/types/` 跨层纯 DTO · **L2 wire 子集** | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `contracts` | `crates/contracts/` | `contracts` | adapter trait 出口；L3 子集（KV+Instr） | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| `binancex` | `crates/adapters/exchange/binance/` | `binancex` | exchange scaffold + mock HTTP + `server_time` 解析 | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `okxx` | `crates/adapters/exchange/okx/` | `okxx` | exchange scaffold + mock HTTP + `server_time` 解析 | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `clickhousex` | `crates/adapters/storage/clickhouse/` | `clickhousex` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `kafkax` | `crates/adapters/storage/kafka/` | `kafkax` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `natsx` | `crates/adapters/storage/nats/` | `natsx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `ossx` | `crates/adapters/storage/oss/` | `ossx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `postgresx` | `crates/adapters/storage/postgres/` | `postgresx` | storage adapter scaffold | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| `redisx` | `crates/adapters/storage/redis/` | `redisx` | storage scaffold + `live` `RedisLiveKv`（CT-9） | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
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
  contract-testkit ── contracts（**仅 dev-dep**；Fake/suite；禁止 production graph）
  （kernel/types MUST NOT depend on adapters）
```

## 镜像 vs 落地（R7）

| 上游镜像域 | 镜像路径 | 本仓 crate | 状态 |
|------------|----------|------------|------|
| kernel | `.agents/ssot/kernel/` | `crates/kernel` | **已落地**；ClockDomain + loom CI + wait_timeout；见 kernel 对齐文 |
| testkit | `.agents/ssot/testkit/` | `crates/testkit` | **ManualClock core 已落地**（含 domain） |
| contract-testkit | `.agents/ssot/testkit/` §3.2 | `crates/test-support/contracts` | **已落地**（Fake + per-trait suite；仅 dev-dep） |
| schedulex | `.agents/ssot/schedulex/` | `crates/schedulex` | **registry 已落地**（active SSOT 最小合同） |
| types | `.agents/ssot/types/` | `crates/types/{decimal,canonical}` | **已落地**；decimal **L1**；canonical **L2** committed v1–v1.3；package stable **OPEN** |
| configx | `.agents/ssot/configx/` | `crates/configx` | **0.1.0 内存 KV 已落地**；多源/热更新 DEFER |
| bootstrap | `.agents/ssot/bootstrap/` | `crates/bootstrap` | **组合根已落地**；`Bounded*` + Instrumentation/Evidence；`require_evidence` **release fail-closed**（#168）；全量 async contracts **DEFER** |
| resiliencx | `.agents/ssot/resiliencx/` | `crates/resiliencx` | **重试 + 熔断 + 限流 + 舱壁 + `retry_async`/`AsyncWait`**（#167）；budget/stable **DEFER** |
| observex | `.agents/ssot/observex/` | `crates/observex` | **TracingInstrumentation 最小面**；OTEL 导出 **DEFER** |
| infra 其余域 | `.agents/ssot/{gate,testkitx}` | — | **仅镜像**；勿把镜像 COMPLETE 当本仓 ship |
| adapters | `.agents/ssot/adapters/` | `crates/adapters/**`（9 package） | **镜像已注册**；多数 scaffold；**redisx live KV** + exchange **只读 server_time** 入口（#168/#172）；**非**业务 Production Ready |
| （本仓）contracts | `.agents/ssot/contracts/`（若有） | `crates/contracts` | **trait 出口**；Fake/suite 在 `contract-testkit`；**L3 子集** KV+Instr（#172）；Tx/Bus/Repo/Venue 业务 live **DEFER** |
| transport | `.agents/ssot/transport/` | `crates/transport` | **active 合同已落地**（含 P0 硬化 #166）；未达 M3 |
| tools | `.agents/ssot/tools/` | `crates/evidence`（仅 evidence） | evidence 含 `FileEvidenceAppender`；goalctl/xtask/verifyctl **未**落地 |

规则：

1. 规格写 COMPLETE / Stable ≠ 本仓可宣称 ship
2. 本仓完成声明必须以 **members + 源码 + 本仓测试输出** 为准
3. `.agents/ssot/**` 为本仓域规格 SSOT（见 `.agents/ssot/SSOT.md` R6）；变更走 worktree + PR。从外仓镜像同步用删除感知 rsync（见 [SSOT_SYNC_OPS.md](./SSOT_SYNC_OPS.md)），**禁止**用上游覆盖冲掉本仓 OOS/落地裁定
4. 保留层级：`adapters/`、`tools/` 勿展平到 `.agents/ssot/` 根（`infra/` 已展平）
5. **archgate / `.architecture`：OOS**（#164）— 本仓明确不移植；机控用结构扫描 / CI / public-api 等

## 验证入口

```bash
cargo metadata --no-deps --format-version 1
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 域专项（package 名 = cargo metadata name）
cargo test -p kernel --all-targets
cargo test -p testkit --all-targets
cargo test -p contract-testkit --all-targets
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

# adapters / contracts
cargo check -p contracts -p binancex -p okxx -p redisx -p kafkax \
  -p natsx -p postgresx -p taosx -p ossx -p clickhousex
cargo test -p contracts -p binancex -p okxx -p redisx --all-targets
# optional live（需服务 / 外网；默认 ignore）
# REDIS_URL=redis://127.0.0.1:6379 cargo test -p redisx --features live -- --ignored
# cargo test -p binancex -p okxx --test live_server_time -- --ignored

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
| `kernel` | **L1 + L4 已内部发布**（#159 · #163 · GH Release） | [kernel-ssot-alignment.md](./kernel-ssot-alignment.md) · [0.3.0-internal](../../crates/kernel/releases/0.3.0-internal.md) |
| `testkit` | **L1 ManualClock test-support**（非 runtime；四包 GO 内） | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `contract-testkit` | **已落地** Fake + suite（仅 dev-dep；#178） | [testkit-ssot-alignment.md](./testkit-ssot-alignment.md) |
| `decimalx` | **L1 Internal Ready**（四包 GO 内） | [types-ssot-alignment.md](./types-ssot-alignment.md) |
| `canonical` | **L2 committed wire subset**（v1–v1.3；四包 GO 内） | 同上 |
| `contracts` | **部分就绪**：L3 子集（KV+Instr）；非 first-batch 全绿 | [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) · [L3_FIRST_BATCH_STATUS](../../crates/contracts/docs/L3_FIRST_BATCH_STATUS.md) |
| `redisx`（live） | **非 scaffold KV 入口**（feature `live`；optional CI） | [adapters-ssot-alignment.md](./adapters-ssot-alignment.md) |
| L1 平台（bootstrap/resiliencx/transport/…） | **P0 阻断已收敛**（`infra-s9t`）；≠ 各包 Production Ready | [七包双栏](../report/2026-07-21/seven-l1-contracts-dual-bar-readiness.md) · [prod-consume-surface](../plans/artifacts/prod-consume-surface.md) |

- 四包证据：[`../plans/releases/2026-07-21-four-crates-internal-release.md`](../plans/releases/2026-07-21-four-crates-internal-release.md)
- kernel crate 发布记录：[`../../crates/kernel/releases/0.3.0-internal.md`](../../crates/kernel/releases/0.3.0-internal.md)
- L5 权威：[`../plans/releases/0.3.0-signoff.md`](../plans/releases/0.3.0-signoff.md) · GO-with-Accepts
- 内部 tag / GH Release：[`v0.3.0-four-crates`](https://github.com/xhyperium/infra.rs/releases/tag/v0.3.0-four-crates)
- 审计与 DEFER：[`../report/2026-07-21/core-crates-production-readiness.md`](../report/2026-07-21/core-crates-production-readiness.md) §11  
- STATUS-PROD 行动树：[`../plans/2026-07-21-status-modules-prod-followup.md`](../plans/2026-07-21-status-modules-prod-followup.md)（epic **`infra-s9t` closed**）

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
| 2026-07-21 | kernel **内部发布已执行**：#163 · `crates/kernel/releases/0.3.0-internal.md` · GH Release；对齐快照更新 |
| 2026-07-21 | **infra-s9t** 闭合（#166–#168 · #172）：L1 P0、redis live KV、contracts L3 子集、exchange `server_time`；总览与分域对齐同步 |
| 2026-07-21 | 对齐/同步文档刷新 #174；follow-up CLOSED + report partials closeout #175；本文件补引用 |
| 2026-07-22 | **#178** 独立 `contract-testkit` 落地；members 表补行；Fake 迁出 contracts；SSOT 同步报告纠偏 |
