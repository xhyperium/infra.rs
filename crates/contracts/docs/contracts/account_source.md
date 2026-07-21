# AccountSource

| 字段 | 值 |
|------|-----|
| Trait | `contracts::AccountSource` |
| 实现面 | binancex / okxx |
| Fake | scaffold adapter |

## ownership

- 返回拥有的 `Vec<Position>` / `Vec<Money>`。

## success

- `Ok(vec![…])`；空账户为空 Vec，非错误。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 未认证 / 参数 | `Invalid` |
| 未连接 | `Unavailable` |
| 瞬时 | `Transient` |

## idempotency

- 只读快照；幂等（值可能变）。

## cancel / timeout

- Future 取消 / deadline。

## ordering

- Vec 顺序实现定义。

## resource release

- 无。

## not-found

- 无持仓/余额 → 空集合，非 `Missing`。

## pagination

- 本面一次返回全量；大账户分页 Additive Only。

## object-safety

- 是。

## fake entry

- scaffold `query_position` / `query_balance`。

## test entry

- adapter 单测；`tests/public_surface.rs`。
