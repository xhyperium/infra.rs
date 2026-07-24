# market_data API

## 类型

| 类型 | 说明 |
|------|------|
| `InstrumentType` | 交易产品类型（Spot/Perpetual/Future/Option） |
| `MarketTick` | 标准化行情数据 |

## 使用示例

```rust
use market_data::{InstrumentType, MarketTick};
use market_data::Decimal;
use chrono::Utc;

let tick = MarketTick {
    symbol: "BTC-USDT".into(),
    last_price: Decimal::new(50000, 0),
    volume_24h: Decimal::new(123456, 2),
    timestamp: Utc::now(),
};
```
