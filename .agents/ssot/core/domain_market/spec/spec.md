# domain_market 市场数据域模型规格

**版本**：0.2.3
**状态**：契约冻结；类型 + 时间/簿纯校验、serde fixture 与 typed envelope 并存已实现；全量 typed 管道/canonical 迁移仍待
**实现**：`crates/domain_market/src/lib.rs`、`book.rs`、`time.rs`
**最后更新**：2026-07-22

## 1. 职责与 canonical 边界

本域定义 Tick、Quote、Bar、OrderBook、市场来源和跨交易所聚合数据的公共形状。交易所 wire DTO、HTTP/WebSocket 客户端和重连策略属于 adapter/transport，不在本 crate 实现。

当前 workspace 的唯一 instrument 类型是本 crate 的 `InstrumentKey { exchange, symbol }`。旧文档中“由 `xhyper-canonical` 直接提供”“`product_line + symbol`”与当前源码不一致，暂不作为契约；canonical 迁移由 `DM-CAN-001` 管理。

## 2. 公共类型

### 2.1 标的与产品线

```rust
pub struct InstrumentKey {
    pub exchange: String,
    pub symbol: String,
}

pub enum ProductLine {
    Spot,
    Future,
    Perpetual,
    Option,
}
```

`exchange` 使用稳定的小写 provider key（例如 `binance`、`okx`）；`symbol` 是 provider 规范化后的原始标的，不把不同交易所的分隔符强行抹平。adapter 必须保存双向映射并用 `InstrumentKey` 做 key；不能仅使用 `symbol` 跨交易所去重。

### 2.2 基础行情

- `Tick { instrument, price, quantity, side: Option<TickDirection>, trade_id, timestamp, received_at }`
- `Quote { instrument, bid_price, bid_quantity, ask_price, ask_quantity, bid_levels, ask_levels, timestamp, received_at }`
- `PriceLevel { price, quantity, order_count }`
- `Bar { instrument, interval, open_time, close_time, open, high, low, close, volume, quote_volume, trade_count, taker_buy_volume, taker_buy_quote_volume }`
- `OrderBook { instrument, bids, asks, sequence, first_update_id, last_update_id, timestamp, update_type }`

枚举 `TickDirection`、`OrderBookUpdateType` 和 `BarInterval` 与源码一致，均为 `#[non_exhaustive]`。所有时间是 Unix 毫秒；`received_at` 是本地收到完整消息的时间，不能用 provider event time 代替。

`BarInterval` 当前使用 `#[serde(untagged)]` 的整数变体；零值 interval、月份长度和未闭合 bar 的聚合规则尚未实现，不能由字段名推断。

### 2.3 订单簿不变量

| ID | Snapshot | Delta |
|---|---|---|
| DM-BOOK-001 | `update_type=Snapshot`；levels 是可独立消费的当前状态；provider 无 ID 时保持 None | `update_type=Delta`；只应用 provider 明确的增量；不得把缺失 ID 当连续 |
| DM-BOOK-002 | bid 按价格降序，ask 按价格升序 | 若 provider sequence/checksum 断裂，丢弃增量并请求新 snapshot |
| DM-BOOK-003 | `first_update_id/last_update_id` 可选仅因 provider 不提供 | Binance 的 `U/u/lastUpdateId` 必须保留到这两个字段 |

OKX checksum、Coinbase `sequence_num`、Hyperliquid 的具体恢复规则属于各 adapter spec；公共 `OrderBook` 不足以表达 provider-specific checksum，适配器必须在内部保留原始校验状态。

### 2.4 MarketFactEnvelope 当前基线

当前源码是：

```rust
pub struct MarketFactEnvelope {
    pub instrument: InstrumentKey,
    pub source: DataSource,
    pub fact_type: String,
    pub data: serde_json::Value,
    pub timestamp: Timestamp,
}
```

`MarketFactEnvelope`（`fact_type + JSON data` + 可选 `sequence`）保持兼容管道；typed `MarketFact` / `MarketSubject` 已并存（DM-ENV-001）。聚合数据点必须走 `MarketSubject::Aggregate` 或独立聚合类型，不得强行塞入单一 `instrument`。

