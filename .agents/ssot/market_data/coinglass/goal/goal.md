# coinglass 目标

## 已存在

- [x] crate 已建立：路径 `crates/exchange/coinglass`，package `exchange-coinglass`，lib `exchange_coinglass`
- [x] `CoinglassResponse<T>`、`CoinglassConfig` 和 `RateLimitConfig` 类型及默认值骨架已建立
- [x] `CoinglassAdapter` 已实现 `VenueAdapter` 接口骨架（协议运行方法仍返回未实现错误）

## 待实现

- [ ] HTTP REST 客户端（含 API Key 模式与限频退避）
- [ ] V4 `/api/futures/open-interest/history` → OI 拉取与反序列化
- [ ] V4 `/api/futures/funding-rate/history` → `FundingRatePoint` 拉取与反序列化
- [ ] V4 `/api/futures/liquidation/history` → `LiquidationData` 拉取与反序列化
- [ ] V4 `/api/futures/global-long-short-account-ratio/history` → `LongShortRatioData` 拉取与反序列化
- [ ] V4 `/api/futures/supported-exchange-pairs` → InstrumentKey 映射缓存
- [ ] 跨交易所符号映射表构建
- [ ] HTTP 429 限频退避处理
- [ ] Coinglass → 域模型映射（兑换 domain_market 类型）
- [ ] 五种类型的 JSON 反序列化 round-trip 测试
- [ ] Coinglass API V4 mock 连通性和错误测试（禁止把 live ignore 当通过）
