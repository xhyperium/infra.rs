# Changelog

## [0.3.2] — 2026-07-22

### Fixed

- 签名 REST：HTTP 4xx + 业务错误体优先映射 `ErrorKind`（先于 status→Unavailable）
- 测试：HTTP 400 + `-1013`/`-2011` 驱动 place/cancel 真实入口


## [0.3.1] — 2026-07-22

### Added

- HMAC-SHA256 固定时间戳向量测试（`sign_query_with_timestamp` / `sign_payload`）
- 公共 WS 行情解析：`parse_binance_book_ticker` / `parse_binance_trade` / `parse_binance_orderbook`
- `BinanceAdapter::with_ws` / `with_ws_base`：注入 `WsConnector` 后 `subscribe_*` 产出真实帧流
- 签名 REST mock 内容断言（method/path/`X-MBX-APIKEY`/signature + `OrderAck`/`OrderStatus`）

### Fixed

- 签名 REST：HTTP 4xx + 业务错误体优先映射 `ErrorKind`（先于 status→Unavailable）
- 测试：HTTP 400 + `-1013`/`-2011` 驱动 place/cancel 入口

- `OrderAck.ts` 按 CAN-TIME-001 从交易所毫秒转为纳秒
- 无凭证路径明确 mock（`Open`），不静默假成交

### Notes

- `publish = false`；**非** package stable / crates.io / L5 代签
- 全量用户数据流、OCO 等仍 OPEN

## [Unreleased]

### Changed

- 收敛到 `xhyper-contracts::VenueAdapter` 及能力拆分 trait
- 移除本地 Error / ExchangeAdapter 面
