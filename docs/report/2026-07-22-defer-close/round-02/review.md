# Round 02: 依赖图 — defer-close

| 字段 | 值 |
|------|-----|
| 轮次 | 2/10 |
| 视角 | 依赖图 |
| 日期 | 2026-07-22 |
| 范围 | 13 核心 package + workspace 语境 |
| 对齐 | `docs/ssot/*-ssot-alignment.md` |
| 性质 | defer-close 对抗复核；**≠** L5 |

## 1. 审查摘要

检查新增模块是否引入分层违规或环依赖。

**方法：** 阅读各 crate `Cargo.toml` 与模块 `use`。

**发现：**
- configx 仍仅依赖 kernel；source/watch 为 std。
- bootstrap 新增 StoreSet 持 `Arc<dyn …>` 对象安全面，不 re-export 具体 adapter 类型。
- resiliencx 仍禁止依赖 observex；redis/pg 可选消费 resiliencx。
- contract-testkit 仅 dev 路径；contracts 生产图不含 testkit。
- decimal panicking-ops 为 feature，默认关。

**结论：** 依赖纪律保持；无新环。

**硬边界：** 不构成 Maintainer L5；不宣称 workspace Production Ready；不宣称 exchange 可交易。

## 2. 13 包处置快照

| Package | 关闭前 DEFER | 现状态 | 证据路径 |
|---------|--------------|--------|----------|
| kernel | archgate; drain@bootstrap | OOS-Accept; drain PASS@bootstrap | bootstrap/src/drain.rs |
| testkit | IntegrationHarness | PASS | testkit/src/harness.rs |
| decimalx | wire; panicking | PASS | WIRE_SCHEMA_VERSION; panicking-ops off |
| canonical | envelope | PASS | types/canonical/src/envelope.rs |
| bootstrap | StoreSet; AsyncDrain | PASS | store_set.rs; drain.rs |
| configx | multi-source/hot/secret | PASS (in-process) | source/layered/watch/secret |
| schedulex | Job/timer/cron | PASS (tick) | job/schedule/runner |
| evidence | remote/sign/query | PASS | query/sign/remote |
| observex | OTEL export/flush | PASS (in-process) | export.rs |
| resiliencx | budget; adapters | PASS | budget.rs; redis/pg resilience.rs |
| transportx | TLS/pool/proxy | PASS | tls/pool/proxy.rs |
| contracts | Tx/Bus/Repo/Venue live | PASS (helpers) | contracts/src/live.rs |
| contract-testkit | Batch-2; backend | PASS | fakes/batch2.rs; backend.rs |


## 3. 残留

| 项 | 状态 |
|----|------|
| 非 OOS OBJECTIVE DEFER（13 包） | **空** |
| archgate | **OOS-Accept** |
| Agent L5 / 人签 | **未填** |
| exchange 交易 | **NO-GO** |

## 4. 本轮结论

非 OOS DEFER 对 13 包已关闭；生产发布与交易产品仍 NO-GO。
