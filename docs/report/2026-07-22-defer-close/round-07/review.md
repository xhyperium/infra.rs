# Round 07: 集成接线 — defer-close

| 字段 | 值 |
|------|-----|
| 轮次 | 7/10 |
| 视角 | 集成接线 |
| 日期 | 2026-07-22 |
| 范围 | 13 核心 package + workspace 语境 |
| 对齐 | `docs/ssot/*-ssot-alignment.md` |
| 性质 | defer-close 对抗复核；**≠** L5 |

## 1. 审查摘要

StoreSet、live helpers、adapter resilience 是否构成可组合面。

**方法：** bootstrap StoreSet + contracts live + redis/pg resilience 交叉阅读。

**发现：**
- StoreSet 可注入 KV/Tx/Bus/Repo/Venue 句柄；**不能**弥补 exchange 业务缺失。
- LiveContractProfile 提供 first_batch_kv / storage_stack / venue_stack 配置面。
- redisx/postgresx 有 resilience 模块挂接 resiliencx。
- 端到端「交易可部署」仍被 exchange NO-GO 阻断。

**结论：** 集成接线 OBJECTIVE 关闭；交易产品仍 NO-GO。

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
