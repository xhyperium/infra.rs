# InstrumentCatalog

| 字段 | 值 |
|------|-----|
| Trait | `contracts::InstrumentCatalog` |
| 实现面 | binancex / okxx |
| Fake | scaffold adapter |

## ownership

- `symbol_info` 返回拥有的 `SymbolMeta`。

## success

- `Ok(SymbolMeta)`。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 未知 symbol | `Missing` 或 `Invalid` |
| 未连接 / 不可达 | `Unavailable` |
| 瞬时 | `Transient` |

## idempotency

- 只读；幂等。

## cancel / timeout

- Future 取消 / deadline。

## ordering

- 不适用。

## resource release

- 无。

## not-found

- 未知交易对：`Missing`（推荐）或实现稳定映射。

## pagination

- 本面无 list-all；Additive Only。

## object-safety

- 是。

## fake entry

- scaffold `symbol_info`。

## test entry

- adapter 单测；`tests/public_surface.rs`。
