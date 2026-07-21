# ExecutionVenue

| 字段 | 值 |
|------|-----|
| Trait | `contracts::ExecutionVenue` |
| 实现面 | binancex / okxx（及能力拆分） |
| Fake | 树内 adapter scaffold 充当前端替身；无独立 Fake（W4 前） |

## ownership

- **推荐生产入口**（相对 `VenueAdapter` 迁移 facade）。
- `place_order` 借 `Order`；返回 `OrderAck`。
- cancel/query 使用结构化 [`CancelOrderRequest`]（CAN-ID）。
- `venue_id()` 返回 `VenueId`（拥有的 `String`）。

## success

- `place_order` → `OrderAck`。
- `cancel_order` → `Ok(())`。
- `query_order` → `OrderStatus`。

## failure / XError kinds

| 场景 | 建议 kind |
|------|-----------|
| 参数/状态非法 | `Invalid` |
| 订单不存在 | `Missing` |
| 冲突（重复下单等） | `Conflict` |
| 瞬时 / 不可达 | `Transient` / `Unavailable` |
| 未连接（scaffold） | `Unavailable` |

**无 additive default**：实现必须提供全部方法。

## idempotency

- 客户端订单 ID / `OrderRef` 语义由交易所与上层约定。
- 重复 cancel：实现应安全（已取消 → 成功或 `Conflict`/`Missing`，须稳定）。

## cancel / timeout

- cancel 即业务 API；与 Future 取消不同。
- 网络超时 → `DeadlineExceeded` / `Transient`。

## ordering

- 交易所侧顺序；本 trait 不保证跨请求全局序。

## resource release

- 无长事务；连接状态由 adapter `connect`/`disconnect`（在 `VenueAdapter`）管理。

## not-found

- 查询缺失订单：`Missing`（或实现文档化的 `Invalid`）；须稳定。

## pagination

- 不适用。

## object-safety

- 是（`dyn ExecutionVenue`）。

## fake entry

- 使用 in-tree `BinanceAdapter` / `OkxAdapter` scaffold（见 `tests/venue_override_gate.rs`）。

## test entry

- adapter 单测 + `tests/venue_override_gate.rs`
- 与 `VenueAdapter` 关系见该文件模块文档。
