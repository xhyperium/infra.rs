# VenueTimeSource

| 字段 | 值 |
|------|-----|
| Trait | `contracts::VenueTimeSource` |
| 实现面 | binancex / okxx |
| Fake | scaffold adapter |

## ownership

- 返回 `i64` 服务器时间（实现定义 epoch/单位；scaffold 可为 0）。

## success

- `Ok(ts)`。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 未连接 | `Unavailable` |
| HTTP 非 200 | `Unavailable` |
| 瞬时 | `Transient` |

## idempotency

- 只读；连续调用可返回更新时间。

## cancel / timeout

- Future 取消 / deadline。

## ordering

- 单调性不保证（NTP/服务器跳变）。

## resource release

- 无。

## not-found

- 不适用。

## pagination

- 不适用。

## object-safety

- 是。

## fake entry

- scaffold `server_time`。

## test entry

- adapter 单测；`tests/public_surface.rs`。
