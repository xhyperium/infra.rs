# Wire Commitment Matrix — `canonical`

| 字段 | 值 |
|---|---|
| SSOT | 本文件是 committed wire current-state 类型清单 |
| 状态 | **Approved L2 committed subset**；≠ package stable |
| 格式边界 | strict serde JSON DTO shape |
| 更新 | 2026-07-23 |

## 1. 承诺含义

`Committed` 冻结下列受测行为：

- JSON object 字段名与必填性；
- enum 的 externally tagged / JSON string variant 表示；
- committed 类型 `deny_unknown_fields`，未知 variant 反序列化失败；
- `Decimal` / `Price` / `Qty` 嵌套 JSON shape 及非法 scale 拒绝；
- 双向 golden 与已登记 N-1/legacy 向量读取；
- `ts: i64` 的 Unix epoch 纳秒语义。

`Committed` **不承诺** canonical bytes、序列化字节的跨实现确定性、通用 codec、schema registry、跨语言协议、自动迁移、任意大版本兼容或 package stable。

## 2. 精确版本与兼容查询

| 接口 | 语义 |
|---|---|
| `WireVersion::V1` / `V1_1` / `V1_2` / `V1_3` | 精确 committed 批次常量 |
| `committed_wire_version(type_name)` | 返回精确版本；未知名与 `Money` 返回 `None` |
| `WireCommitment::{CommittedV1,Uncommitted}` | 既有 coarse 等级，保持 API 兼容 |
| `wire_commitment(type_name)` | 任一 committed 版本均返回 `CommittedV1`；不得据此判断精确版本 |
| `COMMITTED_WIRE_V1{,_1,_2,_3}` | 四个显式 inventory 常量 |

## 3. 完整类型矩阵

| 类型 | 精确版本 | JSON shape / 语义 | Golden / N-1 证据 |
|---|---|---|---|
| `CancelOrderRequest` | v1 | object：venue/instrument/id；unknown/missing field 拒绝 | v1 cancel + 既有 OKX fixture，双向 |
| `OrderRef` | v1 | externally tagged `Client` / `Exchange` | v1 两个 variant fixture + 拒绝 unknown variant |
| `OrderAck` | v1 | object：id/status/ts；legacy shape 冻结 | v1 + root legacy fixture；N-1 读取 |
| `OrderStatus` | v1 | JSON string；6 variants | 全 variants inline golden；unknown variant 拒绝 |
| `Side` | v1 | JSON string；Buy/Sell | 全 variants inline golden；unknown variant 拒绝 |
| `Order` | v1.1 | id/symbol/side/price/qty/status | `v1.1/order_legacy.json`；双向 + N-1 + 拒绝样例 |
| `Tick` | v1.2 | symbol/bid/ask/ts(ns) | `v1.2/tick_legacy.json`；双向 + N-1 + 拒绝样例 |
| `Trade` | v1.2 | symbol/price/qty/ts(ns) | `v1.2/trade_legacy.json`；双向 + N-1 + 拒绝样例 |
| `Position` | v1.3 | symbol/qty/entry_price | `v1.3/position_legacy.json`；双向 + N-1 + 拒绝样例 |
| `PriceLevel` | v1.3 | price/qty | `v1.3/price_level_legacy.json`；双向 + N-1 + 拒绝样例 |
| `OrderBookSnapshot` | v1.3 | symbol/bids/asks/ts(ns)；仅快照 | `v1.3/order_book_snapshot_legacy.json`；双向 + N-1 + 嵌套拒绝 |
| `SymbolMeta` | v1.3 | symbol/base/quote/tick_size/min_qty | `v1.3/symbol_meta_legacy.json`；双向 + N-1 + 拒绝样例 |

## 4. 不属于本 crate committed inventory

| 项 | 裁定 |
|---|---|
| `Money` | 本 crate 仅 re-export；wire SSOT 在 `decimalx` |
| `VenueId` / `InstrumentId` | `String` aliases，不作为独立 serde wire 类型登记 |
| `Envelope<T>` | 运输包装，独立于 DTO v1–v1.3 清单；不自动路由 |
| 未知类型名 | `committed_wire_version` → `None`；`wire_commitment` → `Uncommitted` |

## 5. Unknown-field 与演进规则

- committed struct/enum 源码均标注 `#[serde(deny_unknown_fields)]`；Envelope 外层同样 deny。
- 未知字段、未知 variant、缺少必填字段、非法 decimal scale 均须有负向测试。
- 当前 `*_legacy.json` 在首次冻结处可能与当前字段集完全相同；“N-1”只表示该已登记历史向量继续可读，不代表存在通用 migration reader。
- 破坏性变更必须采用新类型或显式版本化迁移，并新增 golden、N-1、拒绝样例与 CHANGELOG；不得静默修改既有 committed shape。

## 6. Envelope 关系

`Envelope<T>` 只提供 `schema_version` + `payload` 运输形状。反序列化不会检查调用方支持的版本；消费者必须显式调用 `validate_version(expected)` 或 `into_payload_if_version(expected)`。它不是版本协商器、自动 decoder router 或通用 codec。
