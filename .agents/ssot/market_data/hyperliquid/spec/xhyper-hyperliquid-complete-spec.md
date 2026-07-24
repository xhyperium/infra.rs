# Hyperliquid 公开行情适配器规格

**版本**：0.2.0
**状态**：协议契约冻结；Rust crate 仅有 DTO/config 与 trait skeleton；订阅/全量簿/meta 映射未实现
**实现**：`crates/exchange/hyperliquid/src/lib.rs`
**范围**：公开 Perpetual/Spot Info API 与 WebSocket；交易签名不在本适配器范围
**最后核验**：2026-07-22

## 1. 官方事实与纠偏

事实来源是 [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)、[WebSocket 总览](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket) 和 [订阅文档](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)。公开请求使用 `POST https://api.hyperliquid.xyz/info`；主网 WebSocket 是 `wss://api.hyperliquid.xyz/ws`。公开自动化客户端必须处理服务端断线并优雅重连。

原有“InfoController JSON-RPC”表述不准确。当前订阅请求是：

```json
{
  "method": "subscribe",
  "subscription": { "type": "l2Book", "coin": "BTC" }
}
```

`allMids` 只提供 mid price，不能映射为要求独立 bid/ask/quantity 的 `Quote`；本规格将其定义为中间价原始数据，待 `MidPrice` 类型加入域模型后再接入统一事件。

当前 crate 的 `CandleSnapshotReq` 使用 `start_time/end_time`，但官方请求字段为 `startTime/endTime`；`HyperliquidStream` 的 `allMids → Quote` 注释和 `webbook2` 目标也不应被视为已支持。所有 trait 方法仍返回 skeleton `Internal`。

## 2. 标的与产品线

Info 文档说明 perpetual 与 spot 均可用：perpetual 的 `coin` 来自 `meta`；spot 使用 spot meta 的 pair 标识，例如 `PURR/USDC` 或 `@<index>`。adapter 不得把所有 coin 固定成一个 symbol 格式。

| 来源 | 本仓映射 | 规则 |
|---|---|---|
| perp `meta.universe[].name` | `InstrumentKey { exchange: "hyperliquid", symbol: coin }` | `ProductLine::Perpetual` |
| spot `spotMeta.universe` | 同上 | `ProductLine::Spot`；保留 pair/index raw metadata |

`coin ↔ InstrumentKey` 映射表必须带 dex/spot 维度，不能只用 coin 字符串作为全局 key。

## 3. WebSocket 频道

`HyperliquidStream` 目标覆盖 `AllMids`、`L2Book { coin }`、`Trades { coin }`。`WebBook2` 暂从可验收范围移除：当前官方公开文档未提供足够的 wire schema，不能把未定义的格式列作 P1 门禁。

| 订阅 | 目标类型 | 约束 |
|---|---|---|
| `{type:"allMids"}` | `MidPrice`（待建） | 不生成 Quote，不伪造数量 |
| `{type:"l2Book", coin}` | `OrderBook` 全量 snapshot update（待 fixture） | `WsBook` 每条消息整簿替换；REST depth 上限以官方页面和 fixture 为准 |
| `{type:"trades", coin}` | `Tick` | price/size/time 精确转换 |

实现必须解析 subscription ack、错误消息和 data envelope；不能把官方未声明的 sequence/checksum 当作事实。每条合法 `WsBook` 消息执行 full refresh；断线后使用 subscription snapshot ack 或 REST `l2Book` 重新 bootstrap，并在 evidence 说明恢复边界。

## 4. REST Info API

所有 Info 查询都是 POST `/info`，body 的 `type` 决定 schema：

| request type | 结果 | 事实与边界 |
|---|---|---|
| `allMids` | coin→mid map | 空 book 时可能使用 last trade fallback；不是 Quote |
| `l2Book` | L2 snapshot | `WsBook` 是 full refresh；REST levels、nSigFigs/mantissa 约束以官方文档为准 |
| `recentTrades` | trades | 逐项映射为 Tick；时间单位必须 fixture 固定 |
| `meta`/`spotMeta` | universe metadata | 建立 perp/spot 双向映射 |
| `candleSnapshot` | candle array→Bar | 最近可用 5000 根；interval 只能使用官方支持值 |

`CandleSnapshotReq` 目标 wire shape 为：

```json
{
  "coin": "BTC",
  "interval": "15m",
  "startTime": 1700000000000,
  "endTime": 1700003600000
}
```

请求 DTO 必须用 `#[serde(rename = "startTime")]`/`endTime` 或等价序列化测试。时间范围超过 provider 上限时按最后返回时间推进，并以 provider id/time 去重。

## 5. 连接与错误

`HyperliquidConfig` 字段为 REST/WS 地址、最大重连次数、基础/最大退避。目标算法为 capped exponential backoff + jitter；重连后恢复全部订阅，订单簿以 snapshot ack 或 REST snapshot 重新 bootstrap，不应用未经证明连续性的增量。

| 场景 | 错误 | 动作 |
|---|---|---|
| coin/interval/body 无效 | `InvalidRequest` | 不重试 |
| HTTP/WebSocket 断开 | `Network`/`WebSocket` | capped reconnect |
| JSON schema/Decimal/未知 candle key | `Parse` | 丢弃并记录 schema version |
| 公共 endpoint 限制 | `RateLimit` | 退避，不忙等 |
| 私有签名被误调用 | `Authentication`/`InvalidRequest` | 本适配器不执行 |

## 6. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| HL-MAP-001 | perp/spot meta coin 双向映射 | official JSON fixtures | pending |
| HL-MID-001 | allMids 不伪装 Quote，MidPrice 边界明确 | negative mapping test | specified |
| HL-WS-001 | 正确 `method/subscribe/subscription`、ack/error | mock WS fixtures | pending |
| HL-BOOK-001 | l2Book full refresh、levels 边界、恢复流程 | REST/WS fixtures | pending |
| HL-CANDLE-001 | `startTime/endTime` 序列化和 5000 上限分页 | request + pagination tests | pending |
| HL-RECON-001 | 断线、退避、订阅恢复、重复去重 | deterministic state machine | pending |
| HL-WEBBOOK2-001 | webbook2 仅在官方 schema 可追溯后重新纳入 | official evidence required | deferred |
| HL-SKELETON-001 | 当前 skeleton 不误报 verified | source audit | specified |
