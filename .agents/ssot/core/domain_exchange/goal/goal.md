# domain_exchange 目标

## 已存在

- [x] crate 已建立：`crates/domain_exchange`（package/lib：`domain_exchange`）
- [x] `VenueAdapter` 13 个方法的异步接口骨架
- [x] StreamType、OrderAmend、AccountInfo、InstrumentMeta
- [x] AdapterError 当前八个变体（含 `Unsupported`）

## 已完成（本轮）

- [x] DE-API-001：13 方法 + Display 全变体测试
- [x] DE-LIFE-001：mock 级 connect/disconnect 门禁与幂等（非 live）
- [x] DE-REST-001：REST-only `Unsupported` 负能力测试

## 已完成（扩展）

- [x] DE-ERR-001：`RateLimitDetail` / `RateLimitDetailed`
- [x] DE-CAP-001：`exchange_id` + `VenueCapabilities`
- [x] DE-PAGE-001：`PageRequest` / `Page<T>` 类型（trait 仍单页）

## 已完成（扩展）

- [x] DE-PAGE trait 默认 `get_open_orders_page` / `get_instruments_page`

## 仍待实现

- [ ] DE-LIFE 深化：live 重连恢复、并发调用、Drop 契约
- [ ] DE-PAGE：各 adapter 覆盖真实 cursor/time-window
- [ ] DE-ERR 全 adapter provider code 映射表
