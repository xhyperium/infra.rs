# orderbook 通用订单簿内核规格

**版本**：0.1.0
**状态**：目标契约已指定；当前无对应 runtime crate，所有 runtime 门禁均为 pending/deferred
**来源**：`.cargo/draft/orderbook.md`、`.cargo/draft/orderbook/1.md`
**公共类型 owner**：`crates/domain_market`

## 1. 契约范围与当前证据

本规格描述一个可嵌入 lib、也可被 service 宿主调用的订单簿内核。当前 workspace 只证明 `domain_market::OrderBook`、`PriceLevel` 及排序/update-id 纯检查存在；不存在 `BookEvent`、state machine、provider adapter、materializer 或真实流回放实现。因此以下 `specified` 不得解释为 `verified`。

## 2. 统一事件与 SPI

### 2.1 事件语义

| 字段 | 语义 | 约束 |
|---|---|---|
| `kind` | `Snapshot` 或 `Diff` | Snapshot 可独立消费；Diff 只应用变更 |
| `venue/market/symbol` | provider 维度 | 不得只用 symbol 跨 provider 去重 |
| `event_time_ms` | provider 事件/生成时间 | 缺失时保留缺失或拒绝 |
| `received_at_ms` | 本地完整收到时间 | 由 ingestion 注入 |
| `bids/asks` | `Level { price, quantity }` | Decimal；Diff 中 quantity=0 删除 |
| `seq` | `first/final/previous_final` | 仅保留 provider 明确字段 |
| `checksum` | provider 可选的历史/兼容字段 | 当前 OKX 文档标为废弃且固定为 0；不得默认解释为完整性凭证 |

适配器 SPI 的语义接口：

1. `Parser`：raw bytes → 一个或多个 `BookEvent`，解析错误映射为 `Parse`。
2. `ContinuityRule`：负责外部快照对齐和 SYNCED 连续性，必须是可测试纯规则。
3. `IntegrityVerifier`：可选，仅用于当前 provider 文档明确有效的完整性字段；失败必须触发 resync。不能因为事件中出现名为 `checksum` 的字段就启用旧算法。
4. `SnapshotSource`：仅模型 A 使用；模型 B/C 不应伪造外部 snapshot。
5. `ResubscribeAction`：模型 B 发生恢复时由宿主执行；不得由 core 直接连接 provider。

### 2.2 统一输出

`BookSnapshot` 应包含 `venue`、`market`、原始 `symbol`、可选规范化标的、时间、位点、Top-N bids/asks、mid、spread、weighted mid、imbalance、深度指标、`state` 和 `lag_ms`。输出必须能转换为 `domain_market::OrderBook`；派生指标不能反向改变公共簿。

规范化 symbol 的 owner 尚未确定。当前 domain_market 只承诺 `InstrumentKey { exchange, symbol }`，不得在本规格中把 draft 的 `BTC-USDT.PERP` 当成已冻结公共值。

## 3. 三种同步模型

### 3.1 模型 A：外部快照 + diff（Binance）

Spot 快照位点为 `L`：丢弃 `u <= L`；首个有效事件须满足 `U <= L+1 <= u`；之后须 `U == previous_u + 1`。UM/CM：丢弃 `u < L`；首个有效事件须 `U <= L <= u`；之后须 `pu == previous_u`。任何不满足均 resync。上述规则必须以官方原始 fixture 复核，当前仅为 draft-derived requirement。

### 3.2 模型 B：流内快照 + diff（OKX/Coinbase）

首个合法 snapshot 直接建立簿。后续 diff 需先过 provider sequence rule，再应用绝对量档位；失败时清理簿、标记 `WAITING_BOOTSTRAP` 并请求重订阅。OKX 当前以 `prevSeqId == previous seqId` 为连续性凭证；官方文档说明 checksum 已废弃且固定为 0，因此不能执行 draft 中的 CRC32 checksum gate。Coinbase Advanced Trade `level2` 文档保证更新交付，消息外层 `sequence_num` 不能仅凭字段名假定为每簿严格递增；必须以 raw fixture 确认其作用域，未确认前使用连接/订阅恢复边界。

### 3.3 模型 C：全量快照流（Hyperliquid）

每条有效消息是完整 bids/asks，必须调用 `ReplaceAll`，不应用 diff 合并；不要求 sequence continuity。空/非法/交叉快照丢弃并告警；静默超阈值进入 `STALE`。只有官方 wire schema 与 fixture 足够时，才增加增量优化。

## 4. 内核状态转移

