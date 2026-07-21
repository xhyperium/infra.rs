# Changelog — decimalx

## [Unreleased]

## [0.1.0] - 2026-07-21 — four-crate production tranche（L1 Internal Ready）

### Added

- 可运行 `examples/basic.rs`（checked 四则 + Money serde）
- `tests/public_api_surface.rs` 覆盖构造/四则/舍入/错误变体/serde
- 真实 `benches/hot_path`
- `docs/API.md` 完整公开面；README 声明 **L1 Internal Ready** 与硬限制
- package 选择器统一为 `decimalx`
- W1 证据：`oracle_diff` / `boundary_matrix` / `adversarial_serde`；panicking 门禁脚本
- scheduled CI：`decimal-mutants.yml` / `decimal-miri.yml`

### Notes

- 证据：`docs/plans/releases/2026-07-21-four-crates-internal-release.md`
- **≠** package stable / 跨版本 wire / crates.io

## 0.1.0 — historical initial

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
