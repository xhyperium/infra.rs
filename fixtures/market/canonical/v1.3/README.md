# canonical wire goldens — v1.3

| 文件 | 类型 | 等级 |
|------|------|------|
| `position_legacy.json` | `Position` | Committed v1.3 |
| `price_level_legacy.json` | `PriceLevel` | Committed v1.3 |
| `order_book_snapshot_legacy.json` | `OrderBookSnapshot` | Committed v1.3 |
| `symbol_meta_legacy.json` | `SymbolMeta` | Committed v1.3 |

- 订单簿仅为快照结构体（无 diff/增量语义，ADR-001）。
- 变更须跑 `cargo test -p canonical`。
