# hyperliquid 目标

## 已存在

- [x] crate 已建立：路径 `crates/exchange/hyperliquid`，package `exchange-hyperliquid`，lib `exchange_hyperliquid`
- [x] 已建立 WebSocket 流、Info 请求/响应、K 线周期、币种元数据、配置和连接状态类型骨架
- [x] `HyperliquidAdapter` 已实现 `VenueAdapter` 接口骨架（协议运行方法仍返回未实现错误）

## 待实现

- [ ] WebSocket 连接/断连/重连
- [ ] `allMids` → `MidPrice` 边界（不得伪装为 Quote）
- [ ] `l2Book` → `OrderBook` 全量 snapshot 更新映射（每条消息整簿替换；连接恢复后重新 bootstrap）
- [ ] `trades` → `Tick` 映射
- [ ] webbook2：仅在官方 schema 可追溯后重新纳入
- [ ] REST Info API 实现（allMids、l2Book、recentTrades、meta、candleSnapshot）
- [ ] coin ↔ InstrumentKey 双向映射
- [ ] 指数退避重连
