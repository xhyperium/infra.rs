# resiliencx SSOT 对齐（infra.rs）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-21 |
| Active SSOT | `.agents/ssot/infra/resiliencx/spec/spec.md` |
| 用户路径别名 | `.agent/ssot/resiliencx` → `.agents/ssot/infra/resiliencx` |
| 实现 | `crates/resiliencx` · package `xhyper-resiliencx` |

## 结论

- Active §2 retry 合同已落地并可测；**Lines Cover 100%**。
- 熔断 / 限流 / async wait / package stable：**未交付**（residual OPEN/DEFER）。
- Instrumentation 在本 crate 定义（无 `xhyper-contracts`）；禁止 observex。

详见 `plan/alignment-matrix-infra-2026-07-21.md` 与 `plan/residual-open.md`。
