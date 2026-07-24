# coinbase 设计文档

## 当前实现状态

当前 crate 位于 `crates/exchange/coinbase`，manifest 中的 package 名为 `exchange-coinbase`，lib 名为 `exchange_coinbase`。`src/lib.rs` 已建立 Coinbase 频道、订阅、产品、配置、连接状态及 `VenueAdapter` 接口类型骨架；协议运行时尚未实现，不能据此认定 WebSocket 或 REST 已通过验证。

## 设计决策

### D1. Product ID 格式

Coinbase 使用 `{BaseAsset}-{QuoteAsset}` 格式作为产品标识（如 `BTC-USD`、`ETH-USD`），与 Binance 的 `{Base}{Quote}`（`BTCUSDT`）和 OKX 的 `{Base}-{Quote}`（`BTC-USDT`）均不同。适配器在实现 `InstrumentKey` 映射时需注意：

- Coinbase product_id 分隔符为 `-`
- Quote 资产为美元稳定币（`USD`、`USDC`）或 USDT
- `InstrumentKey.symbol` 存储 `{BaseAsset}-{QuoteAsset}` 原始值，不做额外转换

### D2. Level2 Snapshot + Update 模型

Coinbase Level2 深度数据模型：

1. **初始 snapshot**: 订阅 `level2` 频道后，服务端立即推送 `type: "snapshot"` 消息，包含全量 bids/asks
2. **增量更新**: 之后持续推送 `type: "l2update"` 消息，包含变更的 bids/asks
3. **序列元数据边界**: 消息 envelope 可能携带 `sequence_num`，但不能仅凭字段名假定它是每簿严格 `+1` 位点；Advanced Trade `level2` 官方文档强调保证更新交付。作用域、是否可用于 gap 检测必须由 raw fixture 固定；连接/订阅丢失时丢弃本地状态并重建
4. **价格精度**: Coinbase 深度价格使用 decimal 字符串表示，适配器在映射时转换为 `Decimal`

不涉及 `lastUpdateId` 或 checksum 校验机制（与 Binance/OKX 不同）；不能从旧 Coinbase feed 协议引入未经证实的 sequence 算法。

### D3. REST 分页

Coinbase Advanced Trade API 使用 cursor 分页。`GET /api/v3/brokerage/products` 等端点返回：

```json
{
    "products": [...],
    "num_products": 100,
    "has_next": true,
    "cursor": "abc123"
}
```

适配器 REST 客户端需实现 cursor 分页遍历：

```rust
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub has_next: bool,
    pub cursor: Option<String>,
}
```

### D4. 单一产品线

Coinbase Advanced Trade API 仅覆盖 Spot 产品线，无需 `ProductLine` 路由。适配器配置中不包含 `product_line` 字段，所有请求默认路由至 Spot 端点。

## 当前与后续 crate 布局

当前实现仍是单文件类型骨架：

```
crates/exchange/coinbase/
├── Cargo.toml
└── src/
    └── lib.rs            # 当前类型骨架与 VenueAdapter 实现骨架
```

以下为协议运行时实现后的拆分计划，相关文件目前尚未建立：

### 后续拆分计划（待实现）

```
crates/exchange/coinbase/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── config.rs
│   ├── types.rs           # 特有类型（CandleGranularity 等）
│   ├── ws/
│   │   ├── mod.rs
│   │   ├── channel.rs     # 频道类型定义
│   │   └── client.rs      # WebSocket 客户端
│   ├── rest/
│   │   ├── mod.rs
│   │   ├── client.rs      # REST API 客户端
│   │   └── pagination.rs  # Cursor 分页
│   └── mapping/
│       ├── mod.rs
│       └── coinbase.rs    # Coinbase → 域模型映射
└── tests/
    └── integration.rs
```
