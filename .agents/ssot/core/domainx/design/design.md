# domainx 设计文档

## 当前实现状态

实现位于 `crates/domainx`（package/lib：`domainx`）。当前提供共享类型、纯订单校验和多资产手续费汇总；交易对象中的 `instrument: String` 仍是等待统一 canonical owner 的显式兼容占位。网络、下单、风控和 PnL 计算不属于本 crate。

## 设计决策

### D1. 纯值对象与纯不变量

domainx 仅包含数据载体和不依赖 I/O 的纯校验/汇总函数。所有类型均为 `struct` + `enum`，派生标准 trait；网络、下单、风控和 Position PnL 计算仍放在上层 domain/service。

### D2. InstrumentKey 迁移边界

domainx 不定义 `InstrumentKey`。当前交易对象暂用 `String`，待 canonical crate 真正进入 workspace 后再由一次迁移统一替换；在此之前禁止各 adapter 自行定义 instrument 类型。

### D3. OrderSide: Buy/Sell 而非 Bid/Ask

采用传统语义 `Buy`/`Sell`（对应 taker 视角），而非 `Bid`/`Ask`（对应挂单方向）。前者在交易执行上下文中更直观。

## crate 布局

```
crates/domainx/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── order.rs
│   ├── order_enums.rs
│   ├── position.rs
│   ├── trade.rs
│   ├── execution_report.rs
│   └── portfolio.rs
└── tests/
    └── roundtrip.rs
```
