# Goal — testkit Deterministic Test Support 闭合（SPEC-TESTKIT-002）

| 字段 | 值 |
|------|-----|
| Goal ID | `GOAL-DETERMINISTIC-TEST-SUPPORT` |
| Status | **`Done`** |
| Source Spec | [spec/spec.md](../spec/spec.md)（**Stable** 2026-07-14） |
| Design | [design.md](../design/design.md)（索引式 · 权威在 spec §1–§7、§25） |
| Package | path `crates/testkit` · package **`xhyper-testkit` `0.1.1`** · lib **`testkit`** |
| Registry | **Stable**（`publish = false`；internal only，不发布 crates.io） |
| Ship | [#247](https://github.com/xhyperium/infra.rs/pull/247) · [#254](https://github.com/xhyperium/infra.rs/pull/254) · [#255](https://github.com/xhyperium/infra.rs/pull/255) · tag `testkit-v0.1.1` |
| Residual | [plan/residual-open.md](../plan/residual-open.md) · DEF-001…010 **全 CLOSED** + 1 OPTIONAL（branch cov ≥90%） |
| Plan | [plan/plan.md](../plan/plan.md) · 十轮验收 DONE |
| Campaign evidence | 仓库根 [`evidence/testkit/2026-07-14-stable-gates/`](../../../../evidence/testkit/2026-07-14-stable-gates) |

## Acceptance Criteria（对照 spec §24，ship 时点已闭合主体）

- [x] AC-1（§24.1 定位闭合）：layer = test-support · 不再声明 L0 runtime · active spec 唯一 · README/AGENTS/architecture 对齐
- [x] AC-2（§24.2 Core 闭合）：只依赖 kernel · 无 feature · 无宏 · 无 FixtureBuilder · ManualClock V2 · 无真实时间/sleep/unchecked arithmetic/Clone/Default
- [x] AC-3（§24.3 测试闭合）：unit/property/concurrency/compile assertions · line ≥95% · mutation missed=0 · Miri PASS（`branch ≥90%` 仍 OPTIONAL）
- [x] AC-4（§24.4 Contract 闭合）：contract-testkit trait-level suites · 无具体 adapter dependency · broken impl negative tests
- [x] AC-5（§24.5 图隔离闭合）：所有消费 dev-dependency · 无 build-dependency · `cargo xtl test-graph-check` 生效
- [x] AC-6（§24.6 治理闭合）：ADR/公开面/archgate/test-graph-check/negative fixtures/CHANGELOG/Evidence

> ship 时点（2026-07-14）AC-1…6 主体已闭合。残留：AC-3 的 `branch cov ≥90%` 为 OPTIONAL（line≥95% 已强制），不影响 0.1.1 Stable。

## Metrics

| ID | 状态 |
|----|------|
| M1 Spec Stable | **PASS**（2026-07-14） |
| M2 W0–W6 ship | **PASS**（PR #247 #254 #255 · tag testkit-v0.1.1） |
| M3 Plan 十轮验收 | **PASS**（fail_rounds=0） |
| M4 §24 验收闭合 | **PASS**（主体闭合 · branch cov OPTIONAL） |
| M5 Residual 清零 | **PASS**（DEF-001…010 全 CLOSED · 1 OPTIONAL 非阻塞） |
| M6 package Stable / 0.1.1 | **PASS**（`publish = false`，Stable CLAIMED，不发布 crates.io） |

## Next

1. 保持 Stable：任一破坏性改动须新 spec 版本或 supersede（AGENTS.md §4.1）。
2. 收 OPTIONAL：`branch cov ≥90%` 进 nightly（非阻塞 Stable）。
3. integration harness：跨 crate 依赖（INFRA-010+），非 testkit 本体范围。

**战役 Done。0.1.1 Stable CLAIMED。`publish = false` 保持。**
