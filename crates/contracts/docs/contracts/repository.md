# Repository

| 字段 | 值 |
|------|-----|
| Trait | `contracts::Repository<T, Id>` |
| 实现面 | postgresx 等 |
| Fake | `FakeRepository<T, Id>` |

## ownership

- `save` 借 `entity`；实现自行拷贝/序列化。
- `find` 返回拥有的 `T`（`Option`）。

## success

- `find` → `Ok(Some(T))` / `Ok(None)`（不存在）。
- `save` → `Ok(())` 表示持久化被接受（insert 或 upsert，实现定义）。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 约束冲突 | `Conflict` |
| 参数非法 | `Invalid` |
| 瞬时 / 不可达 | `Transient` / `Unavailable` |
| 内部 | `Internal` |

## idempotency

- 同一 `Id` 重复 `save`：推荐 upsert 语义；Fake 为覆盖写。

## cancel / timeout

- Future 取消；超时映射 `DeadlineExceeded` 或 `Transient`。

## ordering

- 无全局顺序；事务内顺序见 `TxContext`。

## resource release

- 无会话句柄；连接由实现管理。

## not-found

- `find` → `Ok(None)`，非错误。

## pagination

- 本最小面无 list/page；Additive Only 扩展。

## object-safety

- 否（泛型 trait）；具体 `Repository<MyT, MyId>` 可对象化仅当类型参数固定且 object-safe 边界满足。
- 对象安全边界由调用方用具体类型或包装 trait 解决。

## fake entry

- `FakeRepository::new(|entity| id)`

## test entry

- `fake_repository_save_find`
- `tests/conformance_first_batch.rs`
