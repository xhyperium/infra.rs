> **Post-ship**：战役 COMPLETE · residual OPEN=0 · crates.io xhyper-kernel 0.1.1 · archgate 15/15（API-002）。

# kernel — Goal 管线契约

> 实现代码唯一位置：[`crates/kernel`](../../../crates/kernel)  
> **当前 SSOT Spec**：`SPEC-KERNEL-002`（[spec/spec.md](spec/spec.md) ≡ [xhyper-kernel-complete-spec.md](spec/xhyper-kernel-complete-spec.md)）  
> **Source Goal**：`GOAL-KERNEL-RUNTIME-SEMANTICS` — **Done**（§18 闭合 · stable · published）  
> **Ship PR**：[\#235](https://github.com/xhyperium/xhyper.rs/pull/235) · 分支 `feat/kernel-002-e2-migrate-banned-apis`  
> **历史**：`SPEC-KERNEL-001` → [spec/SPEC-KERNEL-001.superseded.md](spec/SPEC-KERNEL-001.superseded.md)

## 11 层映射

| 管线层 | 路径 | 相对 002 |
|--------|------|----------|
| Goal | [goal/goal.md](goal/goal.md) | Active · AC-1..4 done |
| Spec | [spec/spec.md](spec/spec.md) | **Approved** |
| Design | [design/design.md](design/design.md) | Active · 主路径已落地 |
| Plan | [plan/plan.md](plan/plan.md) | D/E/C/L done · G partial |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | E1–E3/C1–C2/L1–L2/G1/G2 **done**（机器轨） |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | DONE（campaign complete） |
| **Code** | **`crates/kernel/`** | 主路径 + G2 测试/门禁 |
| Test | [test/test.md](test/test.md) | §18.3 全实测 PASS（line/branch/mutants/miri）；mutants/miri/branch 为 ad-hoc，CI 门禁待补（P1） |
| Review | [review/review.md](review/review.md) | PASS（PR #235/#238/#241 已 merge） |
| Release | [release/release.md](release/release.md) | DONE（0.1.1 · tag · crates.io published） |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 本波记录 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/2026-07-14/](evidence/2026-07-14/) |
| Residual ledger | [evidence/2026-07-14/residual-open.txt](evidence/2026-07-14/residual-open.txt) |
| R10 终裁 | [evidence/2026-07-14/EVID-KERNEL-002-R10-verdict.md](evidence/2026-07-14/EVID-KERNEL-002-R10-verdict.md) |

## 硬限制

1. §18 已闭合，registry = **stable**（已 publish crates.io `xhyper-kernel` 0.1.1）
2. 无证据不得宣称 Done / 3/3 / 5/5
3. L0 仅 error/clock/lifecycle；crates.io **`xhyper-kernel`**（`publish = true`）；`[features] default = []`
4. 禁止再引入 `not_found` / `other` / 默认 monotonic / `Component` trait

## 验证

```bash
cmp .agent/SSOT/kernel/spec/spec.md .agent/SSOT/kernel/spec/xhyper-kernel-complete-spec.md
cargo test -p kernel
rg -n "status = \"stable\"" .architecture/workspace.toml | grep kernel
```

**Phase D–L + G2 机器轨：PASS · §18 全闭合（含 ad-hoc 实测）· Spec Approved · registry stable · 已 publish crates.io。注：mutants/miri/branch-cov CI 门禁化见 P1 follow-up。**
