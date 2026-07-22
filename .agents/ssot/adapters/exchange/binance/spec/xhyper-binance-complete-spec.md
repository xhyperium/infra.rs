# binance 交易所适配器当前实现规范

> 状态：`binancex` `0.3.2` 已有签名 REST + 公共 WS 解析/注入的默认实现入口；**交易 NO-GO**，未宣称 package stable 或可真实交易。

## 1. 权威与定位

- 路径 / package：`crates/adapters/exchange/binance` / `binancex`。
- 实现 `contracts::VenueAdapter` 及 execution/market/instrument/account/time 能力 trait。
- 生产依赖包含 contracts/canonical/decimalx/kernel/transportx 和签名/序列化库；`publish = false`。
- 当前源码与测试是可观察实现证据；历史 mock/COMPLETE 叙事不能覆盖本文边界。

## 2. 可观察实现

| 能力 | 当前事实 |
|------|----------|
| 适配器 | `BinanceAdapter::{mainnet,testnet,new}`；显式注入 `HttpDriver`、`BinanceApiKey`、`WsConnector` |
| 认证 | HMAC-SHA256 query 签名、API key header；Debug 不暴露 secret |
| REST | server time；签名 place/cancel/query；账户/余额与 instrument 请求面；服务错误映射 |
| 公共 WS | bookTicker/trade/depth URL、fixture 解析与 connector 注入 |
| DTO | decimalx/canonical/contracts 类型；订单状态与行情报文解析 |
| 测试 | 签名向量、请求路径/header、4xx 业务错误、公共行情 fixture；live 仅受控 `server_time` ignore 入口 |

未注入 HTTP/凭据/WS 时，部分 trait 表面会成功返回内存占位或空流。该行为是已知 fail-open 风险：能力不可用时尚未统一 fail-closed，调用方不能据此判定订阅健康或交易能力可用。

## 3. 已实现不等于可交易

签名 REST 和公共 WS 只能证明协议入口、离线请求形状与解析面存在。当前证据没有闭合真实资金交易链路、订单生命周期连续性或故障恢复，因此不能把“默认实现”写成 Production Ready。

## 4. OPEN / 交易 NO-GO

以下交易安全条件仍 OPEN：

- 根据 exchangeInfo filters 做价格/数量/stepSize/minQty/minNotional **精度**量化与下单前拒绝；
- canonical percent-encoding、query 参数排序/重复键语义与最终 signature 输入保持逐字节一致；
- 全局/端点/订单维度**限流**、429/418 backoff 与预算；
- server time **时钟**偏移测量、recvWindow 策略与签名前校准；
- 账户/订单**私有 WS**、listen-key 生命周期、断线**重连**、重订阅、去重与 gap 恢复；
- client order id 幂等、未知提交结果对账、部分成交/撤单竞态和失败清理；
- 仅 testnet 的受控 live 下单/查询/撤单证据、金额上限、人工开关与零遗留订单证明。
- 缺少 HTTP/凭据/WS 能力时统一返回可分类错误，而不是成功占位或空流。

这些条件未闭合前维持**交易 NO-GO**。公共 WS 行情解析不得代替私有订单流或连续性证据。

## 5. 验证

```bash
cmp .agents/ssot/adapters/exchange/binance/spec/spec.md \
  .agents/ssot/adapters/exchange/binance/spec/xhyper-binance-complete-spec.md
cargo test -p binancex --all-targets
cargo clippy -p binancex --all-targets -- -D warnings
```

验收只覆盖当前签名 REST、公共 WS 和错误映射声明；不得据此解除交易 NO-GO。
