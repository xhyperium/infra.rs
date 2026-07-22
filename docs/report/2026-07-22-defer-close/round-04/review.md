# Round 04: 测试证据 — defer-close

| 字段 | 值 |
|------|-----|
| 轮次 | 4/10 |
| 视角 | 测试证据 |
| 日期 | 2026-07-22 |
| 范围 | 13 核心 package + workspace 语境 |
| 对齐 | `docs/ssot/*-ssot-alignment.md` |
| 性质 | defer-close 对抗复核；**≠** L5 |

## 1. 审查摘要

模块内单测/集成测是否覆盖关闭项主路径。

**方法：** 扫描新增模块 `#[cfg(test)]` 与 crate tests 引用。

**发现：**

- harness / envelope / drain / store_set / export / budget / live / batch2 / backend 等均含 unit 或 crate 测试。
- 本轮**不**替代 `cargo test --workspace` 全量 CI 门禁记录；对齐文指向可复现命令。
- live 真后端仍 `#[ignore]` + 环境探测（BackendProfile）。

**结论：** 声明层测试锚点存在；L5 人签仍缺。

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
