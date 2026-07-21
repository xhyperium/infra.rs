# CI Negative Fixtures Inventory（SPEC-CI-SSOT-001 §26）

> **性质**：`PHASE-1-13` 清单；可执行子集 + 残余 **STUB**。  
> **权威**：`.agent/SSOT/cicd/20260716/xhyper-ci-ssot-complete-spec-2026-07-16.md` **§26** 原文 20 项。  
> **机器清单**：`ci_negative/manifest.toml`（精确 ID / maturity / driver / seam / expected outcome）。本文件是人读投影。
> **≠** 全量 20/20 已生产激活 / Aggregate Ruleset 裁决 / Goal Achieved。  
> 台账：`PHASE-1-13` / `PHASE-5-07`。

| # | Spec §26 Fixture（字面） | 状态 | Notes |
| - | ------------------------ | ---- | ----- |
| 1 | Missing Lane | **EXECUTABLE** | `fixtures/missing_lane.json` + chaos |
| 2 | Cancelled Lane | **EXECUTABLE** | `fixtures/cancelled_lane.json` + aggregate driver |
| 3 | Unexpected Skip | **EXECUTABLE** | `fixtures/unexpected_skip.json`（SKIP ≠ PASS） |
| 4 | Invalid N/A | **EXECUTABLE** | `fixtures/invalid_na.json`（无 reason） |
| 5 | Invalid Reused Attestation | **EXECUTABLE** | `fixtures/missing_reused_attestation.json` |
| 6 | Fingerprint Mismatch | **STUB** | REUSED 输入不一致 |
| 7 | Runner Digest Mismatch | **EXECUTABLE** | 隔离 verify-runner 正控 + digest 单变量 |
| 8 | Tool Version Mismatch | **EXECUTABLE** | 隔离 locks 正控 + nextest version 单变量 |
| 9 | Disk Insufficient | **EXECUTABLE** | 隔离 verify-runner 正控 + disk 单变量 |
| 10 | Planner Unknown | **STUB** | unknown → full（plan 单测覆盖分类） |
| 11 | Cargo Graph Parse Failure | **STUB** | affected graph |
| 12 | Merge Group Event | **STUB** | merge_group cadence |
| 13 | Fork PR | **STUB** | trust policy workflow 条件 |
| 14 | GitHub API 429/5xx | **STUB** | control-plane |
| 15 | Generated Drift | **EXECUTABLE** | temp root render/MATCH 正控 + workflow-contract hand-edit |
| 16 | Ruleset Drift | **STUB** | reconcile |
| 17 | Cache Corruption | **STUB** | CAS/L3 |
| 18 | External Cleanup Failure | **STUB** | lease controller |
| 19 | Flake Expiry | **EXECUTABLE** | `fixtures/flake_expired.toml` |
| 20 | Aggregate Unknown State | **EXECUTABLE** | `fixtures/aggregate_unknown_state.json` + aggregate driver |

## 使用说明

- `EXECUTABLE` 仅表示本次 `ci chaos` 通过闭集 driver 调用 shipped seam，并观察到场景特异失败；不等于外部生产能力已验证。
- 未落地前保持 **STUB**。全量 20/20 仍 **DEFERRED**。
- 当前计数：11 `EXECUTABLE` + 9 `STUB`；允许 `gate_ok=true` 且 `coverage_complete=false`。
- Aggregate **无** `decisions_file` 默认 FAIL（禁止全绿）；见 `ci aggregate` / chaos `aggregate_no_decisions_file`。
- 机读：`cargo xtl ci chaos --json`；manifest 结构负测：`cargo test -p xhyper-xtask ci::chaos`。
