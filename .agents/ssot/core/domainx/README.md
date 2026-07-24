# domainx — 领域共享值对象

**SSOT 根**：`.agents/ssot/domainx/`
**规格**：`spec/spec.md`
**crate 路径**：`crates/domainx`
**package/lib**：`domainx`
**当前状态**：类型、纯校验和 serde/组合 fixture 已建立；instrument canonical 迁移待实现。

## 概述

domainx 定义全仓库共享的领域值对象与枚举，不依赖业务 crate。它提供订单、持仓、成交、执行回报、组合及共享时间/精度类型，不执行网络请求、下单、风控或 PnL 计算。

## 关键职责

- 订单值对象：Order、OrderSide、OrderType、OrderStatus、TimeInForce
- 仓位值对象：Position、PositionDirection、PositionStatus
- 成交值对象：Trade、ExecutionReport
- 组合值对象：Portfolio
- 通用枚举、Decimal 与 Timestamp 类型别名

## 落地状态

- **Status**：契约冻结 — 类型、纯校验和组合扩展已存在，canonical 迁移待实现
- **crate**：`crates/domainx`
- **门禁**：见 `spec/spec.md` 的 `DX-*` 表
