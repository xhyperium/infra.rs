# canonical wire goldens — v1

| 文件 | 类型 | 等级 |
|------|------|------|
| `cancel_order_request_okx.json` | `CancelOrderRequest` | Committed-candidate |
| `order_ack_legacy.json` | `OrderAck` | Committed-legacy |
| `order_ref_exchange.json` / `order_ref_client.json` | `OrderRef` | Committed-candidate |

- 其它 DTO：**Uncommitted** — 无 v1 golden。
- 变更 Committed 文件必须跑 `cargo test -p xhyper-canonical` 且更新 CHANGELOG。
- **≠** 跨语言 / 跨大版本保证；见 `wire-commitment-matrix.md`。
