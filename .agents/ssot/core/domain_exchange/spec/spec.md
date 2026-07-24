# domain_exchange 交易所抽象规格

**版本**：0.2.3
**状态**：契约冻结；trait/DTO、mock 生命周期、REST-only 负能力、结构化错误、能力矩阵和默认分页已实现；live 重连、真实 adapter 映射和 provider cursor 仍待
**实现**：`crates/domain_exchange/src/lib.rs`、`tests/mock_lifecycle.rs`
**最后更新**：2026-07-22

## 1. 职责与边界

`domain_exchange` 定义各交易所适配器共享的异步入口、订单修改参数、账户与标的信息、错误分类。它不实现 HTTP/WebSocket、签名、重试、连接池或交易所特有 DTO。

当前公共 trait 是兼容骨架；它保留了交易和行情两类方法，因此 REST-only provider 只能对不适用方法返回明确的 `Unsupported` 语义（`AdapterError::Unsupported` 已提供），不得伪装为网络或内部故障。结构化 retry_after/scope 仍见 `DE-ERR-001`。

## 2. VenueAdapter 当前契约

trait 为 `Send + Sync`，不额外要求 `Debug`。方法必须与源码保持以下 13 个签名语义：

| 分组 | 方法 | 结果 |
|---|---|---|
| 生命周期 | `connect()` / `disconnect()` | 会话建立/释放 |
| 公开行情 | `subscribe_ticker(&InstrumentKey)` / `subscribe_order_book(&InstrumentKey)` / `subscribe_trades(&InstrumentKey)` | 单标的一次订阅请求 |
| 交易 | `place_order(&Order)` / `cancel_order(&OrderId, &InstrumentKey)` / `amend_order(&OrderAmend)` | 执行回报或错误 |
| 订单查询 | `get_order(&OrderId, &InstrumentKey)` / `get_open_orders(&InstrumentKey)` | 单订单或单页未结订单 |
| 账户与标的 | `get_account_info()` / `get_instruments()` | 账户信息或单页标的元数据 |
| 行情查询 | `get_order_book(&InstrumentKey, Option<u32>)` | 单次订单簿快照 |

`get_open_orders`、`get_instruments` 的 `Vec<T>` 只表示单页返回，不隐含“全量”。
分页入口为默认方法 `get_open_orders_page` / `get_instruments_page`（`PageRequest` → `Page<T>`）；
默认实现忽略 cursor、应用 limit 并返回 `has_more=false`。真实游标由 adapter 覆盖。

## 3. 辅助类型

### 3.1 StreamType

源码变体为 `Ticker`、`Level1`、`Trade`、`Level2`、`MiniTicker`，使用 camelCase serde 和 `#[non_exhaustive]`。provider 不支持的流必须在 adapter spec 的能力矩阵标为 `unsupported`。

### 3.2 OrderAmend

```rust
pub struct OrderAmend {
    pub order_id: OrderId,
    pub price: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub new_client_order_id: Option<String>,
}
```

`order_id` 是必填定位字段；price/quantity/stop_price 至少一个非 None，除非 provider 支持只修改 client id。最终校验由 adapter 完成。

### 3.3 AccountInfo 与 InstrumentMeta

`AccountInfo` 字段为 `account_id`、`balances: Vec<Balance>`、`can_trade`、`can_withdraw`、`can_deposit`、`update_time: Timestamp`；`Balance` 为 `asset`、`free`、`locked`。

`InstrumentMeta` 当前字段为 `symbol`、`status`、`base_asset`、`quote_asset`、`price_precision`、`quantity_precision`、`min_notional`、`min_quantity`、`step_size`、`tick_size`。由于当前实现仍用 provider symbol，不能声称该类型已与 `domain_market::InstrumentKey` 完成强类型连接。

## 4. 错误契约

源码的 `AdapterError` 为 `#[non_exhaustive]`，当前变体是：

