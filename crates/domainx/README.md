# domainx

L0 共享交易值对象层：Order、Position、Trade、Portfolio 及共享枚举。

## 职责

- 交易领域跨子系统共享的数据结构（纯类型 + 序列化边界）
- 订单 / 持仓 / 成交 / 执行回报 / 组合的值对象定义
- `validate` 模块提供纯函数校验（数量非负、时间序、GTD 截止等）

## 非目标

- 不执行下单、风控、网络请求、持久化或 PnL 计算
- 不依赖 I/O crate（仅 serde / chrono / rust_decimal / thiserror）

## 分层共存

本 crate 属于 **L0 类型层**（`core/` 平面），与 `market_data`（L1 管线）独立共存。

详见 [docs/ssot/core-ssot-alignment.md](../../docs/ssot/core-ssot-alignment.md)。

## 规格

SSOT：`.agents/ssot/core/domainx/spec/spec.md`
