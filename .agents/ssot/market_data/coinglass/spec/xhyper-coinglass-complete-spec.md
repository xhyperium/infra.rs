# CoinGlass API V4 公开数据适配器规格

**版本**：0.2.0
**状态**：协议契约冻结；REST DTO/config 默认 V4 base、REST-only 负能力已 verified；schema/header 映射与 REST 数据方法仍待实现
**实现**：`crates/exchange/coinglass/src/lib.rs`
**范围**：公开可购买/可授权的聚合行情数据；无 WebSocket、账户或下单
**最后核验**：2026-07-23

## 1. 官方事实与认证

CoinGlass 的数据是公开市场数据，但当前 API 请求需要认证 header `CG-API-KEY`；不能把“公开数据”写成“匿名 API”。事实来源是[官方 API V4 入门](https://docs.coinglass.com/reference/getting-started-with-your-api)、[认证说明](https://docs.coinglass.com/reference/authentication)和[endpoint 总览](https://docs.coinglass.com/reference/endpoint-overview)。V4 base URL 是 `https://open-api-v4.coinglass.com`。

当前 crate 默认与 `new`/`from_config` 使用 V4 base `https://open-api-v4.coinglass.com`。`CoinglassResponse<T>` 仍固定 `data: Vec<T>`（对象/数组包装待改）；请求 header、限频、字段映射与 REST 数据方法仍 pending。

## 2. 数据模型归属

聚合结果使用 `domain_market` 的唯一公共类型：`OpenInterestPoint`、`FundingRatePoint`、`LiquidationData`、`LongShortRatioData`。Coinglass adapter 只定义 wire DTO 和转换函数，不重新导出同名业务类型。

`source=Coinglass` 表示数据供应商；结果中的 `exchange` 表示被聚合的交易所。`coin`/`symbol` 不能直接当作 `InstrumentKey`，必须通过支持交易所/交易对 endpoint 建立映射。

## 3. V4 endpoint 与 wire 事实

| endpoint | 目标数据 | 已核验事实 |
|---|---|---|
| `GET /api/futures/supported-exchange-pairs` | 符号/交易所映射 | 返回按 exchange 分组的 `instrument_id`、base/quote、settlement、tick size 等对象；缓存更新频率需按官方页面记录 |
| `GET /api/futures/open-interest/history` | OI OHLC | `time` 为毫秒；open/high/low/close 是 OI 值，官方样例为字符串 |
| `GET /api/futures/liquidation/history` | `LiquidationData` | `long_liquidation_usd`、`short_liquidation_usd` 为 USD；`limit` 默认/最大和 interval 以页面为准 |
| `GET /api/futures/open-interest/exchange-history-chart` | 跨交易所 OI chart | `time_list`、`price_list`、`data_map` 是对象形状，不能用 `Vec<T>` 泛化 |
| `GET /api/futures/funding-rate/history` | `FundingRatePoint` | 端点已列入官方总览；raw 字段、单位、interval 需 fixture 锁定 |
| `GET /api/futures/global-long-short-account-ratio/history` | `LongShortRatioData` | 端点已列入官方总览；account ratio 与 position ratio 不能混为同一指标 |

以上 endpoint 的 query 参数必须逐项记录 `exchange`、`symbol`、`interval`、`limit`、`start_time`、`end_time`、`range`、`unit` 的适用范围。不要把 option OI chart、futures pair OI 和 coin aggregated OI 互相替换。

## 4. 字段、单位和缺失值

| raw 语义 | domain 类型 | 单位/转换 | 缺失策略 |
|---|---|---|---|
| OI quantity | `OpenInterestPoint.oi` | contracts；不乘价格 | parse error 或 None，不填 0 |
| OI USD | `OpenInterestPoint.oi_value` | USD notional | 必须有 raw unit |
| funding rate | `FundingRatePoint.rate` | signed decimal；禁止百分比/小数静默换算 | schema evidence required |
| liquidation long/short | `LiquidationData.amount` + side | USD；side 由 raw long/short key 生成 | 不可识别 side 拒绝 |
| long/short ratio | `LongShortRatioData` | 明确 0–1 或 0–100；同一字段不能混用 | unit test required |
| time/time_list | `Timestamp` | Unix milliseconds | 秒/微秒输入拒绝或显式转换 |

所有价格/数量/费率使用 Decimal。官方样例中数值可能是 number 或 string，wire DTO 必须允许其文档化的形式，并在映射层统一成 Decimal。

## 5. 响应、认证、分页和限频

目标响应 wrapper 必须表达 endpoint-specific `T`：

```rust
struct CoinglassResponse<T> {
    code: String,
    msg: String,
    data: T,
}
```

`code == "0"` 才是成功；`data` 可以是数组或对象。HTTP 401/403、非零 code、HTTP 429、超时、空 data 和 decode failure 必须分别落到 `AdapterError` 映射表。

API key 只从外部 secret store 读取，通过 `CG-API-KEY` header 发送；不进入 `Debug`/fixture。认证页面还公开 `API-KEY-MAX-LIMIT` 和 `API-KEY-USE-LIMIT` response headers，限频器应据此动态观测，而不是硬编码旧的 30/min 或 200/min。

历史数据必须按官方 `limit`/time window 分页。对于 `time_list`/`data_map`，按时间索引对齐并校验数组长度；重试后用 `(endpoint, exchange, symbol, timestamp, metric)` 去重。`RateLimitConfig` 的 `burst_size` 只有在真正实现令牌桶或排队后才可关闭门禁。

## 6. 能力边界

CoinGlass 是 REST-only：不维护行情 WebSocket、heartbeat 或重连订阅；不提供本仓交易下单/账户功能。当前为了 workspace 复用 `VenueAdapter`，不适用方法必须返回未来的 `Unsupported` 错误，不得返回含糊的 `Internal` 并声称功能存在。后续优先拆出 `PublicMarketDataSource` trait。

## 7. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| CG-URL-001 | base URL、V4 endpoint、CG-API-KEY header 正确 | official URL + request fixture | pending |
| CG-SCHEMA-001 | object/array response wrapper 与 code/msg/error | success/400/401/429 fixtures | pending |
| CG-MAP-001 | OI/funding/liquidation/ratio 字段和单位映射 | raw JSON + Decimal tests | pending |
| CG-INST-001 | supported pairs 到 InstrumentKey 双向映射 | cache fixture + collision tests | pending |
| CG-PAGE-001 | limit/time window、time_list 对齐、重复去重 | deterministic pagination tests | pending |
| CG-RATE-001 | response limit headers、429 Retry-After、burst/timeout | mock limiter + clock | pending |
| CG-REST-001 | 无 WS/交易能力的 negative tests | `rest_only_ws_and_trading_return_unsupported` | verified |
| CG-SKELETON-001 | 当前 skeleton 不误报 verified | source audit | specified |
