# Prompt — SPEC-TESTKIT-002

| 字段 | 值 |
|------|-----|
| Source Spec | `SPEC-TESTKIT-002`（[spec/spec.md](../spec/spec.md)） |
| Ship PR | [#247](https://github.com/xhyperium/infra.rs/pull/247) · [#254](https://github.com/xhyperium/infra.rs/pull/254) · [#255](https://github.com/xhyperium/infra.rs/pull/255) |
| Next | 保持 Stable · 收 OPTIONAL（branch cov ≥90%）· 破坏性改动须新 spec 版本 |
| Residual | [plan/residual-open.md](../plan/residual-open.md) · DEF-001…010 全 CLOSED + 1 OPTIONAL |
| Plan | [plan/plan.md](../plan/plan.md) · Tasks [plan/tasks.md](../plan/tasks.md) |
| 10× verdict | [plan/testkit-plan-10x-verdict.md](../plan/testkit-plan-10x-verdict.md) |
| Stable evidence | 仓库根 [`evidence/testkit/2026-07-14-stable-gates/`](../../../../evidence/testkit/2026-07-14-stable-gates) |

## 已完成（勿重复）

- Spec Stable（2026-07-14）· W0–W6 ship（PR #247 #254 #255 · tag testkit-v0.1.1）
- `crates/testkit`：ManualClock V2（Mutex State · checked wall/mono · Fault 三态 → ClockError · Snapshot · poison 恢复）
- 公开面 4 类型：`ManualClock` / `ManualClockError` / `ManualClockFault` / `ManualClockSnapshot`
- `contract-testkit`：trait-level suites（KeyValueStore/EventBus 等 · broken impl negative tests）
- Plan 十轮验收（fail_rounds=0）· spec-inventory I-1…I-26 · gap-matrix · approval A1–A10
- 退役宏拆除：`xlib_test!` / `mock!` / `FixtureBuilder` / `provider_capability_contract_tests!`

## Next（按优先级）

1. **保持 Stable**：任一破坏性 API/合同变更须新 spec 版本或 supersede（AGENTS.md §4.1）。
2. 收 OPTIONAL：`branch cov ≥90%` 进 nightly `testkit-quality`（非阻塞 Stable）。
3. integration harness：跨 crate（INFRA-010+），非 testkit 本体——演进时另开 spec。
4. contract suite 矩阵扩展 / Miri 进 CI required 周期：演进度量，不影响 0.1.1 Stable。

## 禁止回退

- `xlib_test!` / `mock!` / `FixtureBuilder` / `provider_capability_contract_tests!`（退役宏，见 spec §8）
- `ManualClock` 读真实时间 / `sleep` / unchecked arithmetic / `Clone` / `Default`（见 spec §7）
- testkit 装入生产依赖图 / build-dependency（`cargo xtl test-graph-check` 必须保持 PASS）
- 把 testkit 当「测试工具大全」——它是极小 runtime-neutral deterministic primitives（spec §25）
- 假 Done / 手写 PASS 代替命令输出 / SKIP 当 PASS

**战役 Done。破坏性改动走新 spec 版本。无证据不得改 §24 勾选状态。**
