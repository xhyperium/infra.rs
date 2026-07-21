# Changelog

## [Unreleased]

### Added

- 生产默认：`TaosConfig` / `TaosPool` REST 客户端（6041）
- `TimeSeriesStore` 真实 write/query + 库精度探测
- live smoke（`#[ignore]`）与 `hot_path` bench
- feature `scaffold`：保留内存 `TaosAdapter`

### Changed

- 收敛到 `xhyper-contracts::TimeSeriesStore`；默认路径不再是 scaffold
