# Changelog

## [0.3.4] — 2026-07-23

### Added

- 公开 API 表面测试扩面：crate-root 常量、`TaosConfig` URL/`from_env`、池同步面方法全点名
- 十轮 draft 审查与最终缺口矩阵：`docs/report/2026-07-23/taosx-ten-round-review.md`
- 真实 dev live 证据（`export-foundationx-env` + `live_smoke` 2/2）与 SSOT matrix S-5b

### Changed

- SSOT 对齐文档 version 与 package 同步为 `0.3.4`；明确 **不** 另建 `adapters/storage/taosx/` SSOT 树
- crate docs 补充真实配置 live / 有界 bench 命令（密钥仅环境注入）

### Boundaries

- 仍不宣称 package stable / Native SQL / HA / 幂等自动重试 / 24h soak

## [0.3.3] — 2026-07-23

### Changed

- `query_series` 缺表空集路径改为依赖类型化 `ErrorKind::Missing`（`map_taos_code`），去掉驱动文案子串匹配

### Added

- 精度配置与探测结果不一致时 `connect` fail-closed 单测
- 缺表 Missing vs 非 Missing 错误传播对抗单测
- `tests/taos_conformance.rs`：远程明文拒绝、响应上界、schema 冲突、close/背压等离线边界

### Boundaries

- 未宣称 package stable / 真实集群 HA

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
