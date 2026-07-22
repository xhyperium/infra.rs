# Changelog

## [0.3.1] — 2026-07-22

### Added

- `insert_batch(table, rows, BatchInsertOptions { max_rows_per_chunk })` 分块插入
- 纯函数 `chunk_ranges`（可测 chunk 尺寸）
- 池强化：`max_idle_per_host` / `max_in_flight`（默认 64）/ `acquire_timeout`
- `ClickHousePoolStats { in_flight, closed }`；关闭后拒绝新请求
- `ClickHouseConfig::validate`：`max_in_flight ≥ 1`

### Changed

- 版本 PATCH 0.3.0 → 0.3.1
- `connect` 使用配置的 `pool_max_idle_per_host`；请求经 Semaphore 背压

## [Unreleased]

### Added

- 生产默认：`ClickHouseConfig` / `ClickHousePool` HTTP 客户端（8123）
- `AnalyticsSink` 真实 insert 路径 + `query_text` / `query_rows` / `insert_json_each_row`
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `ClickHouseAdapter`

### Changed

- 收敛到 `xhyper-contracts::AnalyticsSink`；默认路径不再是 scaffold
