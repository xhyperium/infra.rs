# types/decimal — Review

> **状态**：实现基线已落地 · **未** Goal Achieved · **未** Spec Approved · **不得** READY  
> 审查默认 NOT PASS（人审项仍开放）。

## 本轮（2026-07-21 / infra.rs）

- **实现路径**：`crates/types/decimal` · package `xhyper-decimalx` · lib `decimalx` · `0.1.0`
- **对齐权威**：Active SSOT `spec/spec.md`（dual mirror `cmp` exit 0）
- **agent-safe 基线**：表示、五策略舍入、checked 四则/rescale、数值 Eq/Ord/Hash、Currency/Money/Price/Qty/Ratio、serde 字段 shape、MAX_SCALE 生产路径强制、`# Panics` 文档
- **门禁**：`cargo test -p xhyper-decimalx`（unit+proptest+entry）、`fmt --check`、`clippy -D warnings`、依赖仅 kernel+serde、无 f32/f64 金融路径
- **明确未闭合**（见 `plan/residual-open.md`）：T-HUM-001..005、T-DEF-001..003、T-POL-001

## 禁止

- 空目录批量标 DONE / READY / PASS
- 将 HUMAN_ONLY / DEFERRED / POLICY 伪标完成
- 宣称 serde wire 跨版本 stable 或 Goal Achieved
