# domain_exchange

L0 交易所抽象层：VenueAdapter trait、StreamType、OrderAmend、AccountInfo。

## 职责

- 定义交易所适配器的领域 trait 契约（`VenueAdapter`）
- 交易所事件类型（StreamType、ExecutionReport、InstrumentMeta）
- 订单修改（OrderAmend）和账户信息（AccountInfo）类型
- `AdapterError` 统一错误枚举

## 非目标

- 不包含具体交易所实现（由 L2 `exchange/{binance,okx,...}` 负责）
- 不实现 `contracts::Exchange` trait（由 L2' `adapters/exchange/*` 负责）
- 不依赖 I/O crate

## 分层共存

本 crate 属于 **L0 类型层**（`core/` 平面）。

- L2 Provider（`exchange/*`）实现本 crate 的 `VenueAdapter` trait
- L2' Infra Adapter（`adapters/exchange/*`）实现 `contracts::Exchange` trait——**两套独立契约，无桥接层**

详见 [docs/ssot/core-ssot-alignment.md](../../docs/ssot/core-ssot-alignment.md)。

## 规格

SSOT：`.agents/ssot/core/domain_exchange/spec/spec.md`
