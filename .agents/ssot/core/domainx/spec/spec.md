# domainx 共享交易值对象规格

**版本**：0.2.3
**状态**：契约冻结；类型 + DX-VAL 纯校验与 DX-API-002 fixture 已实现；canonical / 多资产手续费仍待迁移
**实现**：`crates/domainx/src/lib.rs`、`crates/domainx/src/validate.rs`
**最后更新**：2026-07-22

## 1. 职责与边界

`domainx` 是 L0 共享值对象层，承载订单、持仓、成交、执行回报和组合的跨域数据结构。它只定义数据和序列化边界，不执行下单、风控、网络请求、持久化或 PnL 计算。

当前 workspace 没有 `xhyper-canonical` 依赖，因此交易对象的 `instrument` 暂以 `String` 表示。这是明确的兼容占位，不得在新 adapter 中继续引入第二种 instrument 结构；待 canonical crate 纳入 workspace 后，由一次迁移统一替换并更新本 spec。

## 2. 公共类型

### 2.1 标识与时间

```rust
pub type OrderId = String;
pub type TradeId = String;
pub type ReportId = String;
pub type PositionId = String;
pub type PortfolioId = String;
pub type Timestamp = i64; // Unix milliseconds since 1970-01-01T00:00:00Z
pub use rust_decimal::Decimal;
```

ID 在同一交易所、账户和对象类型内唯一。`Timestamp` 统一为毫秒；adapter 不得把秒、微秒或 `DateTime<Utc>` 未转换地写入这些字段。

### 2.2 枚举

以下枚举均为 `#[non_exhaustive]`，派生 `Debug + Clone + PartialEq + Eq + Hash + Serialize + Deserialize`，并使用 `#[serde(rename_all = "camelCase")]`：

- `OrderSide`: `Buy | Sell`
- `OrderType`: `Market | Limit | StopMarket | StopLimit`
- `OrderStatus`: `New | PartiallyFilled | Filled | Canceled | Rejected | Expired | PendingNew | PendingCancel | PendingReplace`
- `PositionDirection`: `Long | Short | Flat`
- `PositionStatus`: `Open | Closed | Liquidated`
- `ExecType`: `New | Canceled | Replaced | Rejected | Trade | Expired | TradeCancel | Status`

`TimeInForce` 同样为 `#[non_exhaustive]`，但使用显式 adjacently tagged serde：

```rust
#[serde(tag = "type", content = "value")]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtd(Timestamp),
}
```

`Gtd` 的截止时间是 UTC 毫秒时间戳，必须晚于创建时间。`PositionStatus` 当前是独立枚举，`Position` 尚未带 status 字段，调用方不得据此推断持仓实时状态；该缺口由 `DX-POS-001` 追踪。

### 2.3 交易结构体

实现必须与 `crates/domainx/src/lib.rs` 的字段名和类型一致：

- `Commission { amount: Decimal, asset: String }`
- `Order { order_id, instrument: String, side, order_type, status, price, stop_price, quantity, filled_quantity, remaining_quantity, avg_fill_price, time_in_force, created_at, updated_at, client_order_id }`
- `Position { position_id, instrument: String, direction, quantity, entry_price, current_price, unrealized_pnl, realized_pnl, created_at, updated_at }`
- `Trade { trade_id, order_id, instrument: String, side, price, quantity, commission, executed_at, is_maker }`
- `ExecutionReport { report_id, order_id, exec_type, order_status, instrument: String, side, order_type, price, quantity, last_filled_price, last_filled_quantity, cumulative_filled_quantity, remaining_quantity, commission, trade_id, reject_reason, occurred_at }`
- `Portfolio { portfolio_id, account_id, positions, total_unrealized_pnl, total_realized_pnl, total_commission, total_trades, updated_at }`

字段顺序不是协议契约；字段名、类型、Option 语义和时间单位是契约。组合手续费当前只有一个 `Decimal`，不能表达多资产汇总；调用方需要保留 `Trade.commission.asset`，该限制由 `DX-PORT-001` 追踪。

## 3. 不变量与校验边界

以下是目标数据不变量；crate 提供纯函数 `validate_order` 及细分校验（`ValidationError`），adapter/service 层应在写入前调用：

| ID | 不变量 | 失败动作 | 实现 |
|---|---|---|---|
| DX-VAL-001 | `quantity >= 0`，`filled_quantity >= 0`，`remaining_quantity >= 0` | 返回 `ValidationError::Quantity` | `validate_non_negative_quantities` |
| DX-VAL-002 | `filled_quantity + remaining_quantity == quantity`（交易所修正值需有证据） | 返回 `ValidationError::Quantity` | `validate_quantity_balance` |
| DX-VAL-003 | `price`/`stop_price` 只在对应订单类型允许时出现 | 返回 `ValidationError::Price` | `validate_order_prices` |
| DX-VAL-004 | `created_at <= updated_at`；事件时间未知时不能伪造当前时间 | 返回 `ValidationError::Time` | `validate_created_before_updated` |
| DX-VAL-005 | `Gtd` 截止时间不早于创建时间 | 返回 `ValidationError::Time` | `validate_gtd_deadline` |

## 4. 序列化契约

- 结构体字段使用 camelCase；未知字段默认由 serde 忽略，兼容性不能只依赖这一默认行为。
- 需要向后兼容的新增字段必须是 `Option<T>` 或带 `#[serde(default)]` 的字段；删除或改名属于 breaking change。
- `Decimal` 必须使用十进制精确表示；fixture 至少覆盖尾随零、负数和大数，不能转为 IEEE-754 浮点。
- 枚举的 serde 形式由源码属性决定；禁止在文档中写“默认 untagged”。每次改变 tag 或 variant 名称都要增加版本迁移说明。

## 5. 依赖与导出

| 依赖 | 用途 |
|---|---|
| `serde` + `derive` | 序列化派生 |
| `chrono` | workspace 兼容依赖；当前公共时间字段仍是 `Timestamp` |
| `rust_decimal` | 精确十进制 |

当前没有 `xhyper-canonical` 依赖；所有 instrument 迁移必须由 domainx、domain_market、domain_exchange 和五个 adapter 同步完成。

## 6. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| DX-API-001 | 公共符号、字段和 serde 派生与源码一致 | `cargo test -p domainx` + 编译 | verified |
| DX-API-002 | `TimeInForce`、Decimal、camelCase fixture round-trip | `crates/domainx/tests/serde_fixtures.rs` + `tests/fixtures/*.json` | verified |
| DX-VAL-001 | 数量和时间不变量（含 001–005） | `crates/domainx/src/validate.rs` 单元测试 | verified |
| DX-CAN-001 | instrument 从 String 迁移到唯一 canonical owner | ADR-001 + 迁移 PR + workspace 全量测试 | blocked |
| DX-COMP-001 | Position.status + Portfolio.commissions 多资产手续费 | `aggregate_commissions_by_asset` + `tests/comp_fixtures.rs` | verified |

门禁状态只表示当前证据强度。`blocked`/`pending` 项不得因本轮纯校验合入而静默标为 verified。

DX-CAN-001 规划见 `design/ADR-001-canonical-instrument.md`（依赖未入仓前保持 blocked）。
