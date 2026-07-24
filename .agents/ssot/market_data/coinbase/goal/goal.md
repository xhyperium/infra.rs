# coinbase 目标

## 已存在

- [x] crate 已建立：路径 `crates/exchange/coinbase`，package `exchange-coinbase`，lib `exchange_coinbase`
- [x] 已建立 Coinbase 频道、粒度、订阅、认证频道、产品、配置和连接状态类型骨架
- [x] `CoinbaseAdapter` 已实现 `VenueAdapter` 接口骨架（协议运行方法仍返回未实现错误）

## 待实现

- [ ] WebSocket 连接/断连/重连（含指数退避）
- [ ] `ticker` → `Quote` 映射
- [ ] `market_trades` → `Tick` 映射
- [ ] `level2` snapshot → `OrderBook(Snapshot)` 映射
- [ ] `level2` update → `OrderBook(Delta)` 映射（保留 sequence 元数据；连续性规则须由 raw fixture 证明）
- [ ] `candles` → `Bar` 映射
- [ ] REST `GET /products` — 产品列表查询
- [ ] REST `GET /products/{product_id}/candles` — K 线历史
- [ ] REST `GET /products/{product_id}/ticker` — 实时 Ticker
- [ ] REST `GET /products/{product_id}/book` — 订单簿快照
- [ ] REST cursor 分页遍历
- [ ] Heartbeat 频道接收
- [ ] 多 product_ids 批量订阅/取消
