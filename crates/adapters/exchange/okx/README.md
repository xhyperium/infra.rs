# okxx

OKX exchange adapter — **生产默认 REST+WS 协议路径**（对标 storage P0，**非** package stable / L5）。

实现 `contracts::VenueAdapter` + MarketDataSource / ExecutionVenue / AccountSource / InstrumentCatalog / VenueTimeSource。

## 行为分层

| 注入 | 行为 |
|------|------|
| 默认（无 http / key / ws） | 进程内明确 mock：`place`→`Open`（非 Filled）；行情空流 |
| `with_http` + `with_api_key` | 四头鉴权 REST + `code`/`data` 信封：place / cancel / query / balance |
| `with_ws` | 公共行情：`tickers` / `trades` / `books5` 订阅帧 + 推送解析 |
| `#[ignore]` live | 仅公共 `server_time` |

- `publish = false`；禁止宣称 crates.io / package stable / L5 代签
- 私有账户 WS、统一账户全量端点等仍 **OPEN**
- `demo()` 与 mainnet 同 host；模拟盘头 `x-simulated-trading` **未**默认注入（勿当已可 demo 交易）

## Live 只读 server_time

```bash
cargo test -p okxx --test live_server_time -- --ignored --nocapture
cargo test -p okxx --all-targets
```
