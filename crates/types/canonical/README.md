# canonical

`/types/` 跨层共享 DTO（ADR-001，spec §4.2）。只放纯数据形状，无业务逻辑。

## 主要内容

- 复用 `decimalx::Money`（ADR-007）。
- 标识：`VenueId`、`InstrumentId`、`OrderRef`。
- 取消：`CancelOrderRequest`。
- 枚举：`OrderStatus` / `Side`。
- DTO：`Order` / `OrderAck` / `OrderBookSnapshot` / `Position` / `SymbolMeta` / `Tick` / `Trade` / `PriceLevel`。
- 辅助模块：`shape`（adapter 形状检查）、`proposed_time`（ns↔ms 转换）。
- **时间**：DTO `ts: i64` = Unix epoch **纳秒**（CAN-TIME-001 Approved）。
- **标识**：新接口优先 `OrderRef`；`OrderId` 类型已删除，id 字段为 `String`。

## 定位

`/types/` 层，跨 crate 共享的数据契约。`Money` / `Decimal` 族复用自 `decimalx`，不在本 crate 重定义。

权威当前实现合同：`.agents/ssot/types/canonical/canonical-spec.md`。  
完整规范（Approved，≠ package stable）：`.agents/ssot/types/canonical/20260717/`。

## 非职责

- 无业务行为方法（行为在 domain 层 newtype 上）。
- 不依赖 L1/适配器；不做 I/O。
- 不替代 `contracts` trait 出口。
- **不是**通用 Canonical Encoding Core / schema registry / hash-sign-evidence 框架（evidence 有独立 versioned encoding）。

## 限制与安全

- DTO 变更影响面广；仅 `CancelOrderRequest` / `OrderRef` 与 legacy `OrderAck` 有固定 JSON fixture/回归证据，其余 serde shape 默认视为实现细节，不自动等于跨版本 wire 承诺。
- 金额字段类型必须来自 `decimalx`，禁止浮点别名。
- `ts` **必须**按纳秒写；adapter 从交易所 ms 入口须经 `ns_from_unix_millis`。
- **生产路径**见 `.agents/ssot/types/canonical/plan/production-upgrade.md`。**≠** package stable / crates.io。

## 版本

0.1.0（见 `Cargo.toml`）。**≠** package stable · **≠** 全 wire Production Ready。
