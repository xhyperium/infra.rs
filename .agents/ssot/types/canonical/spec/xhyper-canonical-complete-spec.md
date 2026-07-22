# `canonical` 当前实现规范

| 字段 | 值 |
|---|---|
| 状态 | **Approved current-state**（2026-07-23） |
| Package / lib / version | `canonical` / `canonical` / `0.1.2`（Cargo 选择器：`-p canonical`） |
| Path | `crates/types/canonical` |
| 层级 | **L2 committed wire subset**：strict serde JSON DTO shape（v1 / v1.1 / v1.2 / v1.3） |
| 发布边界 | `publish = false`；**不是** package stable / crates.io Production Ready |
| Active authority | 本文件 + [Wire Commitment Matrix](../plan/wire-commitment-matrix.md) |
| 历史裁决 | [SPEC-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-spec.md) · [GOAL-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-goal.md) |
| Residual | [residual-open.md](../plan/residual-open.md) |

> 本文件描述当前 crate 已实现并承诺的边界。20260717 文档保留历史裁决语境，但不覆盖本文件登记的 current-state 实现事实。

## 1. 定位与明确非目标

- `canonical` 只提供跨层共享纯 DTO、轻量形状/时间辅助与运输包装；不包含业务状态机、I/O、授权、风控、重试或审计。
- `Money` / `Decimal` / `Price` / `Qty` 的唯一数值定义点是 `decimalx`；本 crate 仅 `pub use decimalx::Money`。
- 生产依赖仅 `decimalx` 与 `serde`；不得反向依赖 contracts、domain、adapter、service、app、kernel 或 evidence。
- 本 crate 的 committed wire 只承诺受测的 **serde JSON DTO shape**：字段名、枚举表示、必填字段、未知字段/variant 拒绝及嵌套 decimal shape。
- **不承诺** canonical bytes、确定性字节编码、通用 codec、schema registry、hash/sign/evidence、任意格式转换或跨语言协议。
- L2 committed subset **不等于** 整个 package stable，也不等于所有消费者、所有大版本或所有语言的兼容保证。

## 2. 当前公开数据面

### 2.1 标识与取消

| 类型 | 形状与语义 |
|---|---|
| `VenueId` | `String` alias；adapter 入口可用 `shape::is_plausible_venue_slug` |
| `InstrumentId` | `String` alias；不做跨 venue 归一化 |
| `OrderRef` | externally tagged JSON enum：`Client(String)` / `Exchange(String)` |
| `CancelOrderRequest` | `venue`, `instrument`, `id: OrderRef` |
| `OrderId` | **不存在**；`Order.id` / `OrderAck.id` 为 wire `String` |

### 2.2 DTO 与枚举

| 类型 | 字段 / variants | Committed wire |
|---|---|---|
| `OrderStatus` | Pending / Open / PartiallyFilled / Filled / Cancelled / Rejected | v1 |
| `Side` | Buy / Sell | v1 |
| `OrderAck` | id / status / ts | v1（legacy shape） |
| `Order` | id / symbol / side / price / qty / status | v1.1 |
| `Tick` | symbol / bid / ask / ts | v1.2 |
| `Trade` | symbol / price / qty / ts | v1.2 |
| `Position` | symbol / qty / entry_price | v1.3 |
| `PriceLevel` | price / qty | v1.3 |
| `OrderBookSnapshot` | symbol / bids / asks / ts | v1.3 |
| `SymbolMeta` | symbol / base / quote / tick_size / min_qty | v1.3 |

`OrderBookSnapshot` 仅为快照形状，不承诺排序、合并、diff 或增量更新语义。完整 wire 清单、证据和限制见 [wire-commitment-matrix.md](../plan/wire-commitment-matrix.md)。

## 3. Wire 查询与兼容面

- `WireVersion` 精确表示 `V1` / `V1_1` / `V1_2` / `V1_3`。
- `committed_wire_version(type_name)` 返回类型所属的精确 committed 版本；未知类型及 `Money` 不属于本 crate 清单。
- `WireCommitment` / `wire_commitment(type_name)` 保留兼容：任一已承诺版本仍粗粒度返回 `CommittedV1`，未知项返回 `Uncommitted`。不得用该兼容查询推断精确版本。
- `COMMITTED_WIRE_V1{,_1,_2,_3}` 是四个显式类型名清单；新增、删除或迁移 committed 类型必须同步实现、golden、测试、CHANGELOG 与本 SSOT。

## 4. Strict serde JSON 合同

