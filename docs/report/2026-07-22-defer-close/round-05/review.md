# Round 05: 安全边界 — defer-close

| 字段 | 值 |
|------|-----|
| 轮次 | 5/10 |
| 视角 | 安全边界 |
| 日期 | 2026-07-22 |
| 范围 | 13 核心 package + workspace 语境 |
| 对齐 | `docs/ssot/*-ssot-alignment.md` |
| 性质 | defer-close 对抗复核；**≠** L5 |

## 1. 审查摘要

secret 脱敏、TLS 默认、签名/HMAC、禁止日志泄密。

**方法：** 阅读 secret/tls/sign/proxy Debug 实现。

**发现：**
- `SecretString` Debug = `***`。
- transport `ProxyConfig` 密码 Debug 脱敏。
- evidence 签名为 HMAC-SHA256 本地面；**不是** 远程 CA / 不可抵赖合规产品。
- TLS 提供配置面；**未**宣称企业 PKI / mTLS 产品完成。
- 无硬编码生产密钥进入源码（本轮审查范围）。

**结论：** 声明层安全处置 PASS；合规产品 OPEN。

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
