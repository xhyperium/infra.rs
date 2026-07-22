# Round 07: 量化交易场景 — 模块 Spec 审查

| 字段 | 值 |
|------|-----|
| 轮次 | 7/10 |
| 视角 | 量化交易场景 |
| 日期 | 2026-07-22 |
| 范围 | 22 个 `crates/**` package |
| 判据 | [production-readiness-criteria.md](../production-readiness-criteria.md) |
| 证据源 | `.agents/ssot/**` · `docs/ssot/*-ssot-alignment.md` · `docs/report/2026-07-21/**` · crate `src/`/`tests/` |
| 执行 | Agent Team 并行证据盘点（L0+types+L1 · contracts+adapters）+ 本轮合成 |

## 1. 审查摘要

按 QT-1…QT-7 评估每个 crate 与场景 readiness。

**方法：** 映射行情/下单/风控/持久化/配置调度/可观测/分析。

**本轮结论（一句话）：** 无场景 Ready 全链路；交易执行与可靠消息为 P0 阻塞。

**硬边界：** 本轮**不**构成 Maintainer L5 签核；**不**宣称 workspace Production Ready。

## 2. 本轮聚焦表

| Package | 主 QT | Go | 层 |
|---------|------|------|------|
| `kernel` | QT-6 Cond; 其余 N/A/弱 Cond | 有条件 GO（库语义） | L1+L4 |
| `testkit` | 全 N/A（非 runtime） | 有条件 GO（仅测试） | L1 test-support |
| `decimalx` | QT-3 Ready(checked_*); QT-1/2/7 Cond | 有条件 GO（内部） | L1 |
| `canonical` | QT-1/2/4/7 Cond | 有条件 GO（committed wire） | L2 subset v1–v1.3 |
| `bootstrap` | 横切 Cond | NO-GO 交易装配 | L1 有条件 |
| `configx` | QT-5 Gap/Cond | 合同内 GO / 配置中心 NO | L1 内存合同 |
| `schedulex` | QT-5 Gap | 登记 GO / 调度 NO | L1 registry |
| `evidence` | QT-4 Cond/Gap | 开发默认 GO / 合规 NO | L1 append |
| `observex` | QT-6 Gap/Cond | 最小面 GO / OTEL NO | L1 + L3 Instr 入口 |
| `resiliencx` | QT-3 Cond | 有条件 GO（同步原语） | 接近 L1 Internal |
| `transportx` | QT-1/2 Cond | 有条件 GO | L1 有条件 I/O |
| `contracts` | 接口面 Cond | 子集 GO / first-batch NO | L3 子集 KV+Instr |
| `contract-testkit` | 测资 Ready/Cond | 有条件 GO（仅 dev） | L1 test-support |
| `binancex` | QT-1/2 Gap（仅 time Cond） | NO-GO 交易 | scaffold + server_time |
| `okxx` | QT-1/2 Gap | NO-GO 交易 | scaffold + server_time |
| `redisx` | QT-4 Cond; cache Ready | 有条件 GO（KV） | L1 + L3-KV 入口 |
| `postgresx` | QT-4 Cond | 有条件 GO（SQL） | L1 池+Tx |
| `kafkax` | QT-4 Gap(offset) | 有条件 / EOS NO | L1 AMO EventBus |
| `natsx` | QT-4 Gap(JetStream) | Core GO / JetStream NO | L1 Core NATS |
| `ossx` | QT-4 Cond | 有条件 GO | L1 ObjectStore |
| `clickhousex` | QT-7 Gap/Cond | 部分 / 批量 NO | L1 HTTP 部分 |
| `taosx` | QT-7 Cond | 部分 | L1 REST 部分 |

## 3. 逐 crate 分析

### `kernel`