| 变体 | 触发场景 | 默认动作 |
|---|---|---|
| `InvalidRequest(String)` | 缺字段、非法 symbol/limit/订单参数、未连接调用 | 不重试 |
| `Authentication(String)` | key/JWT/签名失效 | 重新认证或终止 |
| `RateLimit(String)` | provider 限频、连接/账户额度超限 | 读取 Retry-After 后退避 |
| `Network(String)` | DNS/TCP/HTTP 传输失败 | 仅幂等请求可重试 |
| `WebSocket(String)` | WS close、订阅拒绝、ping/pong 超时 | 按 adapter 状态机恢复 |
| `Parse(String)` | 响应 shape、数字或序列化失败 | 丢弃报文并告警，不盲目重试 |
| `Internal(String)` | 未实现或违反内部状态 | 阻断门禁，不能当 provider 错误 |
| `Unsupported(String)` | venue 明确不支持该操作（REST-only 的 WS/交易等） | 不重试；不得伪装 Network/Internal |

供应商错误码映射必须逐 adapter 记录 HTTP status、provider code、request id、重试性和恢复动作。`RateLimit(String)` 尚不能表达 `retry_after_ms`/scope，结构化错误由 `DE-ERR-001` 追踪。

## 5. 生命周期与并发

目标状态机为 `Disconnected → Connected → Subscribed`，但当前 trait 没有状态查询。实现必须明确：

- 未连接调用订阅、查询或交易的结果；
- 重复 connect/disconnect 是否幂等；
- 重连是否恢复订阅、是否重新拉 snapshot；
- `&self` 方法的内部状态同步和并发调用行为；
- Drop、取消和认证过期后的资源释放。

所有 adapter 当前实现方法返回 skeleton `Internal` 错误；这证明“入口存在”，不证明状态机门禁通过。

## 6. 能力、分页与 REST-only 限制

trait 已提供 `exchange_id()` 与 `capabilities()`（默认 unknown）；adapter 应覆盖并在自身 spec 给出能力矩阵（产品线、行情流、订单、深度、认证、分页）。

Coinglass 只提供公开 REST 数据，不支持行情 WebSocket、账户或下单。它暂时实现该兼容 trait 仅为 workspace 接口对齐；所有不适用方法均为 `pending/unsupported`，不得列 P0“连接通过”。

## 7. 依赖

| 依赖 | 用途 |
|---|---|
| `domainx` | Order、ExecutionReport、Decimal、Timestamp |
| `domain_market` | InstrumentKey、OrderBook |
| `async-trait` | 异步 trait |
| `serde` | DTO 序列化 |
| `thiserror` | AdapterError 展示 |

历史文档中的 `xhyper-canonical`、`transport`、`resiliencx` 不是当前 workspace 依赖，只有在实际纳入 Cargo workspace 后才能成为实现依赖。

## 8. 可执行门禁

| ID | 门禁 | 证据 | 状态 |
|---|---|---|---|
| DE-API-001 | 13 个方法、参数、返回值与源码一致；`AdapterError` Display | `cargo test -p domain_exchange` + `tests/mock_lifecycle.rs` | verified |
| DE-ERR-001 | `RateLimitDetail` + `RateLimitDetailed`（retry_after/scope/code/http/request_id） | `tests/cap_err_page.rs` | verified（结构化类型；全 adapter 映射仍待） |
| DE-LIFE-001 | 未连接、重复连接、断开、mock 级 13 方法可达（非 live 重连/并发压力） | `tests/mock_lifecycle.rs` StatefulMock | verified（mock 级） |
| DE-CAP-001 | `exchange_id()` / `capabilities()` 默认方法 + `VenueCapabilities` | trait + `tests/cap_err_page.rs` + coinglass 覆盖 | verified |
| DE-PAGE-001 | `PageRequest`/`Page` + trait 默认 `*_page` 方法 + cursor 不变量 | `tests/cap_err_page.rs` | verified（默认单页；真实游标由 adapter 覆盖） |
| DE-REST-001 | REST-only provider 不伪装 WS/交易能力 | `RestOnlyMock` + coinglass `Unsupported` | verified |

说明：DE-LIFE-001 的 live 重连恢复与并发压力、DE-ERR-001 的全 adapter provider 映射、DE-CAP/PAGE 的各 adapter 覆盖仍 pending；本规格已 verified 的范围仅是公共类型、默认方法和 mock/negative tests，不因 mock 通过而声称 live 完成。
