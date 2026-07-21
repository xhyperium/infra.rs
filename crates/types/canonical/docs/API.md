# canonical 公开 API

**Package**：`canonical` · **角色**：跨层纯 DTO  
**生产层级**：L2 committed wire subset（v1–v1.3）

## 公开消费面

### 类型别名与 re-export

| 符号 | 说明 |
|------|------|
| `VenueId` / `InstrumentId` | `String` 别名 |
| `Money` | `decimalx::Money` re-export（wire SSOT 在 decimalx） |

### DTO / 枚举

| 符号 | Wire | 说明 |
|------|------|------|
| `OrderRef` | v1 | `Client(String)` · `Exchange(String)` |
| `CancelOrderRequest` | v1 | `venue` · `instrument` · `id` |
| `OrderStatus` | v1 | `Pending` · `Open` · `PartiallyFilled` · `Filled` · `Cancelled` · `Rejected` |
| `Side` | v1 | `Buy` · `Sell` |
| `OrderAck` | v1 | `id` · `status` · `ts`（Unix ns） |
| `Order` | v1.1 | `id` · `symbol` · `side` · `price` · `qty` · `status` |
| `Tick` / `Trade` | v1.2 | 行情与成交 |
| `Position` / `OrderBookSnapshot` / `PriceLevel` / `SymbolMeta` | v1.3 | 持仓 / 簿 / 元数据 |

Committed 类型均 `deny_unknown_fields` + serde 往返。

### shape

| 函数 | 说明 |
|------|------|
| `is_nonempty_token` | 非空 trim token |
| `is_plausible_venue_slug` | venue slug 形状 |
| `is_plausible_instrument_id` | instrument 形状 |
| `order_ref_payload_nonempty` | OrderRef 载荷非空 |
| `cancel_request_shape_ok` | 取消请求形状 |

### proposed_time

| 符号 | 说明 |
|------|------|
| `TS_UNIT` / `PROPOSED_TS_UNIT` | `"unix_epoch_nanoseconds"` |
| `ns_from_unix_millis` / `proposed_ns_from_unix_millis` | ms → ns（溢出 `None`） |
| `unix_millis_from_ns` / `proposed_unix_millis_from_ns` | ns → ms |
| `dto_ts_from_unix_millis` / `proposed_dto_ts_from_unix_millis` | DTO 时间入口 |

### wire

| 符号 | 说明 |
|------|------|
| `WireCommitment` | `CommittedV1` · `Uncommitted` |
| `COMMITTED_WIRE_V1` / `_V1_1` / `_V1_2` / `_V1_3` | 类型名清单 |
| `wire_commitment(&str)` | 查询承诺等级 |

### 模块路径

`canonical::shape` · `canonical::proposed_time` · `canonical::wire`

## 最小用法

```rust
use canonical::{Order, OrderStatus, Side, wire_commitment, WireCommitment};
use decimalx::{Decimal, Price, Qty};

let o = Order {
    id: "1".into(),
    symbol: "BTCUSDT".into(),
    side: Side::Buy,
    price: Price::new(Decimal::new(1, 0)),
    qty: Qty::new(Decimal::new(1, 0)),
    status: OrderStatus::Open,
};
assert_eq!(wire_commitment("Order"), WireCommitment::CommittedV1);
let _ = serde_json::to_string(&o).unwrap();
```

```bash
cargo run -p canonical --example basic
```

## 覆盖

`tests/public_api_surface.rs` 驱动 shape/time/wire 与全部 DTO serde 往返。  
API 棘轮：`docs/api-baselines/canonical.txt`。  
Golden：`fixtures/market/canonical/v1{,.1,.2,.3}/`。