| 项 | 值 |
|----|-----|
| 路径 | `crates/kernel` |
| 平面 | L0 |
| SSOT | `.agents/ssot/kernel/` |
| 对齐 | `docs/ssot/kernel-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/5/5/4（Σ=34/35） |
| 生产层 | L1+L4 |
| 补齐需求 | 低 |
| Go/No-Go | 有条件 GO（库语义） |
| 量化 | QT-6 Cond; 其余 N/A/弱 Cond |
| 主 DEFER | archgate OOS; 组合根 drain 在 bootstrap |


### `testkit`

| 项 | 值 |
|----|-----|
| 路径 | `crates/testkit` |
| 平面 | T0 |
| SSOT | `.agents/ssot/testkit/` |
| 对齐 | `docs/ssot/testkit-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/5/5/3（Σ=33/35） |
| 生产层 | L1 test-support |
| 补齐需求 | 低 |
| Go/No-Go | 有条件 GO（仅测试） |
| 量化 | 全 N/A（非 runtime） |
| 主 DEFER | integration harness DEFER |


### `decimalx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/types/decimal` |
| 平面 | types |
| SSOT | `.agents/ssot/types/decimal/` |
| 对齐 | `docs/ssot/types-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/5/5/3（Σ=33/35） |
| 生产层 | L1 |
| 补齐需求 | 低（纪律） |
| Go/No-Go | 有条件 GO（内部） |
| 量化 | QT-3 Ready(checked_*); QT-1/2/7 Cond |
| 主 DEFER | wire 跨版本 stable; panicking 算子仍公开 |


### `canonical`

| 项 | 值 |
|----|-----|
| 路径 | `crates/types/canonical` |
| 平面 | types |
| SSOT | `.agents/ssot/types/canonical/` |
| 对齐 | `docs/ssot/types-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/5/5/3（Σ=33/35） |
| 生产层 | L2 subset v1–v1.3 |
| 补齐需求 | 中（envelope） |
| Go/No-Go | 有条件 GO（committed wire） |
| 量化 | QT-1/2/4/7 Cond |
| 主 DEFER | 无 schema_version envelope |


### `bootstrap`

| 项 | 值 |
|----|-----|
| 路径 | `crates/bootstrap` |
| 平面 | L1 |
| SSOT | `.agents/ssot/bootstrap/` |
| 对齐 | `docs/ssot/bootstrap-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/3（Σ=32/35） |
| 生产层 | L1 有条件 |
| 补齐需求 | 高（接线） |
| Go/No-Go | NO-GO 交易装配 |
| 量化 | 横切 Cond |
| 主 DEFER | StoreSet/adapter 未接线; async drain DEFER |


### `configx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/configx` |
| 平面 | L1 |
| SSOT | `.agents/ssot/configx/` |
| 对齐 | `docs/ssot/configx-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/3（Σ=32/35） |
| 生产层 | L1 内存合同 |
| 补齐需求 | 高（生产配置） |
| Go/No-Go | 合同内 GO / 配置中心 NO |
| 量化 | QT-5 Gap/Cond |
| 主 DEFER | 多源/热更新/secret DEFER |


### `schedulex`

| 项 | 值 |
|----|-----|
| 路径 | `crates/schedulex` |
| 平面 | L1 |
| SSOT | `.agents/ssot/schedulex/` |
| 对齐 | `docs/ssot/schedulex-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/3（Σ=32/35） |
| 生产层 | L1 registry |
| 补齐需求 | 高（名实） |
| Go/No-Go | 登记 GO / 调度 NO |
| 量化 | QT-5 Gap |
| 主 DEFER | 无 timer/cron/Job 执行 |


### `evidence`

| 项 | 值 |
|----|-----|
| 路径 | `crates/evidence` |
| 平面 | L1 |
| SSOT | `.agents/ssot/tools/evidence/` |
| 对齐 | `docs/ssot/evidence-ssot-alignment.md` |
| Spec S1–S7 | 5/4/3/4/3/4/2（Σ=25/35） |
| 生产层 | L1 append |
| 补齐需求 | 中 |
| Go/No-Go | 开发默认 GO / 合规 NO |
| 量化 | QT-4 Cond/Gap |
| 主 DEFER | 远程/签名 wire; 查询 API |


### `observex`

| 项 | 值 |
|----|-----|
| 路径 | `crates/observex` |
| 平面 | L1 |
| SSOT | `.agents/ssot/observex/` |
| 对齐 | `docs/ssot/observex-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/3（Σ=32/35） |
| 生产层 | L1 + L3 Instr 入口 |
| 补齐需求 | 高（OTEL） |
| Go/No-Go | 最小面 GO / OTEL NO |
| 量化 | QT-6 Gap/Cond |
| 主 DEFER | OTEL exporter/flush DEFER |


