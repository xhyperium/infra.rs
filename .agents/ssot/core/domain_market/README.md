# domain_market — 市场数据域模型

**SSOT 根**：`.agents/ssot/domain_market/`
**规格**：`spec/spec.md`
**crate 路径**：`crates/domain_market`
**package/lib**：`domain_market`
**当前状态**：类型、纯时间/订单簿校验、serde fixture 和 typed envelope 已建立；协议映射与全量 typed 管道迁移待实现。

## 概述

市场数据域模型覆盖 Tick、Quote、Bar、OrderBook 及跨交易所聚合点类型，并提供 InstrumentKey、ProductLine、DataSource 和兼容 MarketFactEnvelope。交易所 wire DTO 和传输层不属于本域。

## 关键职责

- 行情数据的结构化表示
- provider symbol 到统一 instrument 的映射边界
- Snapshot/Delta、时间戳、精度和序列语义
- 聚合数据的来源/被聚合交易所双维度表达

## 落地状态

- **Status**：契约冻结 — 类型/纯校验/typed envelope 已存在，协议映射与 canonical 迁移待实现
- **crate**：`crates/domain_market`
- **门禁**：见 `spec/spec.md` 的 `DM-*` 表
