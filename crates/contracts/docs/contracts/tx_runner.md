# TxRunner

| 字段 | 值 |
|------|-----|
| Trait | `contracts::TxRunner` |
| 实现面 | postgresx 等 |
| Fake | `FakeTxRunner` / `RecordingTxRunner` |

## ownership

- Runner 通常为长生命周期共享（`Arc<dyn TxRunner>` 友好）。
- 每次 `begin_tx` 产出独立 `TxContext`。

## success

- `begin_tx` → `Ok(Box<dyn TxContext>)`。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 连接耗尽 / 瞬时 | `Transient` / `Unavailable` |
| 配置非法 | `Invalid` |
| 内部 | `Internal` |

## idempotency

- 多次 `begin_tx` 各自独立；非幂等写操作。

## cancel / timeout

- `begin_tx` 可被 Future 取消；已开启事务由调用方负责 rollback。

## ordering

- 事务之间无全局顺序保证（除非后端提供）。

## resource release

- 调用方必须终结每个 `TxContext`。
- 推荐 `run_tx_commit_on_ok(&dyn TxRunner, …)`。

## not-found

- 不适用。

## pagination

- 不适用。

## object-safety

- **是**（设计硬约束；`&dyn TxRunner` 合同测覆盖）。

## fake entry

- `FakeTxRunner`、`RecordingTxRunner`

## test entry

- `tx_runner_is_object_safe`、`recording_tx_runner_commit_and_rollback`
- `tests/conformance_first_batch.rs` / `tests/public_surface.rs`
