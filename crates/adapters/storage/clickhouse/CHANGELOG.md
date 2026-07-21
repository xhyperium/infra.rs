# Changelog

## [Unreleased]

### Added

- 生产默认：`ClickHouseConfig` / `ClickHousePool` HTTP 客户端（8123）
- `AnalyticsSink` 真实 insert 路径 + `query_text` / `query_rows` / `insert_json_each_row`
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `ClickHouseAdapter`

### Changed

- 收敛到 `xhyper-contracts::AnalyticsSink`；默认路径不再是 scaffold
