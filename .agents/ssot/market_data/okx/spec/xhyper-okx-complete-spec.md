# OKX 公开行情适配器规格

**版本**：0.2.0
**状态**：协议契约冻结；Rust crate 仅有 DTO/config/连接参数与 trait skeleton；无 parse/seq/保活运行时
**实现**：`crates/exchange/okx/src/lib.rs`
**范围**：Spot、SWAP 永续、FUTURES 交割合约的公开行情；私有交易仅保留兼容入口
**最后核验**：2026-07-22

## 1. 官方事实与当前状态

事实来源是 [OKX V5 API 官方文档](https://app.okx.com/docs-v5/en/)。公开 WebSocket 不需要登录；连接若超过 30 秒没有数据需要在小于 30 秒的定时器触发时发送字符串 `ping`，并在没有 `pong` 时重连。订阅/取消订阅/登录请求还有连接级频率限制。

当前 `OkxConfig` 已有 REST、公有/私有 WS 地址、认证字段、重连次数、基础退避和 ping 参数，`OkxConnection` 也已建立；`OkxAdapter` 和所有 trait 方法仍是 skeleton。`reconnect_max_delay_ms`、消息解析、`seqId/prevSeqId` 连续性和重连恢复必须在实现后才可关闭门禁；当前官方文档已将 checksum 标为废弃，不得把旧 CRC32 算法列为实现要求。

## 2. 标的与频道

OKX 的原始标的使用 `instId`，示例为 `BTC-USDT`、`BTC-USDT-SWAP`、`BTC-USD-240927`。adapter 必须维护 `instId ↔ domain_market::InstrumentKey` 双向映射：

| OKX `instType` | 本仓 `ProductLine` | 备注 |
|---|---|---|
| `SPOT` | `Spot` | 现货 |
| `SWAP` | `Perpetual` | 结算/保证金币种另存 raw metadata |
| `FUTURES` | `Future` | 交割日期不得丢失 |

`OkxStream` 目标变体包括 `Tickers`、`Candle1m/5m`、`Trades`、`Books`、`Books5`、`Books50L2Tbt`。频道能力和深度上限以官方当前页面为准，不由名称猜测 snapshot/delta。

## 3. WebSocket 协议

订阅使用 `op=subscribe` 和 `args[{channel, instId}]`；响应有 subscribe/error 事件，推送包含 `arg`、`data` 和连接标识。实现必须保存订阅 ack，拒绝未确认订阅产生的业务数据。

| 频道 | 目标类型 | 必须保留 | 待确认行为 |
|---|---|---|---|
| `tickers` | `Quote` | `bidPx/bidSz/askPx/askSz/ts` | 空盘口处理 |
| `trades` | `Tick` | `tradeId/px/sz/side/ts` | side 语义映射 |
| `candle*` | `Bar` | interval、时间、OHLC、量、完成状态 | 数组下标与时间单位 |
| `books*` | `OrderBook` | `action`、levels、`seqId/prevSeqId`；checksum 仅作废弃兼容字段 | snapshot/update、重同步 |

订单簿不能按 `books` 这个名称静态标成 Snapshot。必须依据原始 `action` 和 sequence 处理首个快照、后续更新、跳号、重复和 sequence reset；当前 checksum 已废弃且固定为 0，不执行 checksum 失败恢复。sequence 失败时丢弃本地状态并重新订阅/拉 REST snapshot。

## 4. REST 目标

REST endpoint base 默认 `https://www.okx.com`，路径和返回 wrapper 必须以 V5 文档为准：public instruments、market candles、market trades、books snapshot。每个请求记录 `instType`、`instId`、limit、before/after 或时间窗口；`limit` 不能被误称为分页游标。

`OkxResponse<T> { code, msg, data }` 只是当前 skeleton DTO。`code != "0"` 必须映射 provider error；`data` 为空、字段缺失、精度解析失败分别进入 `Parse` 或 `InvalidRequest`。

## 5. 登录、心跳与重连

公有行情不需要登录；私有行情/交易需要 API key、passphrase、时间戳和签名。认证失败不能无限重试，密钥不进入日志、fixture 或 debug。

目标重连配置需要补充 `reconnect_max_delay_ms` 和 jitter 语义：`min(base*2^attempt+jitter,max)`。重连后重新登录私有连接、等待订阅 ack、对 books 强制重新 snapshot，并清理旧 sequence 状态；废弃 checksum 只可作为原始兼容字段记录。ping 定时器必须在收到任意消息后重置；pong 超时转为 `WebSocket`。

## 6. 错误、限频与分页

| 场景 | `AdapterError` | 动作 |
|---|---|---|
| `code != 0`/参数不合法 | `InvalidRequest` | 不重试 |
| 登录错误 | `Authentication` | 停止并要求重新认证 |
| 50011 或连接请求超限 | `RateLimit` | 依据官方窗口退避 |
| ping/pong/WS close | `WebSocket` | capped reconnect |
| JSON/Decimal/sequence shape 错 | `Parse` 或 `WebSocket` | 丢弃并按类型恢复 |

历史数据接口必须给出排序方向、时间窗口、最大 limit、before/after 的推进方式和去重 key。当前 trait 的 `Vec<T>` 只表示单页 skeleton。

## 7. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| OK-INS-001 | SPOT/SWAP/FUTURES instId 双向映射 | fixed instrument fixtures | pending |
| OK-WS-001 | subscribe ack/error/ticker/trades fixtures | mock WS | pending |
| OK-BOOK-001 | action/seq/prevSeqId/gap/reset resync | books fixtures + state machine | pending |
| OK-WS-002 | 30 秒保活、pong 超时、重连重订阅 | deterministic clock | pending |
| OK-REST-001 | instruments/candles/trades/books mock | HTTP fixtures | pending |
| OK-RATE-001 | connection/request rate limit 和 429/50011 | mock rate limiter | pending |
| OK-SKELETON-001 | 现有 skeleton 不误报 verified | source audit | specified |
