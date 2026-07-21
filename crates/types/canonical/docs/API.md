# canonical 公开 API

**角色**：跨层纯 DTO

## 公开消费面

- DTO：`Order` / `OrderAck` / `OrderRef` / `CancelOrderRequest` / `OrderStatus` / `Side` / `Position` / `Tick` / `Trade` / `OrderBookSnapshot` / `PriceLevel` / `SymbolMeta`
- 时间：`ns_from_unix_millis` 等（纳秒刻度）
- 形状：`is_plausible_venue_slug` / `cancel_request_shape_ok` 等
- Wire：`wire_commitment` / `COMMITTED_WIRE_V1*` / `WireCommitment`

## 最小用法

```rust
use canonical::{Order, OrderStatus, Side};
use decimalx::{Decimal, Price, Qty};

let o = Order {
    id: "1".into(),
    symbol: "BTCUSDT".into(),
    side: Side::Buy,
    price: Price::new(Decimal::new(1, 0)),
    qty: Qty::new(Decimal::new(1, 0)),
    status: OrderStatus::Open,
};
let _ = serde_json::to_string(&o).unwrap();
```