### `resiliencx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/resiliencx` |
| 平面 | L1 |
| SSOT | `.agents/ssot/resiliencx/` |
| 对齐 | `docs/ssot/resiliencx-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/4/4/5/3（Σ=31/35） |
| 生产层 | 接近 L1 Internal |
| 补齐需求 | 中（集成） |
| Go/No-Go | 有条件 GO（同步原语） |
| 量化 | QT-3 Cond |
| 主 DEFER | budget; 未接入 adapters |


### `transportx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/transport` |
| 平面 | L1 |
| SSOT | `.agents/ssot/transport/` |
| 对齐 | `docs/ssot/transport-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/3（Σ=32/35） |
| 生产层 | L1 有条件 I/O |
| 补齐需求 | 中（TLS 矩阵） |
| Go/No-Go | 有条件 GO |
| 量化 | QT-1/2 Cond |
| 主 DEFER | TLS 矩阵/池/代理 DEFER |


### `contracts`

| 项 | 值 |
|----|-----|
| 路径 | `crates/contracts` |
| 平面 | contracts |
| SSOT | `.agents/ssot/contracts/` |
| 对齐 | `docs/ssot/contracts-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/4（Σ=33/35） |
| 生产层 | L3 子集 KV+Instr |
| 补齐需求 | 高（扩面） |
| Go/No-Go | 子集 GO / first-batch NO |
| 量化 | 接口面 Cond |
| 主 DEFER | Tx/Bus/Repo/Venue 业务 live |


### `contract-testkit`

| 项 | 值 |
|----|-----|
| 路径 | `crates/test-support/contracts` |
| 平面 | T0 |
| SSOT | `.agents/ssot/testkit/ §3.2 + contracts` |
| 对齐 | `docs/ssot/testkit-ssot-alignment.md` |
| Spec S1–S7 | 5/4/4/3/5/4/4（Σ=29/35） |
| 生产层 | L1 test-support |
| 补齐需求 | 中（扩 suite） |
| Go/No-Go | 有条件 GO（仅 dev） |
| 量化 | 测资 Ready/Cond |
| 主 DEFER | Batch-2 Fake; 真后端 profile |


### `binancex`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/exchange/binance` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/exchange/binance/` |
| 对齐 | `docs/ssot/adapters-ssot-alignment.md` |
| Spec S1–S7 | 4/4/3/3/3/4/3（Σ=24/35） |
| 生产层 | scaffold + server_time |
| 补齐需求 | 极高 |
| Go/No-Go | NO-GO 交易 |
| 量化 | QT-1/2 Gap（仅 time Cond） |
| 主 DEFER | 签名/下单/WS 行情 |


### `okxx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/exchange/okx` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/exchange/okx/` |
| 对齐 | `docs/ssot/adapters-ssot-alignment.md` |
| Spec S1–S7 | 4/4/3/3/3/4/3（Σ=24/35） |
| 生产层 | scaffold + server_time |
| 补齐需求 | 极高 |
| Go/No-Go | NO-GO 交易 |
| 量化 | QT-1/2 Gap |
| 主 DEFER | 签名/业务协议 |


### `redisx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/redis` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/redis/` |
| 对齐 | `docs/ssot/redisx-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/4（Σ=33/35） |
| 生产层 | L1 + L3-KV 入口 |
| 补齐需求 | 中 |
| Go/No-Go | 有条件 GO（KV） |
| 量化 | QT-4 Cond; cache Ready |
| 主 DEFER | Cluster/Sentinel; TLS 强制; resiliencx |


### `postgresx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/postgres` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/postgres/` |
| 对齐 | `docs/ssot/postgresx-ssot-alignment.md` |
| Spec S1–S7 | 5/5/5/5/4/5/4（Σ=33/35） |
| 生产层 | L1 池+Tx |
| 补齐需求 | 中 |
| Go/No-Go | 有条件 GO（SQL） |
| 量化 | QT-4 Cond |
| 主 DEFER | prod Repository; SSL require; resiliencx |


