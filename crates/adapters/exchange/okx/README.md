# okxx

OKX exchange adapter。

Package：`okxx` · path：`crates/adapters/exchange/okx`

SSOT 镜像：`.agents/ssot/adapters/exchange/okx/`（若已注册）  
合约：`infra-contracts`（`ExchangeAdapter`）

## 状态

**scaffold** — 实现 `ExchangeAdapter` 状态机与占位 `fetch_ticker`；**无**真实 HTTP / package stable。

## 最小用法

```rust
use infra_contracts::exchange::ExchangeAdapter;
use okxx::OkxAdapter;

let mut adapter = OkxAdapter::demo();
adapter.connect().expect("connect");
let _ = adapter.fetch_ticker("BTC-USDT");
```
