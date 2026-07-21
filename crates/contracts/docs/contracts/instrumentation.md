# Instrumentation

| 字段 | 值 |
|------|-----|
| Trait | `contracts::Instrumentation` |
| 实现面 | observex；消费方 resiliencx |
| Fake | `RecordingInstrumentation` |

## ownership

- 同步、无异步；实现应廉价、不阻塞。
- 通常 `Arc<dyn Instrumentation>` 注入。

## success

- `record_*` 无返回值；失败应内部消化（日志），不得 panic 影响热路径。

## failure / XError kinds

- 方法签名无 `XResult`；错误不跨边界。

## idempotency

- 重复 record 产生多条事件；调用方控制频次。

## cancel / timeout

- 不适用（同步即时）。

## ordering

- Recording 按调用顺序追加；生产 metrics/tracing 顺序尽力而为。

## resource release

- 无。

## not-found

- 不适用。

## pagination

- 不适用。

## object-safety

- 是（`dyn Instrumentation`）。

## fake entry

- `RecordingInstrumentation` + `InstrEvent`

## test entry

- `recording_instrumentation_records_events`
- `tests/conformance_first_batch.rs`