`DataSource` 当前覆盖源码中的 Binance、OKX、Bybit、Bitget、KuCoin、Gate、Mexc、Htx、Coinbase、Hyperliquid、Lighter、Upbit、Coinglass。新增 provider 只增加 non-exhaustive 变体并同步证据，不修改历史事件的字符串语义。

### 2.5 聚合数据

- `OpenInterestPoint { coin, exchange, oi, oi_value, timestamp }`：`oi` 为合约数量，`oi_value` 为 USD 名义价值。
- `FundingRatePoint { coin, exchange, rate, timestamp }`：`rate` 为带符号的小数，例如 `0.0001`，不是百分比字符串。
- `LiquidationData { coin, exchange, side, amount, price, timestamp }`：`amount` 为 USD；`side` 为 `Long | Short`。
- `LongShortRatioData { coin, exchange, long_ratio, short_ratio, timestamp }`：字段单位必须在 adapter evidence 标明是 0–1 小数还是 0–100 百分比；当前公共源码注释仍为 percentage，禁止静默换算。

## 3. 时间、序列和精度

| 字段 | 语义 | 来源缺失时 |
|---|---|---|
| `timestamp` | provider event/data time | 保留缺失状态或拒绝，不能伪造 |
| `received_at` | 本地收到完整消息的 wall clock | 必须由 ingestion 注入 |
| `open_time/close_time` | provider candle 区间边界 | 以 provider 原始字段为准，不能用 interval 猜测 |
| `sequence` | provider 或内部单调序列，必须在 adapter evidence 标明种类 | `None` |

所有价格、数量和费率使用 `domainx::Decimal`。转换必须保持字符串精度；异常数字、NaN、负数量和 bid/ask 反转属于 parse/validation error。

## 4. 适配器映射总原则

1. 先解析 provider DTO，再生成本域类型；禁止用 `serde_json::Value` 直接猜字段。
2. 原始 symbol、provider exchange、ProductLine 映射必须可逆；无法可逆时返回 `InvalidRequest`/`Parse` 并记录原因。
3. 单一 mid/mark price 不能伪装成完整 `Quote`；应等待真实 bid/ask 或新增 `MidPrice` 类型。
4. Snapshot/Delta、sequence gap、重复消息、重连恢复均须有固定 fixture。
5. 聚合数据的 `source=Coinglass` 与 `exchange=<被聚合交易所>` 是两个不同维度，不能混写。

## 5. 依赖

| 依赖 | 用途 |
|---|---|
| `domainx` | `Decimal`、`Timestamp`、共享交易值对象 |
| `serde`/`serde_json` | 精确序列化与兼容 envelope |
| `chrono` | 当前 workspace 兼容依赖；公共契约仍以毫秒为准 |

## 6. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| DM-API-001 | 公共结构体/枚举与源码字段逐项一致 | `cargo test -p domain_market` | verified |
| DM-TIME-001 | 毫秒、event/received、Bar 边界 fixture | `src/time.rs` + `tests/serde_and_time.rs` + fixtures | verified |
| DM-BOOK-001 | Snapshot/Delta 排序、update id 纯检查；provider 恢复状态机仍在 adapter | `src/book.rs` 纯函数 + snapshot fixture（非 live 恢复） | verified（纯检查） |
| DM-ENV-001 | typed `MarketFact`/`MarketSubject` + envelope.sequence（与 JSON envelope 并存） | `MarketFact` + `tests/env_typed.rs` | verified（typed 并存；全量迁移非目标） |
| DM-CAN-001 | `InstrumentKey/ProductLine` 的唯一 canonical owner | ADR-001 + workspace 迁移 PR | blocked |
| DM-SER-001 | camelCase、Decimal 无精度损失、fixture round-trip | `tests/fixtures/*.json` + `tests/serde_and_time.rs` | verified |

说明：DM-BOOK-001 的 **provider 跳号恢复 / checksum 状态机** 仍属 adapter 范围；本 crate 仅验证公共档位排序与 update id 区间。DM-ENV-001 仅 verified 于 typed 与兼容 envelope 并存，DM-CAN-001 仍 blocked；全量管道迁移不在本门禁范围。


DM-CAN-001 规划见 `design/ADR-001-canonical-instrument.md`（依赖未入仓前保持 blocked）。
