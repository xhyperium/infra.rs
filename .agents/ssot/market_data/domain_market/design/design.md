# domain_market 设计文档

## 当前实现状态

实现位于 `crates/domain_market`（package/lib：`domain_market`）。当前提供行情结构、聚合点类型、兼容 JSON envelope 和并存的 typed fact/subject；协议映射、全量 typed 管道迁移和 canonical instrument 迁移尚未完成。

## 设计决策

### D1. Tick/Quote/Bar/OrderBook 四元组

市场数据的四种基本形态是行业共识（交易所 API、交易平台、数据库 schema 均以此为基础）。选择保留独立类型而非统一为 `MarketFact` 是为了：
- 类型安全（各类型字段完全不同）
- 序列化清晰（不同 topic/table）
- 消费者按需订阅

### D2. MarketFactEnvelope 分阶段演进

当前实现保留 `instrument + source + fact_type + JSON data + timestamp` 兼容 envelope，同时提供 typed fact/subject 与可选 sequence。全量管道切换和兼容 envelope 退役仍是后续目标，不把目标迁移写成现状。

### D3. Binance C/S 映射定位为语义规格

非代码实现。适配器在各自 package 中实现具体映射，本 spec 只定义语义对应关系和数据转换规则。

## crate 布局

```
crates/domain_market/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── tick.rs
│   ├── quote.rs
│   ├── bar.rs
│   ├── order_book.rs
│   ├── canonical.rs
│   └── envelope.rs
└── tests/
    └── binance_mapping.rs
```
