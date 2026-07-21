> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# R10c Verdict — SPEC-KERNEL-002 plan §4 十轮验收

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Branch | `feat/kernel-002-e2-migrate-banned-apis` |
| HEAD | `1e4b497b`（十轮全程未变） |
| Plan | `PLAN-KERNEL-002-v2-complete` §4 |
| Campaign | R10c（接 R10b；独立重跑验收） |
| **fail_rounds** | **0** |
| L1 status | **PASS** |
| L2 status | PARTIAL（§18.3 branch/mutants/miri 仍 OPEN） |
| L3 / §18 | **still OPEN**（禁止宣称全闭合） |

## 十轮检查项（每轮均 PASS）

| Check | 结果 |
|-------|------|
| C-FMT | PASS ×10（full re-run R1/R2/R5/R10） |
| C-CLIPPY | PASS ×10（full re-run R1/R2/R5/R10；`-D warnings`） |
| C-TEST | PASS ×10（每轮 `cargo test -p kernel`；lib 20 + 集成） |
| C-LOOM | PASS ×10（full re-run R1/R5/R10：2 passed） |
| C-ARCH | PASS ×10（R1/R5/R10：KERNEL-* **13/13 ok**；internal=8） |
| C-DEPS | PASS ×10（R1/R2/R5/R10：`cargo xtl lint-deps` R1–R6） |
| C-API | PASS ×10（R1/R5/R10：`cargo-public-api` 与 `.architecture/api/kernel-public-api.txt` **exact match** 488 lines；全轮 `context_cow`  absent） |
| C-SSOT | PASS ×10（residual **无** `:\s*Unknown`） |
| C-BAN | PASS ×10（无 not_found/other 构造；无 Component trait 导出；mono 非 trait-default Instant::now） |
| C-§18 | PASS ×10（文档诚实：Spec Proposed / incubating / 未宣称 §18 全闭合或 stable） |

轻量轮（R3/4/6/7/8/9）对未变 HEAD 缓存 C-FMT/C-CLIPPY/C-LOOM/C-ARCH/C-DEPS 结果，但 **C-TEST + C-BAN + C-SSOT + C-§18 + C-API(rg)** 每轮实跑。

## Machine evidence (R1/R5/R10 重检摘要)

```text
cargo fmt -p kernel -- --check                          → exit 0
cargo clippy -p kernel --all-targets -- -D warnings     → exit 0
cargo test -p kernel                                    → ok (lib 20 + api/clock/lifecycle/public)
RUSTFLAGS='--cfg loom' cargo test -p kernel \
  --test lifecycle_concurrency_loom                     → 2 passed
cargo run -p archgate -- --json                         → KERNEL-* 13/13 ok
cargo xtl lint-deps                                     → 依赖图校验通过 R1–R6
cargo public-api -p kernel --simplified                 → exact match snapshot; no context_cow
```

## residual OPEN set（Lead 终态对齐后 · 自 residual-open.txt）

| ID | P | 含义 |
|----|---|------|
| **RES-API-007** | P2 | version 仍 `0.1.0`；待 0.1.1 release 策略 |
| **RES-TEST-014** | P2 | branch coverage ≥90% 未在 stable 测得 |
| **RES-TEST-015** | P2 | cargo-mutants ABSENT / 未跑 |
| **RES-TEST-016** | P2 | miri component ABSENT / 未跑 |

**无 Unknown status。** P3 已 CLOSED：RES-DOWN-006 · RES-PERF-001(DEFER) · RES-EVID-001(partial)。  
W6 已 CLOSED：RES-18-APPROVED（Spec Status Approved · EVID-KERNEL-002-18-APPROVED）。  
战役 CLOSED 亦含：ERR-010 / CLK-010 / LC-005 / TEST-005 DEFER / GATE-009 DEFER / DOC-001。

## C-SSOT 后处理（Lead · post residual P3）

- Verifier 写裁决时 OPEN 仍含 DOWN/PERF/EVID（当时 residual 尚未关）— **已过时**。
- Lead 已将 gate/matrix/review/tasks/plan/gap/goal/design/release/approval/todo 与 residual OPEN=5 对齐。
- release.md 退出条件已改为仅列真实 OPEN。

## §18 / registry（诚实）

```text
fail_rounds:     0
L1 战役:         PASS
§18.1 Approved:  PASS（post-R10c 人审授权 · RES-18-APPROVED CLOSED）
§18 全闭合:      NOT PASS（禁止宣称）· still OPEN
registry:        incubating（禁止 stable）
Spec Status:     Approved
战役层级:        L1 PASS · L2 PARTIAL · §18.1 PASS · §18 全勾 OPEN
OPEN 仅余:       RES-API-007 · RES-TEST-014/015/016
```

## Decision

```text
PASS for:  plan §4 十轮验收（R10c）fail_rounds=0；机器轨全绿；P3 residual 关闭；Spec Approved（W6）
NOT PASS for: full §18 / registry stable / version 0.1.1
NEXT: 可选 version 0.1.1 + nightly branch/mutants/miri
```

## Evidence 索引

- [round-log](./EVID-KERNEL-002-R10c-round-log.txt)
- [residual-open](./residual-open.txt)
- [R10b prior](./EVID-KERNEL-002-R10b-verdict.md)
- [plan §4](../../plan/plan.md)
- [approval-packet](../../plan/approval-packet.md)
