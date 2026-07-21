> **Post-ship**：branch/miri/mutants **实测 PASS**（mutants missed=0；均为 ad-hoc 本地/手动跑，CI 门禁待补 P1）。

# Test — SPEC-KERNEL-002

| 字段 | 值 |
|------|-----|
| Status | §18.3 全实测 PASS（line/branch/mutants/miri）；mutants/miri/branch 为 ad-hoc 本地跑，CI 门禁待补（P1） |
| Source Spec | `SPEC-KERNEL-002` |
| Ship PR | [#235](https://github.com/xhyperium/infra.rs/pull/235)（merged e7bda98e） |

## 当前证据

| 项 | 结果 |
|----|------|
| `cargo test -p kernel` | lib + 集成全绿（含 LC-005：poison/1000/guard drop） |
| loom + CI `kernel-loom` | 本地 2 passed；workflow 已挂 |
| archgate KERNEL-* | **infra.rs 不适用（OOS）**（历史 monorepo；本仓不移植） |
| public-api | 本仓以 `cargo public-api` / CI 为准；**不**维护 `.architecture/api/**` |
| line coverage | 98.95% ≥95% PASS |
| branch coverage | 100% ≥90% PASS（ad-hoc nightly；CI 门禁待补 P1） |
| mutants | missed=0 PASS（ad-hoc；CI 门禁待补 P1） |
| miri | 21 passed PASS（ad-hoc；CI 门禁待补 P1） |
| api_compile | static_assertions（含 Guard !Clone / Mono !Default）；trybuild **DEFER accepted**（RES-TEST-005 CLOSED） |
| G2 证据 | [G2-tests](../evidence/2026-07-14/EVID-KERNEL-002-G2-tests.md) · [G2-archgate-ci](../evidence/2026-07-14/EVID-KERNEL-002-G2-archgate-ci.md) |
| ad-hoc artifacts | `kernel-branch-cov-nightly.txt` / `mutants/outcomes.json` / `kernel-miri-lib.txt`（见 `evidence/2026-07-14/`） |

## P1 follow-up（CI 门禁化）

- branch ≥90% 固化为 CI job（RES-TEST-014 CI 化）
- mutants 固化为 nightly/scheduled job（RES-TEST-015 CI 化）
- miri 固化为 nightly/scheduled job（RES-TEST-016 CI 化）

## 禁止

把 ad-hoc 实测结果粉饰为「CI 已门禁」。