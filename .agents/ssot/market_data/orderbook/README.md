# orderbook — 通用订单簿内核与物化引擎

**SSOT 根**：`.agents/ssot/orderbook/`
**规格**：`spec/spec.md`
**输入草稿**：`.cargo/draft/orderbook.md`、`.cargo/draft/orderbook/1.md`
**当前实现**：本 workspace 尚无对应 `orderbook` crate 或 service；`crates/domain_market` 只提供 `OrderBook` 公共形状和纯校验。
**当前状态**：目标契约已收敛；运行时、适配器、外部基础设施和真实流回放均待实现。

## 主题边界

本主题定义交易所无关的订单簿事件、同步模型、状态机、适配器 SPI、质量检查和统一输出。Binance、OKX、Coinbase、Hyperliquid 的 wire DTO 与连接管理仍归各自 adapter SSOT；公共 `OrderBook` 类型仍归 `domain_market`。

`.cargo/draft/orderbook.md` 的 Binance 专用 `orderbook-engine` v1 作为模型 A 的部署 profile 保留；`.cargo/draft/orderbook/1.md` 的多交易所 `orderbook-core` v2 是本主题的主契约。

## 入口

- 目标与验收：[`goal/goal.md`](goal/goal.md)
- 分层与取舍：[`design/design.md`](design/design.md)
- 可执行规格与门禁：[`spec/spec.md`](spec/spec.md)
- 事实与实现边界：[`evidence/README.md`](evidence/README.md)
- 门禁与追溯：[`matrix/README.md`](matrix/README.md)
- 十轮复审：[`review/round-01-10.md`](review/round-01-10.md)

