# Gate — testkit（SPEC-TESTKIT-002）

| 字段 | 值 |
|------|-----|
| Plan | [plan/plan.md](../plan/plan.md) |
| Spec | [spec/spec.md](../spec/spec.md)（**Stable** 2026-07-14） |
| Design | [design.md](../design/design.md)（索引式 · 权威在 spec §1–§7、§25） |
| Residual SSOT | [plan/residual-open.md](../plan/residual-open.md) · DEF-001…010 **全 CLOSED** + 1 OPTIONAL |
| Campaign | **COMPLETE** · ship 2026-07-14 · main@testkit-v0.1.1 · PR #247 #254 #255 |

| Gate | 状态 | 备注 |
|------|------|------|
| Spec Stable | **PASS** | 2026-07-14 |
| §24 验收主体 | **PASS** | §24.1–.6 ship 时点主体闭合（见 spec §24.0） |
| version `0.1.1` | **PASS** | package **`xhyper-testkit`** · tag `testkit-v0.1.1` |
| registry Stable | **PASS** | Stable CLAIMED 2026-07-14（`publish = false`，internal only） |
| crates.io 发布 | **N/A** | `publish = false`——不发布，无 crates.io 产物 |
| `cargo test -p testkit -p contract-testkit` | **PASS** | unit / contract / concurrency |
| property (proptest) | **PASS** | |
| `cargo mutants -p testkit` | **PASS** | missed=0（caught=12, unviable=18） |
| `cargo +nightly miri test -p testkit` | **PASS** | |
| line coverage ≥95% | **PASS** | CI `testkit-quality` |
| branch coverage ≥90% | **OPTIONAL** | line≥95% 已强制；branch 进 nightly（非阻塞 Stable） |
| `cargo xtl test-graph-check` | **PASS** | test-support 不进生产图 |
| archgate TESTKIT-* | **OOS / N/A** | **infra.rs 不移植** archgate / `.architecture`；历史 TESTKIT-* 规则 ID 仅参考。可选机控：结构扫描 / CI job / `test-graph-check`（若落地）— 非 archgate |

## contract-testkit 0.1.2 候选门禁（2026-07-23）

| Gate | 状态 |
|------|------|
| reference suites + 15 broken cases | **stack PASS；待最终主干重放**（[evidence](../evidence/2026-07-23-contract-testkit-stack.md)） |
| `check-test-support-graph.mjs` default/all-features | **stack PASS；待 CI**（同上） |
| `contract-testkit` public API baseline | **stack PASS；待 CI**（同上） |
| 独立 code/spec review | **待执行** |
| 人工批准、PR 合并、发布 | **BLOCKED / 未发生** |

候选门禁不覆盖 live backend、EventBus/PubSub delivery 或跨资源原子性。

## Residual OPEN

**无阻塞项。** DEF-001…010 全 CLOSED。仅 1 **OPTIONAL**（branch cov ≥90%，line≥95% 已强制）。详见 [plan/residual-open.md](../plan/residual-open.md)。

## 禁止

- 将 `Stable` 误读为「发布到 crates.io」或「production runtime 就绪」（testkit 是 T0 test-support，`publish=false`，无 production layer）
- 将「主体闭合」误读为「§24 全勾」——`branch cov ≥90%` 仍 OPTIONAL，不据此宣称额外保证
- 回退退役宏（`xlib_test!` / `mock!` / `FixtureBuilder` / `provider_capability_contract_tests!`）
- 把 testkit 装入生产依赖图（`cargo xtl test-graph-check` 必须保持 PASS）
