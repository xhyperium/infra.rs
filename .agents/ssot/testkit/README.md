> **Stable**：战役 **COMPLETE** · residual **OPEN=0**（仅 1 OPTIONAL branch cov）· `xhyper-testkit 0.1.1`（`publish = false`，internal only）· Stable CLAIMED 2026-07-14。

# testkit — Deterministic Test Support 管线契约

> 实现代码唯一位置：[`crates/testkit`](../../../crates/testkit)（core）· [`crates/test-support/contracts`](../../../crates/test-support/contracts)（contract-testkit）
> **当前 SSOT Spec**：`SPEC-TESTKIT-002`（[spec/spec.md](spec/spec.md) ≡ [xhyper-testkit-complete-spec.md](xhyper-testkit-complete-spec.md)）
> **Source Goal**：`GOAL-DETERMINISTIC-TEST-SUPPORT` — **Done**（ship 2026-07-14）
> **Ship PR**：[#247](https://github.com/xhyperium/infra.rs/pull/247) · [#254](https://github.com/xhyperium/infra.rs/pull/254) · [#255](https://github.com/xhyperium/infra.rs/pull/255) · tag `testkit-v0.1.1`
> **历史**：[testkit-spec.md](spec/spec.md)（SUPERSEDED，`xlib_harness` 草案）→ SPEC-TESTKIT-002

> **本仓维护候选（2026-07-23）**：`contract-testkit 0.1.2` 正在以独立 PR 收口 14 trait / 15 broken case、确定性 `FixtureNamespace`、production graph gate 与公开 API baseline。该候选不改变既有 testkit core Stable 记录；PR/CI/人工批准前不得宣称已发布或真实 backend ready。

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | **Done** · AC 对照 §24 已 ship |
| Spec | [spec/spec.md](spec/spec.md) | **Stable**（2026-07-14）· §24.0 ship 后状态 |
| Design | [design/design.md](design/design.md) | 索引式 · 权威在 spec §1–§7、§25 |
| Plan | [plan/plan.md](plan/plan.md) | 十轮验收 DONE · W0–W6 ship DONE（见 §26） |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | W0–W9 **done** |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | next = 保持 Stable + 收 OPTIONAL |
| **Code** | **`crates/testkit/`** | ManualClock V2 已落地（`clock.rs` + `lib.rs`，496 行） |
| Test | [test/test.md](test/test.md) | **PASS**（mutants missed=0 · Miri PASS · line cov ≥95%）· branch cov OPTIONAL |
| Review | [review/review.md](review/review.md) | **COMPLETE** for 0.1.1 ship |
| Release | [release/release.md](release/release.md) | **COMPLETE** · `publish = false`（internal only，不发布 crates.io） |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | ship 复盘 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/README.md](evidence/README.md) → 仓库根 [`evidence/testkit/2026-07-14-stable-gates/`](../../../evidence/testkit/2026-07-14-stable-gates/) |
| Residual ledger | [plan/residual-open.md](plan/residual-open.md) · DEF-001…010 **全 CLOSED** + 1 OPTIONAL |
| Gap matrix | [plan/gap-matrix.md](plan/gap-matrix.md) |
| Spec inventory | [plan/spec-inventory.md](plan/spec-inventory.md)（I-1…I-26） |
| Approval packet | [plan/approval-packet.md](plan/approval-packet.md) · A1–A10 |
| 10× verdict | [plan/testkit-plan-10x-verdict.md](plan/testkit-plan-10x-verdict.md) |

## 硬限制

1. `testkit` 永远 `publish = false`、T0 test-support plane，**无 production layer**——禁止进入生产依赖图（`cargo xtl test-graph-check`）。
2. **无证据不得宣称 Done / §24 全闭合**——`branch cov ≥90%` 仍 OPTIONAL（line≥95% 已强制），不据此宣称额外保证。
3. 禁止回退退役宏：`xlib_test!` / `mock!` / `FixtureBuilder` / `provider_capability_contract_tests!`（见 spec §8 退役合同）。
4. `ManualClock` 禁读真实时间、禁 `sleep`、禁 unchecked arithmetic、禁 `Clone`/`Default`（见 spec §7）。
5. `crates.io` package 名为 **`xhyper-testkit`**（lib 名 `testkit`）；`[features] default = []`。

## 验证

```bash
# SSOT 双镜像同构（必须 exit 0）
cmp .agents/ssot/testkit/spec/spec.md \
    .agents/ssot/testkit/xhyper-testkit-complete-spec.md

# 包名与 Cargo.toml 一致
grep -n '^name' crates/testkit/Cargo.toml   # 应为 xhyper-testkit

# 测试 + 图隔离
cargo test -p testkit -p contract-testkit
node scripts/quality-gates/check-test-support-graph.mjs
cargo xtl lint-deps
```

**Plan 十轮 + W0–W6 ship：DONE · 0.1.1 Stable CLAIMED · residual OPEN=0（1 OPTIONAL）· `publish = false` 保持。**
