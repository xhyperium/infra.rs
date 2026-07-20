# AGENTS — crates/types/decimal

- Package：`xhyper-decimalx` · lib：`decimalx`
- 生产路径：`try_new` / parse / `checked_*`；禁止资金路径依赖 panicking ops
- 依赖：仅 `xhyper-kernel` + `serde`；禁止 canonical 反向依赖
- Active SSOT：`.agents/ssot/types/decimal/spec/spec.md`
- 禁止 `f32`/`f64` 金额运算
