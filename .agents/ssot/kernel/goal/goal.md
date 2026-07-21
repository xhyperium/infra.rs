# Goal — kernel L0 Runtime Semantics 闭合（SPEC-KERNEL-002）

| 字段 | 值 |
|------|-----|
| Goal ID | `GOAL-KERNEL-RUNTIME-SEMANTICS` |
| Status | **`Done`** |
| Source Spec | [spec/spec.md](../spec/spec.md)（**Approved** · §18 **CLOSED**） |
| Design | [design.md](../design/design.md) |
| Package | path `crates/kernel` · crates.io **`xhyper-kernel` `0.1.1`** · lib **`kernel`** |
| Registry | **`stable`** |
| Ship | [#235](https://github.com/xhyperium/infra.rs/pull/235) MERGED · tag `kernel-v0.1.1` · crates.io published |
| Residual | [residual-open.txt](../evidence/2026-07-14/residual-open.txt) · **OPEN=0** |
| Campaign evidence | [EVID-KERNEL-002-CAMPAIGN-COMPLETE.md](../evidence/2026-07-14/EVID-KERNEL-002-CAMPAIGN-COMPLETE.md) |

## Acceptance Criteria

- [x] AC-1：SPEC-KERNEL-002 SSOT + 001 归档
- [x] AC-2：文档诚实（时点 evidence 与 live SSOT 分离）
- [x] AC-3：registry **stable** + `[features] default=[]` + version **0.1.1** + crates.io **`xhyper-kernel`**（`publish=true`）
- [x] AC-4：§18.2 代码主路径闭合
- [x] AC-5：§18.3 测试闭合（unit/proptest/loom/line；branch/miri/mutants **实测**）
- [x] AC-6：§18.4 治理闭合（本仓机控 = 结构扫描 / tests / CI；archgate **OOS**；version；evidence）

## Metrics

| ID | 状态 |
|----|------|
| M1 Spec SSOT | **PASS** |
| M2 文档诚实 | **PASS** |
| M3 registry stable | **PASS** |
| M4 代码主路径 | **PASS** |
| M5 测试闭合 | **PASS**（mutants missed=0） |
| M6 治理闭合 | **PASS**（API-002 implemented + crates.io） |

## Next

战役 **Done**。可选：workspace deny yank（ossx→spin，非 kernel）。
