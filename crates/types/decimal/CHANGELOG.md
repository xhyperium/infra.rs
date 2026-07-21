# Changelog — xhyper-decimalx

## [Unreleased]

### Added

- 按 crates 子模块标准补齐 `examples/` 骨架（`.gitkeep`）
- W1 证据硬化：`tests/oracle_diff.rs`（BigDecimal 差分）、`boundary_matrix.rs`、`adversarial_serde.rs`
- 生产路径 panicking 运算符门禁：`scripts/quality-gates/check-decimal-no-panicking-ops.mjs`
- scheduled CI：`decimal-mutants.yml` / `decimal-miri.yml`

## 0.1.0 — 2026-07-21

### Added

- 初始落地到 `infra.rs`：`Decimal` / `RoundingStrategy` / `Currency` / `Money` / `Price` / `Qty` / `Ratio`
- 生产路径：`try_new`、`FromStr`、`checked_*` 强制 `MAX_SCALE = 18`
- 数值语义 `Eq` / `Ord` / `Hash`；serde 结构字段 shape（**非** wire stable）
- unit / proptest / crate 外入口测试

### Changed（agent-safe 对账）

- Active SSOT §3/§6 与公开 API、测试计数对齐；dual mirror 保持 `cmp` 同构
- crate 外 `entry_checked_ops` 补强 `checked_sub` / `checked_mul` / `checked_rescale` / newtype 具体返回值

### Notes

- 字段已私有（非法 scale / 币种不可在 crate 外表示）；`DecimalError` 可分类 + 中文 Display
- wire stable、Spec Approved、fuzz/oracle/mutants 仍开放（见 SSOT residual / 报告 DEFER）
- agent-safe 对账完成 **≠** Goal Achieved **≠** 整体 Production Ready
