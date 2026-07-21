# KeyValueStore

| 字段 | 值 |
|------|-----|
| Trait | `contracts::KeyValueStore` |
| 实现面 | redisx 等（adapters/storage） |
| Fake | `FakeKeyValueStore` |

## ownership

- 调用方拥有 key / value 的字节所有权传递；`set` 消费 `Vec<u8>`。
- 实现拥有持久化/缓存生命周期；调用方不得假设跨进程内存地址稳定。

## success

- `get` → `Ok(Some(bytes))` 命中；`Ok(None)` 表示键不存在（**不是**错误）。
- `set` → `Ok(())` 表示写入被接受（含覆盖）。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 参数非法（空 key 等，若实现校验） | `Invalid` |
| 后端瞬时故障 | `Transient` |
| 后端不可达 | `Unavailable` |
| 内部不变式破坏 | `Internal` |

禁止 panic；禁止用字符串匹配驱动重试。

## idempotency

- `set` 同一 key 重复写：最后一次覆盖前值（至少一次语义可接受）。
- `get` 只读，天然幂等。

## cancel / timeout

- 方法本身无 cancel 句柄；由调用方 `Future` 取消或上层 deadline 驱动。
- 超时映射为 `DeadlineExceeded` 或 `Transient`（实现约定，须稳定）。

## ordering

- 单 key 读写顺序：同一客户端串行调用时，后写对后续 `get` 可见。
- 跨客户端顺序由后端提供；本 trait 不保证全局线性一致性。

## resource release

- 无会话句柄；实现内部连接池由 adapter 管理。
- TTL：实现可驱逐；`FakeKeyValueStore` **不**自动过期（仅记录）。

## not-found

- 用 `Ok(None)`，**不要**用 `ErrorKind` 表示缺失键。

## pagination

- 不适用（单 key API）。列表/扫描若需要，Additive Only 扩展。

## object-safety

- 是（`dyn KeyValueStore`）。

## fake entry

- `contracts::FakeKeyValueStore`

## test entry

- 单元：`fakes::tests::fake_kv_get_set_roundtrip`
- 集成：`tests/conformance_first_batch.rs`
