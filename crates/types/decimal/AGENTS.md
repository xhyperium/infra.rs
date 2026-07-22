# AGENTS — crates/types/decimal

- Package：`decimalx` · lib：`decimalx` · path：`crates/types/decimal` · version：`0.1.1`
- 生产层级：**L1 Internal Ready**
- 生产路径：`try_new` / parse / `checked_*`；禁止资金路径依赖 panicking ops
- 依赖：仅 `kernel` + `serde`；禁止 canonical 反向依赖
- Active SSOT：`.agents/ssot/types/decimal/spec/spec.md`
- 禁止 `f32`/`f64` 金额运算
- 门禁：`cargo test -p decimalx` · `cargo clippy -p decimalx --all-targets -- -D warnings`
- 示例：`cargo run -p decimalx --example basic`
- panicking 门禁：`node scripts/quality-gates/check-decimal-no-panicking-ops.mjs`
