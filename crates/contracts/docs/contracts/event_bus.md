# EventBus

| 字段 | 值 |
|------|-----|
| Trait | `contracts::EventBus` |
| 实现面 | kafkax / natsx |
| Fake | `FakeEventBus` |

## ownership

- `publish` 传递 `Bytes`（廉价 clone）；消息 ID 由实现分配。
- `subscribe` 返回 `'static` `BoxStream`；流所有权归调用方。

## success

- `publish` → `Ok(())`：至少完成进程内投递或后端 ack（实现定义）。
- `subscribe` → 流项为 [`BusMessage`]（含 `id` + `payload`）。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| topic 非法 | `Invalid` |
| 后端瞬时 / 不可达 | `Transient` / `Unavailable` |
| lock 中毒等内部 | `Internal`（Fake） |

## idempotency

- 调用方可按 `BusMessage.id` 做消费幂等。
- 本最小面最多表达 **at-most-once**；不保证必达或 redelivery。

## cancel / timeout

- 流丢弃即停止本地消费；远端取消时点由实现定义（Fake 为一次性快照流）。
- 无内建 ack/nack API（见 `MessageAck` 类型预留）。

## ordering

- 单分区顺序由后端约定；Fake 按 publish 顺序回放。

## resource release

- 丢弃 stream / 断开订阅由实现负责；Fake 无外部资源。

## not-found

- 空 topic：`subscribe` → 空流（Fake），非错误。

## pagination

- 不适用；背压由 Stream 模型表达。

## object-safety

- 是（`dyn EventBus`）。

## fake entry

- `FakeEventBus`

## test entry

- `assert_event_bus_surface` 是不 poll 的 portable surface。
- `assert_event_bus` 只属于 snapshot/replay Fake profile，不是 Kafka/NATS 通用 conformance。
- `tests/public_surface.rs`、`tests/conformance_first_batch.rs`
