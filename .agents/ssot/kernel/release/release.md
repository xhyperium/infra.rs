# Release — GOAL-KERNEL-RUNTIME-SEMANTICS

| 字段 | 值 |
|------|-----|
| Release ID | `REL-KERNEL-002-0.1.1` |
| Status | **COMPLETE** · land + tag + GitHub Release + **crates.io** |
| Spec | `SPEC-KERNEL-002` · **Approved** · §18 **CLOSED** |
| Path package / lib | `crates/kernel` · lib **`kernel`** |
| crates.io package | **`xhyper-kernel` `0.1.1`** |
| Registry | **`stable`**（历史 monorepo 曾记于 `.architecture/workspace.toml`；**infra.rs 不维护该路径**） |
| Ship PR | [#235](https://github.com/xhyperium/infra.rs/pull/235) **MERGED** |
| Tag | `kernel-v0.1.1` → `e7bda98e` |
| GitHub Release | https://github.com/xhyperium/infra.rs/releases/tag/kernel-v0.1.1 |
| crates.io | https://crates.io/crates/xhyper-kernel/0.1.1 |
| Evidence | [EVID-KERNEL-002-18-RELEASE.md](../evidence/2026-07-14/EVID-KERNEL-002-18-RELEASE.md) · [EVID-KERNEL-002-CRATES-IO.md](../evidence/2026-07-14/EVID-KERNEL-002-CRATES-IO.md) · [EVID-KERNEL-002-CAMPAIGN-COMPLETE.md](../evidence/2026-07-14/EVID-KERNEL-002-CAMPAIGN-COMPLETE.md) |

## 已交付

- SPEC-002 SSOT · Approved · §18 闭合
- 代码主路径 + design-fix + API-002 机控
- version **0.1.1** · registry **stable**
- residual **OPEN=0**
- tag + GitHub Release
- crates.io **`xhyper-kernel` 0.1.1**（`kernel` 名已被占用故改名）

## 质量门禁

| 项 | 状态 |
|----|------|
| line cov | ~98.95% |
| branch | 100%（nightly） |
| miri | 21 passed |
| mutants | missed=0 |
| KERNEL-API-002 | **implemented**（baseline + RFC allowlist） |
| archgate KERNEL-* | **infra.rs 不适用（OOS）** |

## 后续（非战役阻塞）

- workspace `cargo deny` yank（ossx→spin）
- 发布后请 **撤销** 曾在聊天中暴露的 crates.io token
