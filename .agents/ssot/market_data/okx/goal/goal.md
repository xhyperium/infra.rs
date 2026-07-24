# okx 目标

## 已存在

- [x] crate 已建立：路径 `crates/exchange/okx`，package `exchange-okx`，lib `exchange_okx`
- [x] 已建立 OKX 频道、深度级别、REST 响应、交易对信息、配置和连接状态类型骨架
- [x] `OkxAdapter` 已实现 `VenueAdapter` 接口骨架（协议运行方法仍返回未实现错误）

## 待实现

- [ ] 三条产品线 WebSocket 连接/断连/重连
- [ ] `tickers` → `Quote` 映射
- [ ] `trades` → `Tick` 映射
- [ ] `books` → `OrderBook` 映射（含 `seqId/prevSeqId` 连续性；当前官方废弃 checksum 不作 CRC32 校验）
- [ ] `candle` → `Bar` 映射
- [ ] REST API 接口实现
- [ ] instId ↔ InstrumentKey 双向映射
- [ ] ping 保活
