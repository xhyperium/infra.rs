# orderbook 证据入口

## 本地来源

| 来源 | 用途 | 结论 |
|---|---|---|
| `.cargo/draft/orderbook.md` | Binance 专用 engine v1 | 提供 12 簿 profile、Spot/UM/CM 对齐、物化与自检要求 |
| `.cargo/draft/orderbook/1.md` | 多交易所 core v2 | 提供三种同步模型、SPI、统一输出与契约测试要求 |
| `crates/domain_market/src/lib.rs` | 公共类型 | 当前 `OrderBook`/`PriceLevel`/`InstrumentKey` owner |
| `crates/domain_market/src/book.rs` | 当前纯检查 | 排序、update id、毫秒启发式；无 provider 恢复状态机 |
| `.agents/ssot/domain_market/spec/spec.md` | 跨主题裁决 | provider-specific checksum/恢复不下沉到 domain_market |

## 外部协议入口

- Binance depth streams：[官方 Spot WebSocket Streams](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md)
- OKX：[官方 V5 API](https://app.okx.com/docs-v5/en/)
- Coinbase：[Advanced Trade level2 channel](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/websocket/websocket-channels)
- Hyperliquid：[Info API](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint) 与 [WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)

外部链接是核验入口，不等于本仓已保存 wire fixture。没有固定 fixture 和可重复命令时，门禁保持 `pending`/`specified`。

**2026-07-23 协议纠偏**：当前 [OKX V5 文档](https://app.okx.com/docs-v5/en/) 的 order book 说明将 `checksum` 标记为 deprecated、固定为 `0`，并要求使用 `seqId/prevSeqId` 检查连续性。因此 draft 中“OKX CRC32 checksum 必须校验”的旧要求被外部事实覆盖，已降为 `OB-OK-002` 的“不误用旧算法”负向门禁。

同日核验的 [Coinbase Advanced Trade level2 文档](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/websocket/websocket-channels) 明确 `new_quantity` 是绝对量、`0` 表示删除，并强调该频道保证更新交付；文档示例的 envelope `sequence_num` 不能直接推导为每簿严格递增。因此 `OB-CB-001` 要求先固定 sequence 作用域，未固定前不启用严格 gap 算法。

同日核验的 [Binance Spot depth 文档](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md) 明确 snapshot `lastUpdateId`、事件 `U/u`、丢弃 `u <= lastUpdateId`、首事件区间匹配和后续 `U == previous_u + 1`；因此模型 A 的 Spot 规则有官方事实入口。UM/CM 的 `pu` 规则仍须由对应 futures 原始 fixture 另行固定。

同日核验的 [Hyperliquid WebSocket 文档](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket) 与 [subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions) 将 `l2Book` 定义为 `WsBook` snapshot updates，并要求断线优雅重连；重连期间遗漏的数据由 snapshot ack/对应 Info 请求补齐。因此模型 C 虽无 sequence continuity，仍必须实现连接恢复后的整簿 bootstrap。