- 所有 committed DTO/enum 都 derive serde 并标注 `deny_unknown_fields`。
- 结构体 JSON object 拒绝未知字段与缺失必填字段；没有隐式 default。
- `OrderRef` 使用 externally tagged enum；`OrderStatus` / `Side` 使用 Rust variant 名对应的 JSON 字符串；未知 variant 反序列化失败。
- `Decimal` / `Price` / `Qty` 的 JSON shape 与合法 scale 由 `decimalx` 约束；非法 scale 的拒绝测试属于 committed 证据。
- v1–v1.3 各 committed 表示均有文件 golden 或穷举 inline golden；已登记 legacy/N-1 向量的结构 DTO 保持可读。首次冻结批次的 `*_legacy.json` 与当前字段集相同，只证明该历史向量仍可读取；它不是通用 migration reader 或跨大版本兼容声明。

## 5. 时间合同

- `OrderAck` / `Tick` / `Trade` / `OrderBookSnapshot` 的 `ts: i64` 是 Unix epoch **纳秒**。
- `ns_from_unix_millis` / `dto_ts_from_unix_millis` 对 ms→ns 使用 checked multiplication，溢出返回 `None`。
- `unix_millis_from_ns` 是兼容保留的**向 0 截断**转换；调用者不得把结果视为无损。
- `unix_millis_from_ns_exact` 只在纳秒值可被 `1_000_000` 整除时返回毫秒，否则返回 `None`；需要无损语义时必须使用该入口。
- canonical 与 kernel 仅共享时间刻度，不建立 crate 依赖。

## 6. `Envelope<T>` 边界

- `Envelope<T>` 是运输包装，JSON 字段固定为 `schema_version` + `payload`；外层拒绝未知字段。
- `ENVELOPE_SCHEMA_VERSION` 描述包装形状；`CURRENT_PAYLOAD_SCHEMA_VERSION` 是 payload 的默认起点。两者当前数值均为 `1`，语义相互独立。
- `wrap` / `wrap_current` 只包装数据；serde 反序列化只校验字段形状与类型。
- 消费方在使用 payload 前必须显式调用 `validate_version(expected)` 或 `into_payload_if_version(expected)`。Envelope **不自动路由、不协商版本、不选择 DTO decoder，也不证明 payload 的业务有效性**。
- `Envelope<T>` 不属于 v1–v1.3 DTO committed 清单，不把本 crate 扩展为通用 envelope/codec 框架。

## 7. Golden、N-1 与验证要求

证据目录：

- `fixtures/market/canonical/v1/`：cancel / OrderRef / legacy OrderAck；
- `fixtures/market/canonical/v1.1/`：Order；
- `fixtures/market/canonical/v1.2/`：Tick / Trade；
- `fixtures/market/canonical/v1.3/`：Position / PriceLevel / OrderBookSnapshot / SymbolMeta；
- `fixtures/market/order_cancel_okx.json` 与 `fixtures/market/order_ack_legacy.json`：既有 consumer 基线。

测试必须覆盖：

- committed 清单与精确 `WireVersion` 映射，以及 coarse `WireCommitment` 兼容行为；
- 所有公开 DTO/enum serde round-trip 与所有 variants；
- 双向 golden，以及有登记的 N-1/legacy 历史向量；
- 未知字段、未知 variant、缺字段与非法 decimal scale 拒绝；
- `Money` 与 `decimalx::Money` 类型同一；
- ms→ns 溢出、ns→ms 截断与 exact 转换的整除/非整除边界；
- Envelope round-trip、缺/未知字段、显式版本成功与不匹配。

```bash
cargo test -p canonical -p decimalx
cargo check -p canonical --all-targets
cargo clippy -p canonical --all-targets -- -D warnings
cargo fmt -p canonical -- --check
node scripts/quality-gates/check-canonical-align.mjs
```

## 8. 版本与剩余边界

- 当前交付版本为 `0.1.2`；相对 `0.1.1` 的 wire/time 查询行为仅执行一次 PATCH bump，并同步 Cargo/CHANGELOG/发布证据。
- package stable / 发布仍为 HUMAN_ONLY；newtype、布局搬迁、serde 移除与非主路径 consumer 迁移仍为 deferred。
- v1–v1.3 committed 清单、strict unknown-field 策略和 golden/N-1 已不再是 OPEN。后续若提出破坏性 wire 变化，必须新建版本化迁移决策，不得回退成含混的“全 DTO wire OPEN”。
