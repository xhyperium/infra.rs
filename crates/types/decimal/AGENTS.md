# AGENTS — crates/types/decimal

- Package / lib：`decimalx`；当前交付版本 `0.1.2`；相对 `0.1.1` 已执行一次 PATCH +1。
- 生产声明：仅 **L1 checked path**；不得声明跨语言 wire stable、package stable 或 crates.io Ready。
- Active SSOT：[`.agents/ssot/types/decimal/spec/spec.md`](../../../.agents/ssot/types/decimal/spec/spec.md)。
- 当前事实权威：本仓宪章 + active spec + `src/lib.rs`；ADR-006/007 仅为历史来源记录。
- 私有字段：`Decimal(mantissa, scale)`、`Currency([u8; 3])`、`Money(amount, currency)`；使用公开构造器与访问器。
- 生产入口：`try_new` / parse / validate；运算只用 `checked_*`（`div` 是 checked 别名）。
- `Decimal::new`、`Decimal::rescale` 仅供 const/test/兼容便利，可能 panic，不属于资金生产路径。
- `panicking-ops` 默认关闭；生产资金路径禁止依赖 `+` / `-` / `*`。
- `MAX_SCALE = 18`；禁止 `f32` / `f64` 金额、价格和数量运算。
- `DecimalError -> XError` 必须保留 source chain。
- serde v1 只承诺内部 Rust JSON shape；JSON `i128` 跨语言精度仍是 residual。
- 依赖仅 `kernel` + `serde`；禁止 `decimalx -> canonical`。
- 聚焦门禁：`cargo test -p decimalx`、全 feature clippy、fmt、
  `node scripts/quality-gates/check-decimal-no-panicking-ops.mjs`。
