# orderbook 设计

## 1. 决策摘要

### D1：v2 通用内核取代 v1 重复状态机

`.cargo/draft/orderbook/1.md` 的 `orderbook-core` v2 作为主设计；`.cargo/draft/orderbook.md` 的 v1 不丢弃，其 Binance spot/UM/CM 对齐规则作为模型 A 和 12 簿 service profile 的具体化。两份 draft 的差异是部署范围差异，不是两套公共内核。

### D2：三种同步模型

| 模型 | 典型 provider | 初始化 | 增量/恢复 |
|---|---|---|---|
| A `SnapshotPlusDiff` | Binance | 外部 REST snapshot + 缓存事件对齐 | 连续性失败后重新拉 snapshot |
| B `StreamBootstrapDiff` | OKX、Coinbase | 流内 snapshot | 连续性失败后重订阅；完整性字段按当前官方文档处理 |
| C `FullRefresh` | Hyperliquid | 第一条完整消息 | 每条整簿替换；不伪造 diff 恢复 |

内核只依赖模型枚举和注入的规则，不出现 `if exchange == ...`。provider 名称只能存在于 adapter 注册和观测标签。

### D3：适配器边界

适配器负责原始 JSON/WS/REST、符号与产品线映射、序列字段归一、provider 当前有效的完整性字段、快照来源和重订阅动作；内核负责事件应用、状态迁移、排序/crossed/新鲜度检查和统一输出。

### D4：`domain_market` 是公共形状 owner

内核输出最终应转换为 `domain_market::OrderBook`，但不在 `domain_market` 中实现 provider-specific 恢复状态机。不得在 orderbook 主题再定义第二套公共 `InstrumentKey`、`PriceLevel` 或 `OrderBook`；内部 `BookEvent` 仅为执行边界类型，必须提供明确转换。

### D5：目标输出与当前 workspace 分离

Redis/Kafka/ClickHouse/NATS 是 draft 的 service 输出，不是当前 workspace 的已存在依赖。lib 形态应先接受宿主喂入的 `BookEvent`；service 形态再复用同一内核接入 Kafka/WS，并由独立 materializer 输出。

## 2. 分层

```text
provider wire / host input
        │
        ▼
L1 adapter: Parser + Rule + Verifier + SnapshotSource/Resubscribe
        │ 统一 BookEvent
        ▼
L2 core: state machine + sorted BookStore + QualityGuard
        │ BookSnapshot / BookMetrics
        ▼
L3 materializer: Redis mirror / Kafka snapshot / OLAP anchor / query API
```

依赖方向必须是 `core → core types`；`core` 不依赖 `adapters/*`；adapter 不能互相引用。物化层不能改变簿状态，只能消费 immutable view。

## 3. 内部事件模型

概念模型如下，具体 Rust 类型需在实现任务中冻结：

```text
BookEvent {
  kind: Snapshot | Diff,
  venue: provider key,
  market: provider market,
  symbol: provider symbol,
  event_time_ms: i64,
  received_at_ms: i64,
  bids: Vec<Level>,
  asks: Vec<Level>,
  sequence: SeqInfo,
  checksum: Option<u32>,
}
Level { price: Decimal, quantity: Decimal }
SeqInfo { first: u64, final: u64, previous_final: Option<u64> }
```

`Diff` 的数量是绝对量，零表示删除档位；`Snapshot` 的档位是完整当前状态。时间统一为毫秒，Decimal 不得经浮点中转。缺失 provider 序列不得被填成连续值。

## 4. 状态机

统一状态超集：`INIT`、`BUFFERING`、`WAITING_BOOTSTRAP`、`ALIGNING`、`SYNCED`、`RESYNC`、`STALE`、`HALTED`。

- 模型 A 使用 `INIT → BUFFERING → ALIGNING → SYNCED`；快照失败重试，超过预算进入 `HALTED`。
- 模型 B 使用 `INIT → WAITING_BOOTSTRAP → SYNCED`；序列断裂或当前有效的完整性检查失败时清理状态并请求宿主重订阅。
- 模型 C 第一条完整快照进入 `SYNCED`；静默只进入 `STALE`，整条消息恢复时替换并回到 `SYNCED`。
- 任意模型发现 crossed、非法数量或无法解释的事件时不得继续应用；动作由模型决定，但必须产生告警和原因指标。

## 5. 统一质量语义

- bid 按价格降序，ask 按价格升序。
- `quantity == 0` 只在 diff 语义中删除对应价格档；snapshot 中零量档应在 adapter 侧拒绝或清理并记录。
- `best_bid >= best_ask` 视为 crossed；模型 C 丢弃该快照，模型 A/B 进入恢复。
- `lag_ms = received_at_ms - event_time_ms` 仅在两者可比较时计算；缺失事件时间不能伪造。
- `STALE` 是数据新鲜度状态，不等价于序列断裂；静默阈值按 provider 配置。

## 6. 关键取舍

- 不把 Hyperliquid 的 mid price 或全量 book 伪装成 Quote/Delta。
- 不把 OKX 废弃 checksum 塞进公共 `OrderBook` 或当作 CRC32 门禁；若 provider 将来公开有效完整性字段，由 adapter verifier 单独保留和解释。
- 不在 core 中硬编码 `BTCUSDT`、档位数量或 Kafka topic；这些属于 profile/config。
- 不以 draft 中的 Go API 直接作为 Rust 公共 API；先保留语义契约，再按 workspace 风格设计 Rust trait。
