# `canonical` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | **Approved** 实现合同（S1，2026-07-17）；**≠** package stable / 全 wire Production Ready |
| Package / lib | `xhyper-canonical` / `canonical` |
| Path | `crates/types/canonical` |
| Layer | Types / 跨层共享纯 DTO |
| Authority | 本文件是 active current-state spec（与 20260717 Approved 语义对齐） |
| Complete Spec | [SPEC-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-spec.md)（Approved；≠ package stable） |
| Goal | [GOAL-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-goal.md) |
| Plan | [plan/plan.md](../plan/plan.md) · [alignment-matrix-infra-2026-07-21.md](../plan/alignment-matrix-infra-2026-07-21.md) · [todo.md](../todo.md) |
| Residual | [plan/residual-open.md](../plan/residual-open.md) |
| Dual mirror | 本文件 ≡ [xhyper-canonical-complete-spec.md](./xhyper-canonical-complete-spec.md)（须 `cmp`） |

> `[KNOWN]` 为当前代码或 Approved 裁决直接证据。API、serde attributes/fixtures 变化会使相应结论失效。

## 1. 定位与依赖

- `[KNOWN] HIGH` ADR-001/007：canonical 只放跨层共享数据形状，不含业务逻辑。
- `[KNOWN] HIGH` `Money`/`Decimal` 族唯一归属 `decimalx`；canonical 只 `pub use decimalx::Money`。
- 普通依赖：`xhyper-decimalx`、`serde`；dev-dependency：`serde_json`。
- 禁止依赖 contracts、domain、adapter、service、app 或 evidence。
- 非目标：状态机、订单簿 diff、业务校验/授权、I/O、审计、重试、通用 codec、hash/sign/evidence。

## 2. 当前公开 API

### 2.1 标识与取消

| 类型 | 当前形状 |
|---|---|
| ~~`OrderId`~~ | **类型已删除**（2026-07-17）；`Order`/`OrderAck`.id 为 wire `String` |
| `VenueId` | `String` alias；adapter 入口用 `shape::is_plausible_venue_slug` |
| `InstrumentId` | `String` alias；`shape::is_plausible_instrument_id`；不做跨所归一 |
| `OrderRef` | `Client(String)` / `Exchange(String)` |
| `CancelOrderRequest` | `venue`, `instrument`, `id: OrderRef` |

### 2.2 枚举与 DTO

| 类型 | 当前形状 |
|---|---|
| `OrderStatus` | Pending/Open/PartiallyFilled/Filled/Cancelled/Rejected |
| `Side` | Buy/Sell |
| `Order` | id/symbol/side/price/qty/status（`id: String`） |
| `OrderAck` | id/status/ts（`id: String`；`ts` = Unix **ns**） |
| `Position` | symbol/qty/entry_price |
| `Tick` | symbol/bid/ask/ts（ns） |
| `PriceLevel` | price/qty |
| `OrderBookSnapshot` | symbol/bids/asks/ts（ns） |
| `Trade` | symbol/price/qty/ts（ns） |
| `SymbolMeta` | symbol/base/quote/tick_size/min_qty |

另：`pub use decimalx::Money`。所有字段公开；无业务方法、I/O、全局状态。辅助模块：`shape`、`proposed_time`。

## 3. 已批准语义边界

### CAN-BND / CAN-NUM / CAN-LAYER — `APPROVED`

- 纯 DTO；`OrderBookSnapshot` 无 diff/merge/排序状态机。
- 金额/价格/数量来自 `decimalx`；禁止 f32/f64 金融字段。
- 不反向依赖 contracts/domain/adapter。

### CAN-ID-001 — `APPROVED`（2026-07-17）

- 新接口优先 `OrderRef` / `CancelOrderRequest`。
- `OrderId` **类型已删**；id 字段为 `String`。
- Venue/Instrument 保持 alias；形状由 `shape::*` 在 adapter 入口校验。
- newtype 二期：见 residual OPEN-ID-002（DEFERRED）。

### CAN-TIME-001 — `APPROVED`（2026-07-17）

- DTO `ts: i64` = Unix epoch **纳秒**（与 `kernel::Timestamp` 同刻度）。
- canonical **不**依赖 kernel；交易所 ms → DTO 经 `ns_from_unix_millis` / `dto_ts_from_unix_millis`。

### CAN-WIRE-001 — 部分 candidate

- Committed-candidate：`CancelOrderRequest`、`OrderRef`（fixtures + 单测）。
- Committed-legacy：`OrderAck` JSON shape。
- 其余 DTO：**Uncommitted**（仅 serde RT）。
- 未知字段 / 全量跨版本 golden：OPEN（residual OPEN-WIRE-*）。

### CAN-VALID-001 — 原则 `APPROVED`

- 本 crate 不做业务校验；owner 表见 [validation-owners.md](../plan/validation-owners.md)。

### CAN-CODEC-001 — `REJECTED`

- 禁止 Canonical Encoding Core / schema registry / hash-sign-evidence 进本 crate。

## 4. Serde 与 fixtures

- 本地 DTO/枚举 derive serde（默认字段/variant shape）。
- 固定 wire 证据：
  - `fixtures/market/order_cancel_okx.json`（cancel 双向）；
  - `fixtures/market/order_ack_legacy.json`；
  - `fixtures/market/canonical/v1/*`（cancel / OrderRef / legacy ack）。

## 5. 测试与门禁

必须覆盖：

- 各公开 DTO/枚举构造 + serde round-trip；
- 全部 `OrderStatus` / `OrderRef` variants；
- cancel / legacy ack / v1 golden 双向；
- `Money` 与 `decimalx::Money` 类型同一；
- 时间 ms↔ns 与 venue shape 正反例；
- 无 domain 行为或上层依赖。

```bash
cargo test -p xhyper-canonical
cargo check -p xhyper-canonical --all-targets
cargo clippy -p xhyper-canonical --all-targets -- -D warnings
cargo fmt -p xhyper-canonical -- --check
```

## 6. 仍 OPEN / HUMAN / DEFER（不得假装 DONE）

| 项 | 标签 | 指针 |
|----|------|------|
| package stable / crates.io | HUMAN_ONLY / DEFER S2 | residual DEFER-STABLE |
| 全 DTO 跨版本 wire 冻结 | OPEN / DEFER | OPEN-WIRE-002 |
| unknown-field deny 策略 | OPEN | OPEN-WIRE-001 |
| OrderRef newtype 二期 | DEFER | OPEN-ID-002 |
| types/core·protocol 大搬迁 | DEFER | OPEN-LAYOUT-001 |
| 移除 serde | DEFER | OPEN-SERDE-001 |
| 非主路径全量 consumer 迁移 | DEFER | DEFER-M3-REST |

## 7. 追溯

- Complete Spec：[20260717/xhyper-canonical-complete-spec.md](../20260717/xhyper-canonical-complete-spec.md)
- Alignment：[plan/alignment-matrix-infra-2026-07-21.md](../plan/alignment-matrix-infra-2026-07-21.md)
- Residual：[plan/residual-open.md](../plan/residual-open.md)
- Production：[plan/production-upgrade.md](../plan/production-upgrade.md)
- 实现：`crates/types/canonical/{Cargo.toml,src/**}`
- 依赖：`crates/types/decimal`
- Fixtures：`fixtures/market/**`
