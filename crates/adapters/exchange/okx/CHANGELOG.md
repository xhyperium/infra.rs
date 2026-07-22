# Changelog

## [0.3.3] — 2026-07-22

### Fixed

- place：`code=0` 且 `sCode≠0` 入口测；空体 HTTP 非 2xx → Unavailable
- adapter 级 trades/books5 WS fixture 流测（对齐 binance）

## [0.3.2] — 2026-07-22

### Fixed

- cancel：顶层 `code=0` 时仍校验 `data[0].sCode`（与 place 对齐）
- 测试：`code=0` + `sCode=51400` 必须 Err

## [0.3.1] — 2026-07-22

### Added

- `OkxAdapter::with_api_key`：四头鉴权接到 place/cancel/query/balance
- OKX `code`/`data` 业务信封解析与 `state`→`OrderStatus` 映射
- 公共 WS 行情解析：`parse_okx_ticker` / `parse_okx_trade` / `parse_okx_orderbook`
- `OkxAdapter::with_ws`：注入后 subscribe 发送业务订阅帧并解析推送
- 签名向量与 mock 内容断言（四头 + 信封错误码）

### Fixed

- `OkxApiKey` 时间戳改为 `{unix_seconds}.{millis:03}`（符合 OKX REST 约定）
- 无凭证路径明确 mock，不再用正文子串冒充协议作为主路径

### Notes

- `publish = false`；**非** package stable / crates.io / L5 代签
- 私有账户 WS、统一账户全量端点等仍 OPEN

## [Unreleased]

### Changed

- 收敛到 `xhyper-contracts::VenueAdapter` 及能力拆分 trait
- 移除本地 Error / ExchangeAdapter 面