| 当前 | 事件 | 动作 | 下一状态 |
|---|---|---|---|
| `INIT` | start | 启动对应 bootstrap | A=`BUFFERING`，B=`WAITING_BOOTSTRAP`，C=`INIT` |
| `BUFFERING` | diff | 有界缓存，满时拒绝/计数，不静默丢失 | `BUFFERING` |
| `BUFFERING` | snapshot ok | 装载位点并开始对齐 | `ALIGNING` |
| `ALIGNING` | 首事件合法 | 应用并消费后续连续事件 | `SYNCED` |
| `SYNCED` | 合法 diff | 应用、更新位点、质量检查 | `SYNCED` |
| `SYNCED` | 过期/重复 | 幂等丢弃并计数 | `SYNCED` |
| `SYNCED` | gap/integrity/crossed | 清理状态并执行模型恢复动作 | `RESYNC` 或 `WAITING_BOOTSTRAP` |
| 任意健康态 | 静默超阈值 | 保留诊断信息，标记过期 | `STALE` |
| `STALE` | 有效新 bootstrap/full snapshot | 替换并恢复 | `SYNCED` |
| 恢复重试超预算 | retry exhausted | critical 告警，停止自动重试 | `HALTED` |

事件 buffer 必须有界、可观测，不能以“丢最旧且无恢复”作为默认正确性策略。若 draft profile 采用丢弃策略，必须同时触发 resync/告警并证明没有静默产出。

## 5. 应用与质量规则

1. Snapshot：按完整状态替换；Diff：非零数量覆盖、零数量删除。
2. 每次应用后校验 bid/ask 排序、数量非负、crossed 和事件时间滞后。
3. 序列字段缺失时返回“不可判定”而不是假设连续；adapter 必须决定拒绝或声明无序列模型。
4. sequence/有效 integrity 状态不得丢进 `domain_market::OrderBook` 后再猜回；废弃 checksum 不得被当作有效状态。
5. resync 必须记录 reason、venue、market、symbol、previous state、位点和重试次数。

## 6. Service profile（待实现）

draft 定义的 Kafka raw/snapshot、Redis Top-N mirror、ClickHouse 全档锚点、NATS 告警和 Prometheus 指标属于可选 service profile。profile 的 topic/key/table 名称、物化频率、快照深度和 REST 权重必须在实现仓库中另行冻结；本 workspace 当前没有这些基础设施 crate，不能将其列为已实现。

## 7. 可执行门禁

| ID | 要求 | 证据 | 状态 |
|---|---|---|---|
| OB-API-001 | `BookEvent`、模型、状态和输出语义冻结 | 本 spec + Rust API review | specified |
| OB-CORE-001 | core 不依赖 provider adapter | 依赖检查 + core tests | pending |
| OB-SYNC-001 | 三种同步模型状态转移矩阵 | deterministic state-machine tests | pending |
| OB-BN-001 | Binance spot/UM/CM 对齐边界 | raw depth fixtures + replay | pending |
| OB-OK-001 | OKX `seqId/prevSeqId` 链、snapshot `prevSeqId=-1` 与 sequence reset | official/raw fixtures + gap/reset test | pending |
| OB-OK-002 | OKX 废弃 checksum 不启用旧 CRC32 算法 | official docs + negative compatibility test | specified |
| OB-CB-001 | Coinbase level2 snapshot/update、绝对量与连接恢复边界 | raw level2 fixtures + reconnect/replay test | pending |
| OB-HL-001 | Hyperliquid full refresh 无残留 | full-book replacement test | pending |
| OB-QUAL-001 | 排序、负数量、crossed、lag 检查 | unit/property tests | pending |
| OB-RESYNC-001 | 断裂/校验失败恢复且有界 | fault injection + metrics assertion | pending |
| OB-OUT-001 | 四家统一快照/指标 schema | schema fixture + consumer test | pending |
| OB-SVC-001 | Kafka/Redis/ClickHouse/NATS profile | integration test in service package | deferred |
| OB-PERF-001 | diff/full-refresh P99 目标 | reproducible benchmark | deferred |

## 8. 冲突和状态裁决

- v1 的 Binance 12 簿、Kafka topic 和 Go 线程模型是 profile；不覆盖 v2 的四 provider 通用内核契约。
- `domain_market::OrderBook` 的 `DM-BOOK-001/002` 只负责公共形状、排序和 update-id 纯检查；OB-BN/OK/CB/HL 的恢复状态机属于本主题和各 adapter。
- 当前 adapter skeleton 的 `Internal` 返回值只证明入口存在；不能关闭本主题任何 runtime 门禁。
