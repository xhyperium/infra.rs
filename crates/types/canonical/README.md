# canonical

`/types/` 跨层共享 DTO（ADR-001，spec §4.2）。只放纯数据形状，无业务逻辑。

| 项 | 值 |
|----|-----|
| package | `canonical` |
| lib | `canonical` |
| path | `crates/types/canonical` |
| version | `0.1.0` |
| publish | `false`（internal only） |
| **生产层级** | **L2 committed wire subset**（v1 / v1.1 / v1.2 / v1.3） |
| 支持矩阵 | Linux x86_64 · MSRV 1.85 |

> **已承诺 wire**：见 `COMMITTED_WIRE_V1*` 与 `wire_commitment`。  
> **不是** 全 DTO package Production Ready / crates.io / schema registry。

## 主要内容

- 复用 `decimalx::Money`（ADR-007）
- 标识：`VenueId`、`InstrumentId`、`OrderRef`
- 取消：`CancelOrderRequest`
- 枚举：`OrderStatus` / `Side`
- DTO：`Order` / `OrderAck` / `OrderBookSnapshot` / `Position` / `SymbolMeta` / `Tick` / `Trade` / `PriceLevel`
- 辅助：`shape`（形状检查）、`proposed_time`（ns↔ms）、`wire`（承诺等级）
- 信封：`Envelope<T>`（`schema_version` + `payload`；无业务校验）
- **时间**：DTO `ts: i64` = Unix epoch **纳秒**（CAN-TIME-001）

## Committed wire 清单

| 批次 | 类型 |
|------|------|
| v1 | `CancelOrderRequest` · `OrderRef` · `OrderAck` · `OrderStatus` · `Side` |
| v1.1 | `Order` |
| v1.2 | `Tick` · `Trade` |
| v1.3 | `Position` · `OrderBookSnapshot` · `PriceLevel` · `SymbolMeta` |

Golden：`fixtures/market/canonical/v1{,.1,.2,.3}/`

## 硬限制

- 无业务行为方法；不做 I/O；不依赖 L1/适配器
- 不替代 `contracts` trait 出口
- `Money` re-export 不单独承诺 wire（SSOT 在 decimalx）
- adapter 从交易所 ms 入口须经 `ns_from_unix_millis`
- 金额字段必须来自 `decimalx`，禁止浮点别名

## 最小用法

```bash
cargo run -p canonical --example basic
```

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

## 验证

```bash
cargo test -p canonical --all-targets
cargo clippy -p canonical --all-targets -- -D warnings
cargo bench -p canonical --bench hot_path -- --quick
```

公开 API：[docs/API.md](./docs/API.md) · 变更日志：[CHANGELOG.md](./CHANGELOG.md)
