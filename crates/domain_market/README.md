# domain_market

L0 行情域模型层：Tick、Quote、Bar、OrderBook 及 canonical 类型。

## 职责

- 市场数据领域类型定义（InstrumentKey、行情事件、订单簿）
- 时间验证（`validate_tick_time` / `validate_quote_time` / `validate_bar_time`）
- 订单簿验证（`validate_order_book` / `validate_level_ordering` / `validate_update_ids`）
- 依赖 `domainx` 共享类型（Decimal、OrderSide、Timestamp）

## 非目标

- 不依赖 I/O crate；不直接对接交易所 API（由 L2 `exchange/*` 负责）
- 不是行情处理管线（由 L1 `market_data` 负责）

## 分层共存

本 crate 属于 **L0 类型层**（`core/` 平面），与 `market_data`（L1 管线）独立共存。

详见 [docs/ssot/core-ssot-alignment.md](../../docs/ssot/core-ssot-alignment.md)。

## 规格

SSOT：`.agents/ssot/core/domain_market/spec/spec.md`
