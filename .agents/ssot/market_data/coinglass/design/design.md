# coinglass 设计文档

## 当前实现状态

当前 crate 位于 `crates/exchange/coinglass`，manifest 中的 package 名为 `exchange-coinglass`，lib 名为 `exchange_coinglass`。`src/lib.rs` 已建立 `CoinglassResponse<T>`、`CoinglassConfig`、`RateLimitConfig` 及 `VenueAdapter` 接口类型骨架；协议运行时尚未实现，不能据此认定 REST 已通过验证。Coinglass 的协议设计仍为 REST-only，不提供 WebSocket 流。

## 设计决策

### D1. REST-only 协议

Coinglass API V4 提供公开市场数据的 REST 接口，无 WebSocket 实时流。数据获取方式为定时轮询（pull），非实时推送（push）；API 请求仍需 `CG-API-KEY`。

影响：
- 数据新鲜度由 endpoint 的官方 cache/update frequency 决定，不能在没有证据时写固定分钟数
- 无需维护长连接、重连、心跳等 WebSocket 基础设施
- 实现简单，但实时性受限于轮询间隔

### D2. 跨交易所符号映射

Coinglass V4 的 supported-exchange-pairs 返回 `instrument_id`、base/quote、settlement 等字段；适配器必须以该响应构建映射，不能假设所有交易所都使用 `BTCUSDT`。

适配器需维护按 `ExchangeId + instrument_id` 索引的符号映射表，在首次调用或配置时从 V4 支持交易对接口构建。

### D3. 数据新鲜度策略

Coinglass 数据为聚合计算后的数据，非交易所原生逐笔数据：

| 数据类型 | 典型延迟 | 说明 |
|---------|---------|------|
| 指标 | 新鲜度规则 |
|---|---|
| 各 endpoint | 以官方页面列出的 cache/update frequency 为准 |

适配器不提供数据新鲜度担保，调用方应容忍 ~5 分钟的滞后。对实时性敏感的场景应直接使用交易所适配器。

### D4. API Key 与限频

API key 通过 `CG-API-KEY` header 发送；认证页面公开最大/当前使用限额 header。适配器应读取 provider 返回的限频信息，不能硬编码旧的匿名/有 key 配额。

## 当前与后续 crate 布局

当前实现仍是单文件类型骨架：

```
crates/exchange/coinglass/
├── Cargo.toml
└── src/
    └── lib.rs            # 当前类型骨架与 VenueAdapter 实现骨架
```

以下为协议运行时实现后的拆分计划，相关文件目前尚未建立：

### 后续拆分计划（待实现）

```
crates/exchange/coinglass/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config.rs          # CoinglassConfig、RateLimitConfig
│   ├── client.rs          # REST API 客户端（HTTP 请求层）
│   ├── types/
│   │   ├── mod.rs
│   │   ├── oi.rs          # OpenInterestPoint
│   │   ├── funding.rs     # FundingRatePoint
│   │   ├── liquidation.rs # LiquidationData
│   │   └── ratio.rs       # LongShortRatio
│   ├── api/
│   │   ├── mod.rs
│   │   ├── open_interest.rs
│   │   ├── funding_rate.rs
│   │   ├── liquidation.rs
│   │   ├── long_short_ratio.rs
│   │   └── top_trader_ratio.rs
│   └── mapping/
│       ├── mod.rs
│       └── coinglass.rs   # Coinglass → 域模型映射
└── tests/
    └── integration.rs
```
