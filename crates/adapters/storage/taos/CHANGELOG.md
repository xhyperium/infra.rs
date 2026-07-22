# Changelog

## [0.3.2] — 2026-07-23

### Changed

- 远程明文、空认证与 URL 注入配置 fail-closed；REST 禁止 redirect
- bid/ask 改为 NCHAR(64) Decimal 文本，并拒绝存量 DOUBLE schema
- 为响应、SQL batch、query rows、in-flight 与 close drain 增加硬上界
- 子表名改用 symbol 完整十六进制编码，消除清洗碰撞
- 明确 NativeWs 仅握手可达性探测；Native SQL / HA / 幂等重试仍 NO-GO

### Added

- scale=18 / 大 mantissa / 正负 Decimal 往返测试
- 固定 TDengine 镜像 digest 的隔离 live conformance 脚本

## [0.3.1] — 2026-07-22

### Added

- 显式批量写入：`write_batch` / `write_batch_chunked` + 纯函数 `build_insert_sql_chunks`
- `TransportMode { Rest, NativeWs }`；`native_ws_url` / `connect_native_ws`（真实 WS 握手尝试）
- 池强化：`max_in_flight` Semaphore + `TaosPoolStats { in_flight, closed }`
- acquire 超时 → `DeadlineExceeded`；关闭后拒绝新请求
- `TaosConfig::validate`（`max_in_flight` / `batch_max_rows` ≥ 1）

### Changed

- 版本 PATCH 0.3.0 → 0.3.1
- `TimeSeriesStore::write_series` 委托 `write_batch`

## [Unreleased]

### Added

- 生产默认：`TaosConfig` / `TaosPool` REST 客户端（6041）
- `TimeSeriesStore` 真实 write/query + 库精度探测
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `TaosAdapter`

### Changed

- 收敛到 `xhyper-contracts::TimeSeriesStore`；默认路径不再是 scaffold
