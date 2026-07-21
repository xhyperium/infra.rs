# Gate — kernel（SPEC-KERNEL-002）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-KERNEL-002-v2-complete](../plan/plan.md) |
| Design | [design.md](../design/design.md)（DESIGN-KERNEL-002 · **Active**） |
| Residual SSOT | [residual-open.txt](../evidence/2026-07-14/residual-open.txt) |
| Campaign | **COMPLETE** · 0.1.1 · stable · land · tag · crates.io |

| Gate | 状态 | 备注 |
|------|------|------|
| Spec Approved + §18 | **PASS** | |
| version `0.1.1` | **PASS** | package **`xhyper-kernel`** |
| registry stable | **PASS** | crates/kernel only |
| crates.io | **PASS** | https://crates.io/crates/kernel/0.1.1 |
| cargo test -p kernel | **PASS** | lib 名仍 `kernel` |
| loom CI 资产 | **PASS** | |
| archgate KERNEL-* | **infra.rs 不适用（OOS）** | 历史 monorepo 机控；本仓不移植 archgate |
| API 快照 / public-api | **PASS（本仓 CI 轨）** | 以 unit tests / public-api / 结构扫描为准；本仓不维护 `.architecture/**` |
| KERNEL-API-002 语义 | **设计意图保留** | 公开 API 变更纪律仍有效；**机控不走 archgate** |
| line cov ≥95% | **PASS ~98.95%** | |
| branch ≥90% | **PASS 100%** | nightly llvm-cov --branch（ad-hoc 实测；CI 门禁待补 P1） |
| miri | **PASS 21** | MIRIFLAGS disable-isolation（ad-hoc；CI 门禁待补 P1） |
| mutants | **PASS** | missed=0 · 31 caught · 2 timeout（ad-hoc；CI 门禁待补 P1） |

## Residual OPEN

**无。**

## 禁止

- 将历史 evidence（R10/R10b 时点）当成 live SSOT
- 将 `kernel` crates.io 原名与 **`xhyper-kernel`** 混淆
