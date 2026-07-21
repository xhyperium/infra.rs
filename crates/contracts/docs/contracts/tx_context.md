# TxContext

| 字段 | 值 |
|------|-----|
| Trait | `contracts::TxContext` |
| 实现面 | postgresx 等 |
| Fake | `FakeTxContext` / `RecordingTxRunner` 内嵌上下文 |

## ownership

- 由 [`TxRunner::begin_tx`](./tx_runner.md) 产出 `Box<dyn TxContext>`；调用方独占可变借用直至 commit/rollback。
- 终结后不得再用于业务写。

## success

- `commit` → `Ok(())`：变更对后续读可见（实现定义隔离级别）。
- `rollback` → `Ok(())`：丢弃未提交变更。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 提交冲突 | `Conflict` |
| 瞬时失败 | `Transient`（`FakeTxContext::with_commit_failure`） |
| 已终结后再 commit（若实现检测） | `Invalid` 或 `Invariant` |
| 内部错误 | `Internal` |

## idempotency

- 重复 `commit` / `rollback`：实现定义；不得在已终结后静默再次变更外部状态。
- Fake：允许再次设置标志，不模拟后端错误。

## cancel / timeout

- 无独立 cancel API；事务边界由编排层控制。
- 超时：实现可 `rollback` 后返回 `DeadlineExceeded`。

## ordering

- 单上下文内操作顺序即提交顺序；跨事务顺序由存储隔离级别决定。

## resource release

- 必须 `commit` 或 `rollback` 释放后端事务句柄。
- 参考编排 `run_tx_commit_on_ok`：Ok→commit，Err→rollback（rollback 错误被吞并保留业务 Err）。

## not-found

- 不适用。

## pagination

- 不适用。

## object-safety

- 是（`dyn TxContext`，但 `&mut self` 方法）。

## fake entry

- `FakeTxContext`、`RecordingTxRunner`

## test entry

- `fakes::tests::fake_tx_context_*` / `recording_tx_runner_commit_and_rollback`
- `run_tx_commit_on_ok_*`（lib 单测 + `tests/conformance_first_batch.rs`）
