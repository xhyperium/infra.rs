# `canonical` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 实现合同；wire/生产语义未稳定 |
| Package / lib | `xhyper-canonical` / `canonical` |
| Path | `crates/types/canonical` |
| Layer | Types / 跨层共享纯 DTO |
| Authority | 本文件是 active current-state spec |
| Candidate | [SPEC-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Goal | [GOAL-TYPES-CANONICAL-002](../20260717/xhyper-canonical-complete-goal.md)（Draft） |
| Plan | [plan/plan.md](../plan/plan.md) · [todo.md](../todo.md) |
| Implementation snapshot | `4fe8e988`（2026-07-17 campaign baseline） |
| Document commit | `26b4238befc70ffaae5c7828729c84e29551bc4f` |
| Verified at | `26b4238befc70ffaae5c7828729c84e29551bc4f`（agent-safe 闭合核对） |

> `[KNOWN]` 为当前代码或 Approved ADR 直接证据；`[INFERRED]` 为最低限度推论。API、serde attributes/fixtures 或 ADR 变化会使相应结论失效。

## 1. 定位与依赖

- `[KNOWN] HIGH` ADR-001/007 规定 canonical 只放跨层共享数据形状，不含业务逻辑。
- `[KNOWN] HIGH` `Money`/`Decimal` 族唯一归属 `decimalx`；canonical 只复用和重导出 `Money`。
- 普通依赖：`xhyper-decimalx`、`serde`；dev-dependency：`serde_json`。
- 禁止依赖 contracts、L1、domain、adapter、service、app 或 evidence。

非目标：状态机、订单簿 diff、校验/授权、I/O、审计、重试、通用 canonical codec、hash/sign/evidence。

## 2. 当前公开 API

### 2.1 标识与取消

| 类型 | 当前形状 |
|---|---|
| `OrderId` | deprecated `String` alias；为 legacy wire compatibility 保留 |
| `VenueId` | `String` alias；语义未冻结 |
| `InstrumentId` | `String` alias；语义未冻结 |
| `OrderRef` | `Client(String)` / `Exchange(String)` |
| `CancelOrderRequest` | `venue`, `instrument`, `id: OrderRef` |

### 2.2 枚举与 DTO

| 类型 | 当前形状 |
|---|---|
| `OrderStatus` | Pending/Open/PartiallyFilled/Filled/Cancelled/Rejected |
| `Side` | Buy/Sell |
| `Order` | id/symbol/side/price/qty/status |
| `OrderAck` | id/status/ts |
| `Position` | symbol/qty/entry_price |
| `Tick` | symbol/bid/ask/ts |
| `PriceLevel` | price/qty |
| `OrderBookSnapshot` | symbol/bids/asks/ts |
| `Trade` | symbol/price/qty/ts |
| `SymbolMeta` | symbol/base/quote/tick_size/min_qty |

另：`pub use decimalx::Money`。

所有字段公开；crate 当前无业务方法、I/O、全局状态、异步、锁或本地错误类型。

## 3. 当前语义边界

- `OrderBookSnapshot` 只表达全量快照；排序、diff/merge、交叉盘口和新鲜度不由本 crate 保证。
- `OrderStatus` variants 存在不等于状态迁移已批准。
- `ts: i64` 的 epoch/单位/范围尚未统一裁定（**OPEN**；不得自行声称毫秒或纳秒）。
- Venue/Instrument/Order ID 的字符集、规范化、唯一性与 provider 映射尚未稳定（**OPEN**）。
- 负 Qty、非法价格、symbol 存在性等必须由 adapter/domain 校验；成功反序列化不等于业务有效。

## 4. Serde 与兼容事实

- 本地 DTO/枚举当前 derive serde，使用默认字段/variant shape。
- **已有固定 wire 证据**（测试 + fixture）：
  - `CancelOrderRequest`：`fixtures/market/order_cancel_okx.json` 双向读写；
  - legacy `OrderAck`：JSON shape 回归测试。
- `Order` 及其他类型：有/可有 serde round-trip；**不等于**跨版本或跨语言 wire 稳定承诺。
- 未知字段策略、schema 版本、枚举扩展策略：**OPEN**（见 Candidate 与 [residual-open.md](../plan/residual-open.md)）。

具体兼容治理与迁移候选见 Candidate Draft；Candidate **不**覆盖本 active 合同。

## 5. 当前测试与开放项

测试覆盖（agent-safe 目标）：

- 各公开 DTO/枚举的构造与 serde round-trip；
- 全部 `OrderStatus` / `OrderRef` variants；
- cancel golden fixture 正向+反向；
- legacy `OrderAck` wire 字符串；
- `Money` 与 `decimalx::Money` 类型同一；
- 空订单簿可构造。

开放（**不得**标为已批准）：时间单位、ID/symbol 规范、unknown-field、schema/version、枚举扩展、legacy `OrderId` 迁移时机、各 DTO 的 validation owner。

## 6. 验收

```bash
cargo test -p xhyper-canonical
cargo check -p xhyper-canonical --all-targets
cargo clippy -p xhyper-canonical --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

通过条件：依赖方向正确；API 与源码一致；无 domain 行为或重复数值类型；文档不虚构稳定 wire；不宣称 Spec Approved / package stable。

## 7. 追溯

- [ADR-001](../../../../../docs/architecture/adr/001-venue-adapter-boundary.md)
- [ADR-007](../../../../../docs/architecture/adr/007-spec-consistency-revision.md)
- `crates/types/canonical/{Cargo.toml,src/lib.rs}`
- `fixtures/market/order_cancel_okx.json`
- Candidate：[20260717/xhyper-canonical-complete-spec.md](../20260717/xhyper-canonical-complete-spec.md)
- Plan：[plan/plan.md](../plan/plan.md)
