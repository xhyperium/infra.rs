# Matrix — testkit

| 字段 | 值 |
|------|-----|
| Active Goal | `GOAL-DETERMINISTIC-TEST-SUPPORT` · **Done** |
| Active Spec | `SPEC-TESTKIT-002`（**Stable** 2026-07-14） |
| Design | 索引式 · 权威在 [spec §1–§7、§25](../spec/spec.md) |
| Package | **`xhyper-testkit` 0.1.1** · lib `testkit` · registry **Stable**（`publish=false`，internal only） |
| Ship | #247 #254 #255 · tag `testkit-v0.1.1` |
| Residual | DEF-001…010 **全 CLOSED** + 1 OPTIONAL（branch cov ≥90%） |
| archgate | **OOS**（infra.rs 不移植 archgate / `.architecture`）· 可选残留：结构扫描 / CI / test-graph-check（非 archgate） |

## Residual classes

| Class | 状态 |
|-------|------|
| 定位（layer=test-support · active spec 唯一） | **CLOSED**（§24.1） |
| Core（ManualClock V2 · 只依赖 kernel · 无宏/无 FixtureBuilder） | **CLOSED**（§24.2） |
| 测试（unit/property/concurrency/compile · mutants missed=0 · Miri · line≥95%） | **PASS**（§24.3）· branch cov OPTIONAL |
| Contract（trait-level suites · broken impl negative tests） | **CLOSED**（§24.4） |
| 图隔离（dev-dependency only · 不进生产图） | **CLOSED**（§24.5 · test-graph-check PASS） |
| 治理（ADR/公开面/CHANGELOG/Evidence） | **CLOSED**（§24.6） |

**OPEN：0 阻塞。仅 1 OPTIONAL（branch cov ≥90%，line≥95% 已强制）。**
