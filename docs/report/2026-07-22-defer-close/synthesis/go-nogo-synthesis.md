# defer-close 综合裁定 — 13 包 OBJECTIVE DEFER 关闭

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-22 |
| 范围 | 13 个核心 package（见下表）+ workspace 语境（22 crates） |
| 前序 | [2026-07-22 十轮](../../2026-07-22/synthesis/go-nogo-synthesis.md) |
| 性质 | 文档对齐 + 对抗复核；**≠** L5 签核 |
| 证据基线 | 源码路径 + `docs/ssot/*-ssot-alignment.md` OBJECTIVE 表 |

## 1. 总裁定（Go / No-Go）

| 层级 | 裁定 | 说明 |
|------|------|------|
| **Workspace 整体生产发布** | **NO-GO** | **无 L5 人签**；Agent 禁止代签 |
| **13 包声明层 code+test 就绪** | **GO** | OBJECTIVE 非 OOS DEFER 已关闭；路径证据见各对齐文 |
| **量化交易最小可部署面** | **NO-GO** | exchange 业务协议缺失 |
| **Exchange 可交易** | **NO-GO** | 仅 scaffold + 公共 server_time |
| **Storage 生产默认客户端** | **有条件 GO（L1 工程）** | #188–#190；≠ package stable / L5 |
| **archgate** | **OOS-Accept** | 本仓明确不移植 |

**Agent 禁止代签 L5。** 人签模板：`docs/governance/prod-signoff-TEMPLATE.md`。

## 2. 13 包 OBJECTIVE 处置总表

| Package | 关闭前主 DEFER | 现状态 | 路径证据 | Go/No-Go（声明层） |
|---------|----------------|--------|----------|-------------------|
| `kernel` | archgate; drain 在 bootstrap | archgate **OOS-Accept**；drain 归属 bootstrap **PASS** | docs only OOS；`bootstrap/src/drain.rs` | 有条件 GO（库语义） |
| `testkit` | integration harness | **PASS** | `testkit/src/harness.rs` · `IntegrationHarness` | 有条件 GO（仅测试） |
| `decimalx` | wire stable; panicking 公开 | **PASS** | `WIRE_SCHEMA_VERSION`；feature `panicking-ops` default off | 有条件 GO（内部） |
| `canonical` | schema_version envelope | **PASS** | `types/canonical/src/envelope.rs` | 有条件 GO（wire+envelope） |
| `bootstrap` | StoreSet; async drain | **PASS** | `store_set.rs` · `drain.rs` · `with_store_set`/`register_drain` | 声明层 GO；交易装配 **NO-GO** |
| `configx` | 多源/热更新/secret | **PASS（进程内）** | `source`/`layered`/`watch`/`secret` | 声明层 GO；配置中心 **NO** |
| `schedulex` | timer/cron/Job | **PASS（tick）** | `job`/`schedule`/`runner` · `JobRunner::tick` | 进程内 GO；分布式 **NO** |
| `evidence` | remote/sign/query | **PASS** | `query`/`sign`/`remote` | 声明层 GO；合规产品 **OPEN** |
| `observex` | OTEL export/flush | **PASS（in-process）** | `export.rs` · `TelemetryExporter`/`flush` | 进程内 GO；full OTEL **OPEN** |
| `resiliencx` | budget; adapters | **PASS** | `budget.rs`；`redisx`/`postgresx` `resilience.rs` | 有条件 GO |
| `transportx` | TLS/池/代理 | **PASS** | `tls.rs`/`pool.rs`/`proxy.rs` | 有条件 GO |
| `contracts` | Tx/Bus/Repo/Venue live | **PASS（helpers）** | `contracts/src/live.rs` | helpers GO；exchange 业务 **NO-GO** |
| `contract-testkit` | Batch-2; backend profile | **PASS** | `fakes/batch2.rs` · `backend.rs` | 有条件 GO（仅 dev） |

### 开放 DEFER 列表（13 包 OBJECTIVE）

```text
非 OOS OBJECTIVE DEFER = []
唯一 OOS-Accept = kernel archgate
```

诚实 **OPEN 边界**（不是未关 DEFER，而是产品上限）：

| 边界 | 说明 |
|------|------|
| 无 L5 人签 | 禁止 Production Ready 宣称 |
| observex ≠ full OTEL SDK | 进程内 exporter only |
| schedulex ≠ 分布式调度 | tick 驱动 |
| configx ≠ 远端配置中心 | file/env 分层 |
| bootstrap ≠ 交易栈装配完成 | StoreSet 可接线；exchange 无业务 |
| exchange | 签名/下单/WS 仍 NO-GO |

## 3. 与前序 2026-07-22 十轮的关系

| 项 | 前序 | 本 defer-close |
|----|------|----------------|
| 13 包主 DEFER | 开放 | **关闭**（OOS 除外） |
| Workspace 生产发布 | NO-GO | **仍 NO-GO**（无 L5） |
| 量化端到端 | NO-GO | **仍 NO-GO**（exchange） |
| 内部库语义消费 | 有条件 GO | **加厚**（声明层 DEFER 清空） |
| QT-5 配置/调度 | Gap | **Cond**（进程内源/tick；非平台） |
| QT-6 可观测 | Gap/Cond | **Cond**（进程内 export；非 OTEL 产品） |

## 4. 量化场景矩阵（更新）

| 场景 | 综合 | 说明 |
|------|------|------|
| QT-1 行情 | **Gap** | exchange WS/协议缺失 |
| QT-2 下单 | **Gap** | exchange 业务缺失 |
| QT-3 风控 | **Conditional** | decimalx checked；resiliencx budget + redis/pg wire |
| QT-4 持久化/审计 | **Conditional** | storage L1；evidence query/sign/remote 声明层 |
| QT-5 配置/调度 | **Conditional** | configx file/env/watch；schedulex tick（≠ 分布式） |
| QT-6 可观测 | **Conditional** | tracing + 进程内 flush（≠ full OTEL） |
| QT-7 分析 | **Conditional** | CH/TDengine 部分；批量弱 |

## 5. 残留 backlog（非 13 包 OBJECTIVE）

| 优先级 | 项 |
|--------|-----|
| **P0** | 交易所业务协议（签名/下单/WS） |
| **P0** | Kafka offset / NATS JetStream（若产品需要） |
| **P1** | 其余 adapters 统一 resiliencx wire；TLS 强制策略 |
| **P1** | 完整 OTEL/OTLP（若产品需要） |
| **P2** | package stable / **Maintainer L5 人签** |
| **P2** | first-batch 全 trait 深度 conformance |

## 6. 十轮一致性（defer-close）

| 轮次 | 视角 | 是否推翻「13 包 OBJECTIVE 已关」 |
|------|------|--------------------------------|
| R1 inventory | 清单 | 否 — 13 项均有路径 |
| R2 deps | 依赖 | 否 — 无新环依赖 |
| R3 API | 表面 | 否 — 导出可核对 |
| R4 tests | 测试 | 否 — 模块内单测存在 |
| R5 security | 安全 | 否 — secret Debug 脱敏；TLS 配置面 |
| R6 async | 异步 | 否 — drain/tick/watch 语义诚实 |
| R7 integration | 集成 | 否 — StoreSet/live helpers 接线面 |
| R8 docs | 诚实 | 否 — 对齐文已写 OPEN 边界 |
| R9 quant | 量化 | 否 — 交易仍 NO-GO |
| R10 adversarial | 对抗 | 否 — 维持无 L5 / exchange NO-GO |

## 7. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | defer-close 首版：13 包 OBJECTIVE 非 OOS DEFER 清空；workspace 生产仍 NO-GO |
