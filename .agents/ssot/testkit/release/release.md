> **历史发布记录（2026-07-14，非当前发布声明）**：以下 tag/Stable 信息只描述 `xhyper-testkit 0.1.1`。本仓当前 package 为 `testkit 0.1.3`、`publish = false`；见 [../README.md](../README.md)。

# Release — GOAL-DETERMINISTIC-TEST-SUPPORT

| 字段 | 值 |
|------|-----|
| Release ID | `REL-TESTKIT-002-0.1.1` |
| Status | **COMPLETE** · ship 2026-07-14 |
| Spec | `SPEC-TESTKIT-002` · **Stable** |
| Path package / lib | `crates/testkit` · lib **`testkit`** |
| crates.io package | **`xhyper-testkit`**（**internal only · 未发布**） |
| Version | `0.1.1` |
| Registry | **Stable**（`publish = false`——internal only，**不发布到 crates.io**） |
| Ship PR | [#247](https://github.com/xhyperium/infra.rs/pull/247) · [#254](https://github.com/xhyperium/infra.rs/pull/254) · [#255](https://github.com/xhyperium/infra.rs/pull/255) |
| Tag | `testkit-v0.1.1` |
| Evidence | [plan/residual-open.md](../plan/residual-open.md) · [plan/approval-packet.md](../plan/approval-packet.md) · 仓库根 `evidence/testkit/2026-07-14-stable-gates/` |

> **internal only**：testkit 是 T0 test-support plane，`publish = false`。Release 指仓库内 tag + main 合入，**非** crates.io 发布。无 GitHub Release / crates.io URL。

## 已交付

- SPEC-TESTKIT-002 Stable · ManualClock V2（PR #247 #254 #255）
- `crates/testkit`：Mutex State · checked wall/mono · Fault 三态 → ClockError · Snapshot · poison 恢复（4 公开类型）
- `contract-testkit`：trait-level suites + broken impl negative tests
- 退役宏拆除 + test-graph-check 防回流门禁
- tag `testkit-v0.1.1` · main 合入

## 质量门禁（ship 时点实测）

| 项 | 状态 |
|----|------|
| line coverage ≥95% | **PASS**（CI `testkit-quality`） |
| mutants | **PASS**（missed=0） |
| Miri | **PASS** |
| test-graph-check | **PASS** |
| archgate | **OOS / N/A**（infra.rs 不移植 archgate；CI 质量 job 与 archgate 无关） |
| branch coverage ≥90% | **OPTIONAL**（非阻塞） |

## 后续（非战役阻塞）

- `branch cov ≥90%` 进 nightly（OPTIONAL）。
- integration harness：跨 crate（INFRA-010+），另开 spec。
- 破坏性 API 变更：新 spec 版本或 supersede，bump version 走 `scripts/version.mjs` + CHANGELOG `[Unreleased]`。

**Status: COMPLETE。`publish = false` 保持。未发布到 crates.io。**
