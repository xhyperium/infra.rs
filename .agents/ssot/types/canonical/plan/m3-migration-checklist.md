# M3 Consumer Migration Checklist — `xhyper-canonical`

| 字段 | 值 |
|------|-----|
| Phase | C |
| 状态 | **DONE**（structured cancel + native ack.id + OrderId type removed） |

## DONE

- [x] contracts additive structured cancel/query；legacy deprecated
- [x] binance/okx：`place_order` 返回 **原生** exchange id（不再 `{symbol}:{id}`）
- [x] legacy cancel 仅解析历史编码；裸 id 失败并提示 `cancel_order_request`
- [x] testkit / bootstrap CancelOrderRequest
- [x] DTO `ts` = Unix ns；`OrderId` 类型删除
- [x] 调用路径审计

## DEFER

- [ ] package stable / crates.io publish
