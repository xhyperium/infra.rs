# hyperliquid 设计文档

## 当前实现状态

当前 crate 位于 `crates/exchange/hyperliquid`，manifest 中的 package 名为 `exchange-hyperliquid`，lib 名为 `exchange_hyperliquid`。`src/lib.rs` 已建立 WebSocket 流、Info 请求/响应、K 线周期、币种元数据、配置、连接状态及 `VenueAdapter` 接口类型骨架；协议运行时尚未实现，不能据此认定 WebSocket 或 REST 已通过验证。

## 设计决策

### D1. coin → InstrumentKey 映射

Hyperliquid 的 Info API 同时覆盖 perpetual 与 spot。perp 使用 meta universe 的 coin，spot 使用 spot meta 的 pair/index 标识。映射到本仓 `InstrumentKey` 时：

- perp 的 `product_line` 为 `Perpetual`，spot 的 `product_line` 为 `Spot`
- `symbol` 使用 meta 返回的完整标识，并保留 dex/spot index raw metadata

适配器需从 `meta` 接口获取所有交易对列表，构建 `coin ↔ InstrumentKey` 双向映射表。

### D2. JSON-RPC 风格 API

Hyperliquid 使用自定义 Info/WebSocket 协议，不应称为 JSON-RPC：
- Info 查询统一使用 `POST /info`，请求体为 `{"type": "...", ...}`
- WebSocket 订阅使用 `{"method": "subscribe", "subscription": {"type": "...", "coin": "..."}}`
- 适配器需封装此协议差异，对调用方提供 Rust 原生接口

### D3. REST 订单簿快照作为恢复边界

Hyperliquid 的 `l2Book` REST 接口返回当前订单簿快照；WebSocket `WsBook` 文档定义为按 block 推送的 snapshot update，而不是可按 diff 合并的增量。WebSocket 是否携带可用序列号必须由 fixture 证明。因此：

- WebSocket 每条合法 `WsBook` 都执行整簿 `ReplaceAll`；不能保留上一条消息中已消失的档位
- REST `l2Book` 用作显式查询或连接恢复 fallback；重连后的 snapshot ack/Info snapshot 是新的 bootstrap 边界
- 不把未声明的 sequence/checksum 当作事实；若消息解析失败或断线，清理本地状态并重新 bootstrap

## 当前与后续 crate 布局

当前实现仍是单文件类型骨架：

```
crates/exchange/hyperliquid/
├── Cargo.toml
└── src/
    └── lib.rs            # 当前类型骨架与 VenueAdapter 实现骨架
```

以下为协议运行时实现后的拆分计划，相关文件目前尚未建立：

### 后续拆分计划（待实现）

```
crates/exchange/hyperliquid/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── types.rs          # 特有类型（Coin、InfoRequest 等）
│   ├── ws/
│   │   ├── mod.rs
│   │   ├── stream.rs     # 流类型定义
│   │   └── client.rs     # WebSocket 客户端
│   ├── rest/
│   │   ├── mod.rs
│   │   └── client.rs     # REST API 客户端
│   └── mapping/
│       ├── mod.rs
│       └── hyperliquid.rs # Hyperliquid → 域模型映射
└── tests/
    └── integration.rs
```
