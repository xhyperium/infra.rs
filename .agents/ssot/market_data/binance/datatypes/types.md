# 数据类型映射

> Binance 17 种数据类型与 domain_market 规范类型的映射表。
> 审查来源: R6 (数据类型映射审查)

## 实时 WebSocket 类型

| Binance 流 | 规范类型 | 状态 | 备注 |
|-----------|---------|:---:|------|
| @trade | Tick | ✓ 已存在 | 逐笔成交 |
| @aggTrade | Tick | ✓ 已存在 | 聚合成交 (含 is_buyer_maker) |
| @bookTicker | Quote | ✓ 已存在 | 最优买/卖报价 |
| @depth@100ms | OrderBook::Delta | ✓ 已存在 | 增量深度 (U/u 校验) |
| @depth20@100ms | OrderBook::Snapshot | ✓ 已存在 | 20 档快照 |
| @kline_1m | Bar | ✓ 已存在 | 1 分钟 K 线 |
| @ticker | **TwentyFourHrTicker** | ✗ 缺失 | 24 小时统计 (价格变化/成交量) |
| @markPrice | **MarkPrice** | ✗ 缺失 | 标记价格 + 资金费率 |
| @optionTicker | **OptionGreeks** | ✗ 缺失 | 期权 Greeks / IV |
| @index | **IndexPrice** | ✗ 缺失 | 指数价格 |
| @forceOrder | 待分类 | ⊘ | 强制平仓单 |

## REST API 类型

| Binance 端点 | 规范类型 | 状态 | 备注 |
|-------------|---------|:---:|------|
| exchangeInfo | InstrumentMeta[] | ✓ 已存在 | 交易对元信息 |
| depth?limit=N | OrderBook::Snapshot | ✓ 已存在 | REST 深度快照 |
| klines | Bar[] | ✓ 已存在 | 历史 K 线 |
| aggTrades | Tick[] | ✓ 已存在 | 历史成交 |
| listenKey | SessionToken | ✓ 已存在 | WebSocket 鉴权令牌 |

## 缺失类型规格

以下 4 种规范类型需在 domain_market 中补充定义：

### TwentyFourHrTicker

```rust
/// 24 小时行情统计
pub struct TwentyFourHrTicker {
    pub symbol: SmolStr,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub last_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub event_time: i64,
}
```

### MarkPrice

```rust
/// 标记价格（合约）
pub struct MarkPrice {
    pub symbol: SmolStr,
    pub mark_price: Decimal,
    pub index_price: Decimal,
    pub funding_rate: Decimal,
    pub next_funding_time: i64,
    pub event_time: i64,
}
```

### OptionGreeks

```rust
/// 期权 Greeks
pub struct OptionGreeks {
    pub symbol: SmolStr,
    pub delta: Decimal,
    pub gamma: Decimal,
    pub theta: Decimal,
    pub vega: Decimal,
    pub implied_volatility: Decimal,
    pub event_time: i64,
}
```

### IndexPrice

```rust
/// 指数价格
pub struct IndexPrice {
    pub symbol: SmolStr,
    pub index_price: Decimal,
    pub event_time: i64,
}
```

## 精度安全

- 所有价格/数量使用 `rust_decimal::Decimal`
- Binance JSON 中价格以字符串表示，禁止通过 f64 中转（精度损失风险）
- 映射层 (mapping/binance.rs) 中 DTO 结构体使用 `String` 反序列化，再转换为 `Decimal`

## 时间戳

- Binance 全程使用毫秒 (ms) Unix 时间戳
- domain_market 同样使用毫秒 (i64)
- 无需精度转换
