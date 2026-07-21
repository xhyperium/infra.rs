# canonical wire goldens — v1

| 文件 | 类型 | 等级 |
|------|------|------|
| `cancel_order_request_okx.json` | `CancelOrderRequest` | Committed v1 |
| `order_ack_legacy.json` | `OrderAck` | Committed v1 (legacy) |
| `order_ref_exchange.json` / `order_ref_client.json` | `OrderRef` | Committed v1 |

- 后续批次：`../v1.1/`（Order）、`../v1.2/`（Tick/Trade）、`../v1.3/`（Position/簿/元数据）。
- 变更 Committed 文件必须跑 `cargo test -p canonical` 且更新 CHANGELOG。
- **≠** 跨语言 / 跨大版本 / package Production Ready 保证。
