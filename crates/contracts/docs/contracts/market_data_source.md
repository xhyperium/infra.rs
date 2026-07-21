# MarketDataSource

| 字段 | 值 |
|------|-----|
| Trait | `contracts::MarketDataSource` |
| 实现面 | binancex / okxx |
| Fake | scaffold adapter；无独立 Fake |

## ownership

- 订阅返回 `'static` `BoxStream`；调用方拥有流。
- 从 `VenueAdapter` 拆出的行情能力。

## success

- `subscribe_*` → 可轮询/异步的流（可为空流）。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| symbol 非法 | `Invalid` |
| 未连接 / 不可达 | `Unavailable` |
| 瞬时 | `Transient` |

## idempotency

- 重复订阅：实现定义（多路流或复用）；调用方应避免泄漏订阅。

## cancel / timeout

- 丢弃 stream 取消订阅（实现尽力）。
- 连接超时 → `DeadlineExceeded` / `Unavailable`。

## ordering

- 交易所推送序；本地不重排。

## resource release

- 丢弃 stream / 断开 WS 由实现负责。

## not-found

- 未知 symbol：`Invalid` 或 `Missing`（须文档化）。

## pagination

- 不适用；流式。

## object-safety

- 是。

## fake entry

- in-tree adapter scaffold。

## test entry

- adapter `subscribe_*` 单测；`tests/public_surface.rs` 对象安全绑定。
