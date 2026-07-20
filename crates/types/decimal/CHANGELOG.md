# Changelog — xhyper-decimalx

## 0.1.0 — 2026-07-21

### Added

- 初始落地到 `infra.rs`：`Decimal` / `RoundingStrategy` / `Currency` / `Money` / `Price` / `Qty` / `Ratio`
- 生产路径：`try_new`、`FromStr`、`checked_*` 强制 `MAX_SCALE = 18`
- 数值语义 `Eq` / `Ord` / `Hash`；serde 结构字段 shape（**非** wire stable）
- unit / proptest / crate 外入口测试

### Notes

- 字段仍 `pub`；独立 `DecimalError`、wire stable、Spec Approved 仍开放（见 SSOT residual）
