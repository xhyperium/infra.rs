# SSOT 统一契约与治理基线

**版本**：0.3.0
**状态**：契约冻结；运行时实现仍按各主题门禁推进
**最后核验**：2026-07-23

本文是 `.agents/ssot/` 下十个主题的横向治理基线。主题 `spec/spec.md` 仍是该主题的详细 SSOT；本文件只裁决跨主题冲突、状态含义、证据格式和公共命名。

## 1. 主题与实现映射

| 主题            | 规格入口                                   | workspace 实现                | 当前状态                                     |
| --------------- | ------------------------------------------ | ----------------------------- | -------------------------------------------- |
| domainx         | `core/domainx/spec/spec.md`               | `crates/domainx`              | 类型骨架已存在；跨域 instrument 仍是兼容占位 |
| domain_market   | `core/domain_market/spec/spec.md`         | `crates/domain_market`        | 类型骨架已存在；Envelope 仍为 JSON 载荷      |
| domain_exchange | `core/domain_exchange/spec/spec.md`       | `crates/domain_exchange`      | trait/错误类型已存在；运行时行为待实现       |
| binance         | `binance/spec/spec.md`                     | `crates/exchange/binance`     | DTO/config 骨架；连接、映射和 REST 待实现    |
| okx             | `okx/spec/spec.md`                         | `crates/exchange/okx`         | DTO/config 骨架；连接、映射和 REST 待实现    |
| coinbase        | `coinbase/spec/spec.md`                    | `crates/exchange/coinbase`    | DTO/config 骨架；连接、映射和 REST 待实现    |
| hyperliquid     | `hyperliquid/spec/spec.md`                 | `crates/exchange/hyperliquid` | DTO/config 骨架；连接、映射和 REST 待实现    |
| coinglass       | `coinglass/spec/spec.md`                   | `crates/exchange/coinglass`   | REST DTO/config 骨架；请求、映射和限频待实现 |
| orderbook       | `orderbook/spec/spec.md`                   | 当前无对应 crate/service     | 通用内核契约已指定；Parser、状态机、物化和回放待实现 |
| market_data     | 兼容 API：`crates/market_data/docs/API.md` | `crates/market_data`          | L0 兼容 facade；不重复定义新域 SSOT          |

文档中的路径必须使用上表中的实际路径；历史 domain/adapters 目录布局不再用于新文档。

## 2. 冲突裁决顺序

发生冲突时按以下顺序处理，并在主题 `review/` 记录决议：

1. 外部供应商当前公开协议文档（只作为 wire fact，不决定本仓类型设计）。
2. 本文件的跨域契约。
3. 主题 `spec/spec.md` 的目标 API。
4. `design/`、`goal/` 和 `review/` 的解释与状态。
5. Rust 源码与测试：源码是当前实现证据，不会自动覆盖未决的目标契约。

如果源码与目标 spec 不一致，必须标为 `pending`，不能把编译通过当作运行时门禁通过。

## 3. 本仓公共语义

- `Timestamp` 是 Unix 毫秒的 `i64`，所有供应商输入必须在 adapter 边界转换；不在同一字段混用 `DateTime<Utc>` 与裸整数。
- `event_time`/`timestamp` 表示供应商事件或数据生成时间；`received_at` 表示本地收到完整报文的时间；`ingested_at` 表示进入管道的时间。没有供应商时间时不得伪造事件时间。
- `InstrumentKey` 当前由 `domain_market` 提供，字段为 `exchange + symbol`；`symbol` 保留供应商规范化后的标的字符串。跨交易所同名标的不能仅靠 symbol 去重。
- `ProductLine` 当前实现变体为 `Spot`、`Future`、`Perpetual`、`Option`。OKX `SWAP` 与 Hyperliquid perpetual 的映射必须在适配器 spec 明确，不得默认为 `Future`。
- 所有公共 serde 结构体使用 camelCase；除非源代码明确使用 tagged/untagged，文档不得声称“默认 untagged”。
- `Decimal` 以十进制字符串保存精度；每个 adapter 的原始字符串/数值转换都必须有 fixture 和单位说明。
- `MarketFactEnvelope` 当前是 `source + fact_type + JSON data + timestamp` 的兼容骨架；typed `MarketFact`、序列号和聚合数据信封属于后续目标，未实现前不得列为通过门禁。

## 4. Adapter 生命周期与错误

所有实现当前遵循 `domain_exchange::VenueAdapter` 的 13 个方法：连接、断开、三个公开行情订阅、下单、撤单、改单、单订单查询、未结订单查询、账户查询、标的元数据查询、订单簿快照查询。

- `connect`、`disconnect` 的幂等性、重连订阅恢复、并发调用和取消语义必须在 adapter spec 中单独写出。
- `AdapterError` 的当前公共变体为 `InvalidRequest`、`Authentication`、`RateLimit`、`Network`、`WebSocket`、`Parse`、`Internal`、`Unsupported`。供应商错误码映射表必须标明重试、重新认证或终止动作；不适用能力必须使用 `Unsupported`，不能伪装为 `Internal`。
- REST-only 数据源（例如 Coinglass）不应伪装成支持 WebSocket/交易；在当前兼容 trait 中这些方法必须标为 `unsupported/pending`，后续应拆出只读市场数据 trait。
- 任何分页接口必须说明游标/时间窗口、排序、最大页、终止条件和去重键。没有这些信息时，返回 `Vec<T>` 只能表示“单页骨架”。

## 5. 证据与门禁格式

每个 P0/P1 门禁使用唯一 ID，并至少包含：

```text
ID | requirement | spec lines | implementation | test/fixture | external source | status | rechecked_at
```

`status` 只能是 `specified`、`skeleton`、`pending`、`verified`、`blocked`。`specified` 不等于 `verified`；`verified` 必须有可重复命令、固定 fixture 或 mock 响应。API key、Token 和私密请求不得进入 evidence、日志、fixture、issue 或 PR。

## 6. 公开协议核验来源

以下链接是 2026-07-23 的事实核验入口；供应商变更后先更新 evidence，再修改主题 spec：

- Binance Spot WebSocket Streams：[官方文档仓库](https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md)
- OKX V5 API：[官方 API 文档](https://app.okx.com/docs-v5/en/)
- Coinbase Advanced Trade WebSocket：[官方频道文档](https://docs.cdp.coinbase.com/coinbase-business/advanced-trade-apis/websocket/websocket-channels)
- Hyperliquid Info API：[官方 Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)；[官方 WebSocket subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- CoinGlass API V4：[官方入门](https://docs.coinglass.com/reference/getting-started-with-your-api)；[官方 endpoint overview](https://docs.coinglass.com/reference/endpoint-overview)

## 7. 订单簿边界

- `domain_market` 负责 `OrderBook`/`PriceLevel` 公共形状和纯检查；`.agents/ssot/orderbook` 负责 provider-specific 同步状态机、适配器 SPI、恢复和统一物化。
- orderbook core 不得依赖 Binance/OKX/Coinbase/Hyperliquid adapter；provider sequence 与当前有效的 integrity 字段只能在 adapter verifier/rule 中解释，已废弃字段不得启用旧算法。
- `.cargo/draft/orderbook/1.md` v2 是通用内核主契约；`.cargo/draft/orderbook.md` v1 只作为 Binance service profile。

## 8. 变更要求

跨主题变更必须同时更新：受影响的 `spec.md`、对应 `design/goal`、追溯矩阵、证据来源和 review 决议。提交前运行 `git diff --check`、UTF-8/LF 检查、Markdown lint（若配置）以及仓库规定的 Cargo gates。
