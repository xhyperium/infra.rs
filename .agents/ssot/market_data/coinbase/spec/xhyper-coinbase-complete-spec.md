# Coinbase Advanced Trade 公开行情适配器规格

**版本**：0.2.0
**状态**：协议契约冻结；DTO/config 与 trait skeleton；`new`/`from_config` 与 Advanced Trade endpoint 对齐（CB-URL-001 verified）；VenueAdapter 运行时仍 skeleton
**实现**：`crates/exchange/coinbase/src/lib.rs`
**范围**：Spot 公开行情；`user`/交易接口只保留边界说明
**最后核验**：2026-07-23

## 1. 官方事实与状态

事实来源是 [WebSocket 频道文档](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/websocket/websocket-channels) 和 [WebSocket 总览](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/websocket/websocket-overview)。市场数据 endpoint 是 `wss://advanced-trade-ws.coinbase.com`；公开频道包括 `heartbeats`、`candles`、`ticker`、`ticker_batch`、`level2`、`market_trades`、`status`。官方说明 heartbeat 每秒发送，订阅 heartbeats 可在低流量时保持连接；每条 subscription message 只能指定一个 channel。

当前 `CoinbaseConfig::default` 与 `CoinbaseAdapter::new`/`from_config` 均使用 Advanced Trade endpoint（CB-URL-001）。`CoinbaseChannel` 仍缺少 `MarketTrades`；所有 trait 方法仍返回 skeleton `Internal`。

## 2. 标的、频道与认证

Coinbase Product ID 使用 `BASE-QUOTE`（如 `BTC-USD`）。当前适配器本地 `Product` 只负责 REST DTO，不属于 `domain_market`；映射到 `InstrumentKey { exchange: "coinbase", symbol: product_id }` 时必须保留原始 ID。

| 频道 | 目标类型 | 认证 | 关键契约 |
|---|---|---|---|
| `heartbeats` | connection signal | 公开 | `heartbeat_counter` 可用于检测缺失消息 |
| `ticker`/`ticker_batch` | `Quote`/last-price DTO | 公开 | 不把 last price 当 bid/ask |
| `market_trades` | `Tick` | 公开 | 单独保留 trade id、price、size、time |
| `level2` | `OrderBook` | 公开 | snapshot/update、绝对量更新与原始 sequence 元数据必须保留；严格连续性待 fixture 证明 |
| `candles` | `Bar` | 公开 | granularity、时间边界、未闭合状态 |
| `user` | 交易账户事件 | JWT | 本公共行情适配器不实现 |

官方公开频道可不带 JWT；私有 user channel 需要 CDP JWT。JWT 不能写入 repository、fixture、日志或 PR。

## 3. WebSocket 消息与恢复

订阅消息使用 `type=subscribe`、`channel`、`product_ids`，每个 channel 一条消息；收到 subscribe ack 后才进入 subscribed 状态。取消订阅使用相同 channel/product 语义。

| 输入 | 输出 | 不变量 |
|---|---|---|
| ticker bid/ask 字段 | `Quote` | bid≤ask；缺失一侧拒绝为 Quote |
| market trade | `Tick` | quantity、price 用 Decimal；时间转 Unix ms |
| level2 snapshot/update | `OrderBook` | 官方保证更新交付；连接/订阅恢复时重建；不得假定 envelope sequence 严格每簿递增 |
| candle | `Bar` | open/close 时间以原始字段为准 |
| heartbeat | 不向市场域发 fake quote | counter 单调性/缺失由连接层处理 |

连接若在规定时间内没有 subscribe 会被断开；重连使用 capped exponential backoff，恢复后按 channel 分条订阅。level2 的 snapshot/update、原始 sequence 作用域、重复和连接恢复规则必须用官方 raw fixture 固定，不能从旧 feed 协议猜测严格 gap 算法。

## 4. REST API

REST base 默认 `https://api.coinbase.com`，目标使用 Advanced Trade `/api/v3/brokerage` 资源：[官方 endpoint 文档](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/rest-api)。公开 market-data endpoint 至少包括：

| endpoint | 本仓 DTO | 分页/时间 |
|---|---|---|
| `GET /products` | `Vec<Product>` | cursor；必须返回 next cursor 状态 |
| `GET /products/{product_id}` | `Product` | 单资源 |
| `GET /products/{product_id}/candles` | `Vec<Bar>` | start/end/granularity；按时间排序去重 |
| `GET /products/{product_id}/ticker` | ticker/Quote | 单次快照 |
| `GET /products/{product_id}/book` | `OrderBook` | level 与 sequence 语义必须固定 |

REST public 与 private authentication、HTTP status、错误 body 和 rate limit 必须分开记录。分页 cursor 是 opaque；重试不得重复消费已确认页，去重 key 为 product + provider id/time。

## 5. 配置与错误

`CoinbaseConfig` 字段为 REST/WS 地址、API key/secret、重连次数、基础/最大退避。`CoinbaseAdapter::new` 必须改为使用 config，禁止直接构造另一套 URL。

| 场景 | 错误 | 动作 |
|---|---|---|
| product/channel 无效 | `InvalidRequest` | 不重试 |
| JWT/key 失效 | `Authentication` | 重新生成 JWT |
| HTTP/WS 限频 | `RateLimit` | 使用 provider window 退避 |
| socket close/heartbeat timeout | `WebSocket` | 重连并恢复订阅 |
| JSON 字段/Decimal/sequence shape 错误 | `Parse` | 丢弃并告警，level2 需重建 |

## 6. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| CB-URL-001 | config 与 adapter 使用同一 Advanced Trade endpoint | `cb_url_001_config_and_adapter_share_advanced_trade_endpoints` | verified |
| CB-WS-001 | heartbeat/ticker/market_trades/level2 subscription fixtures | mock WS | pending |
| CB-BOOK-001 | level2 snapshot/update、绝对量与连接恢复 | order book state/reconnect tests | pending |
| CB-REST-001 | products/product/candles/ticker/book fixtures | HTTP mock | pending |
| CB-PAGE-001 | cursor 遍历、重复页和终止条件 | deterministic pagination test | pending |
| CB-AUTH-001 | public/private channel auth boundary | no-secret auth tests | pending |
| CB-SKELETON-001 | 当前 skeleton 只计 compile evidence | source audit | specified |