### `kafkax`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/kafka` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 对齐 | `docs/ssot/kafkax-ssot-alignment.md` |
| Spec S1–S7 | 5/5/4/4/4/5/4（Σ=31/35） |
| 生产层 | L1 AMO EventBus |
| 补齐需求 | 高 |
| Go/No-Go | 有条件 / EOS NO |
| 量化 | QT-4 Gap(offset) |
| 主 DEFER | offset commit; at-least-once; EOS |


### `natsx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/nats` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 对齐 | `docs/ssot/natsx-ssot-alignment.md` |
| Spec S1–S7 | 5/5/4/4/4/5/4（Σ=31/35） |
| 生产层 | L1 Core NATS |
| 补齐需求 | 高 |
| Go/No-Go | Core GO / JetStream NO |
| 量化 | QT-4 Gap(JetStream) |
| 主 DEFER | JetStream; TLS 默认 |


### `ossx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/oss` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/oss/` |
| 对齐 | `docs/ssot/ossx-ssot-alignment.md` |
| Spec S1–S7 | 4/5/4/4/4/4/3（Σ=28/35） |
| 生产层 | L1 ObjectStore |
| 补齐需求 | 中 |
| Go/No-Go | 有条件 GO |
| 量化 | QT-4 Cond |
| 主 DEFER | multipart; retry |


### `clickhousex`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/clickhouse` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 对齐 | `docs/ssot/clickhousex-ssot-alignment.md` |
| Spec S1–S7 | 4/5/4/4/3/4/3（Σ=27/35） |
| 生产层 | L1 HTTP 部分 |
| 补齐需求 | 高 |
| Go/No-Go | 部分 / 批量 NO |
| 量化 | QT-7 Gap/Cond |
| 主 DEFER | 批量 insert; 池强度 |


### `taosx`

| 项 | 值 |
|----|-----|
| 路径 | `crates/adapters/storage/taos` |
| 平面 | adapter |
| SSOT | `.agents/ssot/adapters/storage/taos/` |
| 对齐 | `docs/ssot/taosx-ssot-alignment.md` |
| Spec S1–S7 | 4/5/4/4/3/4/3（Σ=27/35） |
| 生产层 | L1 REST 部分 |
| 补齐需求 | 高 |
| Go/No-Go | 部分 |
| 量化 | QT-7 Cond |
| 主 DEFER | 批量写; native; 池 |




## 4. 跨 crate 观察

- **规格 vs 实现**：S1–S6 普遍 ≥4；生产层缺口主要在 **实现 DEFER** 与 **集成接线**，而非「无 SSOT」。
- **与 2026-07-21 报告关系**：继承 `status-modules-production-readiness.md` 整体 **否** 的结论；storage 默认客户端以 #188–#190 对齐文为准，**修正**纯 scaffold 过时 partial。
- **exchange**：镜像 SSOT COMPLETE **≠** 本仓可交易。
- **evidence**：SSOT 在 tools 平面，对齐文档偏薄（S2/S3/S7 弱）。

## 5. 轮次结论

| 问题 | 裁定 |
|------|------|
| Spec 是否完整？ | **多数完整**（Σ≥28）；evidence / exchange 业务规格落地弱 |
| 是否需要补齐？ | **是** — 见 R9 backlog；补齐重点是**实现与接线**，非重写 SSOT |
| 能否生产发布？ | **workspace NO-GO**；局部内部 GO 见各 crate |
| 量化适用？ | **无端到端 Ready**；缓存/SQL/类型层 Conditional |

### 5.1 本轮增量发现

- R7 视角「量化交易场景」下，未发现足以推翻「整体 NO-GO」的新证据。
- 局部内部就绪（kernel/types/redis KV 等）与 2026-07-21 一致。

## 6. 引用路径（抽样可复核）

- `docs/ssot/kernel-ssot-alignment.md`
- `docs/ssot/contracts-ssot-alignment.md`
- `docs/ssot/adapters-ssot-alignment.md`
- `crates/contracts/docs/L3_FIRST_BATCH_STATUS.md`
- `docs/report/2026-07-21/status-modules-production-readiness.md`
