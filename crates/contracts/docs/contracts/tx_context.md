# TxContext

| 字段 | 值 |
|------|-----|
| Trait | `contracts::TxContext` |
| 实现面 | postgresx 等 |
| Fake | `FakeTxContext` / `RecordingTxRunner` 内嵌上下文 |

## ownership

- 由 [`TxRunner::begin_tx`](./tx_runner.md) 产出 `Box<dyn TxContext>`；调用方独占可变借用直至 commit/rollback。
- 当前 trait 不暴露业务写方法；终结后不得再使用上下文。

## success

- `commit` → `Ok(())`：后端确认生命周期提交完成；具体业务可见性不由本 trait 证明。
- `rollback` → `Ok(())`：后端确认生命周期回滚完成。

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
- Future 取消/panic 时通用合同只保证 drop context，不保证可等待异步 rollback。

## ordering

- 本 trait 没有业务操作面；跨事务顺序由具体存储实现定义。

## resource release

- 正常路径必须 `commit` 或 `rollback` 释放后端事务句柄。
- 推荐 `run_tx_lifecycle`：结构化保留业务+rollback 双失败；commit 失败不自动 rollback。
- deprecated `run_tx_commit_on_ok` 只保留兼容，会丢弃 rollback 错误。

## not-found

- 不适用。

## pagination

- 不适用。

## object-safety

- 是（`dyn TxContext`，`Send` + `&mut self`；不要求 `Sync`）。

## fake entry

- `FakeTxContext`、`RecordingTxRunner`

## test entry

- `fakes::tests::fake_tx_context_*` / `recording_tx_runner_commit_and_rollback`
- `run_tx_lifecycle_*`（四类错误、取消/drop、对象安全）
