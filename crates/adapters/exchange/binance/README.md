# binancex

Binance exchange adapter — **生产默认 REST+WS 协议路径**（对标 storage P0，**非** package stable / L5）。

实现 `contracts::VenueAdapter` + MarketDataSource / ExecutionVenue / AccountSource / InstrumentCatalog / VenueTimeSource。

## 行为分层

| 注入 | 行为 |
|------|------|
| 默认（无 http / key / ws） | 进程内明确 mock：`place`→`Open`（非 Filled）；行情空流 |
| `with_http` + `with_api_key` | HMAC-SHA256 签名 REST：place / cancel / query / account |
| `with_ws` | 公共行情：`bookTicker` / `trade` / `depth5` 帧解析为 contracts 类型 |
| `#[ignore]` live | 仅公共 `server_time`（env 无密钥入库） |

- `publish = false`；禁止宣称 crates.io / package stable / workspace Production Ready / L5 代签
- 全量用户数据流、OCO 等仍 **OPEN**

## Live 只读 server_time

```bash
cargo test -p binancex --test live_server_time -- --ignored --nocapture
cargo test -p binancex --all-targets   # 默认离线绿
```
