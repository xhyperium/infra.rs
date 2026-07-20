# Round 10 — §24 完成定义 / §25 裁定 / Forbidden 交叉 / DEF / consumers 实测

| 字段 | 值 |
|------|-----|
| Scope | 计划完备性（非实现验收） |
| Spec | `SPEC-TESTKIT-002` §24–§25 + 交叉一致 |
| 日期 | 2026-07-14 |
| 消费者扫描 | `rg` / `Cargo.toml` 只读实测 |

## 检查项

| # | 检查内容 |
|---|----------|
| 10.1 | §24.1–24.6 全部 checkbox 是否落入 I-DONE + 有关闭 Task |
| 10.2 | tasks `T-24-*` 是否覆盖每一勾（可追溯） |
| 10.3 | §25 终态/必须删除/原则 与 plan Forbidden 交叉一致 |
| 10.4 | DEF-001…010 在 gap / residual / todo 一致 |
| 10.5 | consumers inventory 与仓库实测一致 |
| 10.6 | Forbidden 十条与规范禁令无矛盾 |

---

## 1) §24 勾选 ↔ Task 追溯表

### 24.1 定位闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| layer = test-support | I-DONE-24.1 | T-ARCH-001 · T-24-001 | YES |
| 不再声明为 L0 runtime | I-DONE-24.1 | T-ARCH-004/008/012 · T-24-001 | YES |
| active spec 只有一份 | I-DONE-24.1 | T-ARCH-007 · T-24-001 | YES* |
| README / AGENTS / architecture 对齐 | I-DONE-24.1 | T-ARCH-004/008/009 · T-24-001 | YES |

\*路径终态争议见 Round 8 F8-1；勾选**有** Task，路径字符串不完整不在本表重复计 FAIL。

### 24.2 Core 闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| 只依赖 kernel | I-DONE-24.2 | T-GATE-003 DEP-001（维持） | YES |
| 无 feature | I-DONE-24.2 | T-GATE-003 FEATURE-001 | YES |
| 无宏 | I-DONE-24.2 | T-DEL-001/002/004 · T-24-002 | YES |
| 无 FixtureBuilder | I-DONE-24.2 | T-DEL-003 | YES |
| 无 provider suite | I-DONE-24.2 | T-DEL-004 | YES |
| ManualClock V2 | I-DONE-24.2 | T-CLK-* · T-24-002 | YES |
| 无真实时间 | I-DONE-24.2 | T-CLK-018 · TIME-001 | YES |
| 无 sleep | I-DONE-24.2 | T-CLK-018 · TIME-001 | YES |
| 无 unchecked arithmetic | I-DONE-24.2 | T-CLK-007/008 | YES |
| 无 Clone / Default | I-DONE-24.2 | T-CLK-006/012 · T-CLK-017 | YES |

### 24.3 测试闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| unit | I-DONE-24.3 | T-CLK-014 | YES |
| property | I-DONE-24.3 | T-CLK-015 | YES |
| concurrency | I-DONE-24.3 | T-CLK-016 | YES |
| compile assertions | I-DONE-24.3 | T-CLK-017 | YES |
| line ≥ 95% | I-DONE-24.3 | T-GATE-008 | YES |
| **branch ≥ 90%** | I-DONE-24.3 / I-TEST-COV / I-METRICS | **T-GATE-008 AC 仅「≥95%」** | **WEAK** |
| mutation ≥ 90% | I-DONE-24.3 | T-GATE-009 | YES* |
| Miri | I-DONE-24.3 | T-GATE-010 | YES |

\*mutation 禁存活列表见 Round 6 F6-2；score Task 存在。

### 24.4 Contract 闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| trait-level suites | I-DONE-24.4 | T-CTC-004…009 | YES |
| 无具体 adapter dependency | I-DONE-24.4 | T-CTC-002 | YES |
| 显式 profile | I-DONE-24.4 | T-CTC-004… | YES |
| broken implementation negative tests | I-DONE-24.4 | T-CTC-012 · T-24-004 | YES |
| fake/sandbox/real 分层 | I-DONE-24.4 | T-CTC-015 | YES |
| 无隐藏依赖 | I-DONE-24.4 | T-CTC-010 · HIDDEN-DEP | YES |

### 24.5 图隔离闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| 所有消费均为 dev-dependency | I-DONE-24.5 | T-GATE-002 GRAPH-001/002 | YES |
| 无 build-dependency | I-DONE-24.5 | GRAPH-003 | YES |
| 所有 release normal graph 无 test-support | I-DONE-24.5 | GRAPH-004 · T-GATE-005 | YES |
| feature 不会泄漏 test-support | I-DONE-24.5 | GRAPH-005 | YES |
| machine gate 生效 | I-DONE-24.5 | T-GATE-001…003 · T-24-005 | YES |

### 24.6 治理闭合

| Checkbox | Inventory | 关闭 Task | 覆盖？ |
|----------|-----------|-----------|--------|
| RFC / ADR | I-DONE-24.6 | T-ARCH-006 · T-HUM-001 · T-24-006 | YES* |
| public API snapshot | I-DONE-24.6 | T-GATE-004 | YES |
| archgate | I-DONE-24.6 | T-GATE-003 | YES |
| xtask test-graph-check | I-DONE-24.6 | T-GATE-001 | YES |
| negative fixtures | I-DONE-24.6 | T-CTC-012 | YES |
| CHANGELOG | I-DONE-24.6 | T-ARCH-010 | YES |
| Evidence | I-DONE-24.6 | T-EVID-001 · T-24-006 | YES |

\*RFC 等价关系见 Round 8 F8-2。

**§24 勾选数量**：24.1×4 + 24.2×10 + 24.3×8 + 24.4×6 + 24.5×5 + 24.6×7 = **40**；I-DONE 结构覆盖 40 勾；`T-24-001…006` 分区闭合 + 底层 Task 支撑。

---

## 2) DEF-001…010 交叉一致

| ID | gap-matrix | residual-open | testkit-todo | 一致？ |
|----|------------|---------------|--------------|--------|
| DEF-001 | layer/docs L0·kernel P0 OPEN | OPEN 同义 | OPEN 同义 | YES |
| DEF-002 | ManualClock Atomic/unchecked P0 | OPEN | OPEN | YES |
| DEF-003 | 无 fault/snapshot/Result P0 | OPEN | OPEN | YES |
| DEF-004 | 宏/FixtureBuilder P0 | OPEN | OPEN | YES |
| DEF-005 | provider 宏+Binance/OKX P0 | OPEN | OPEN | YES |
| DEF-006 | 无 contract-testkit P0 | OPEN | OPEN | YES |
| DEF-007 | 图隔离/archgate/CI P1 | OPEN | OPEN | YES |
| DEF-008 | 多份 spec P1 | OPEN | OPEN | YES |
| DEF-009 | Spec 未 Approved；§24 未闭合 P1 | OPEN | OPEN | YES |
| DEF-010 | coverage/mutation/Miri P1 | OPEN | OPEN | YES |

计数 residual：OPEN DEF 10 · CLOSED 0 —— 与 todo「全部 OPEN」一致。

---

## 3) Consumers inventory vs 仓库实测（2026-07-14）

| 计划声称 | 实测 | 一致？ |
|----------|------|--------|
| `provider_capability_contract_tests!` → binance + okx | `crates/adapters/exchange/binance/src/lib.rs` · `okx/src/lib.rs` | **YES** |
| `xlib_test!` 外部 0 | 仅 `crates/testkit/src/lib.rs` 自测 | **YES** |
| `mock!` 外部 0 | 仅 testkit 自测 | **YES** |
| `FixtureBuilder` 外部 0 | 仅 testkit 自测 | **YES** |
| `ManualClock` 外部 0 | 无 adapters/domain 引用 | **YES** |
| testkit **dev-dep** binance/okx | 两 crate `Cargo.toml` `[dev-dependencies]` | **YES** |
| testkit **normal-dep** 0 | 全仓 `Cargo.toml` 仅 binance/okx dev + 自身 package | **YES** |
| workspace member | 根 `Cargo.toml` members 含 `crates/testkit` | **YES** |

> §10 Mock* 命名审计清单 **不是** testkit 宏消费者 inventory 的一部分；缺口记 Round 6 F6-3，不否定上表。

---

## 4) §25 与 Forbidden 交叉

| §25 要点 | plan 对应 | 一致？ |
|----------|-----------|--------|
| testkit = 极小 deterministic primitives = ManualClock | plan §0.1 / §4 | YES |
| contract-testkit 独立 | plan §5 / W4 | YES |
| integration harness 不在 core | plan §0.1；residual DEFER harness | YES |
| 必须删除 xlib_test!/mock!/FixtureBuilder/provider from core | W3 T-DEL；Forbidden #5 | YES |
| 空 Mock / 包装 #[test] / 零字段 Fixture / 硬编码 provider 非合同 | Forbidden #5/#6；gap §2.2 | YES |
| 质量=消除隐式时间/隐藏依赖/漂移/图污染 | Forbidden #4/#7/#8 | YES |

plan §1.0 十条 Forbidden 与规范多处禁令无冲突；I-26 指向 plan §1.0。

---

## PASS

1. **I-DONE-24.1…24.6** 结构完整，覆盖规范 40 个 checkbox 语义。
2. **T-24-001…007** 分区闭合 + stable 单独决策（T-24-007 DEFER 友好）。
3. **DEF-001…010** 三处台账一致，严重度与关闭 Task 可追踪。
4. **Consumers inventory 与 rg/Cargo 实测一致**（宏/ManualClock/dep kind）。
5. **§25 终态叙述与 plan 战略/Forbidden 交叉一致**。
6. **Campaign 诚实**：PLANNING · 实现 ABSENT · ≠ stable · ≠ §24 —— 符合 Forbidden #1/#2/#9。
7. **W9 依赖链意图正确**：T-24-* 依赖 HUM/DEL/CLK/GATE/CTC/EVID。

## FAIL

### F10-1 — `T-GATE-008` AC 未绑定 branch ≥ 90%（§24.3 勾）

- **规范引用**：§24.3 `branch >= 90%`；§13.7；§23 `manual_clock_branch_coverage >= 90%`。
- **缺失**：
  - `T-GATE-008` 内容/AC：**「coverage ≥95% 强制 | llvm-cov」**——只体现 line；
  - plan §6.3 仅 `--fail-under-lines 95`；
  - I-TEST-COV / I-METRICS / I-DONE-24.3 **有** branch，但 **执行 Task AC 漏写** → §24.3 可能被 line 单项勾过。
- **建议补丁**：
  1. `T-GATE-008` AC → `line≥95% AND branch≥90%`（I-TEST-COV 全勾）。
  2. 写明 llvm-cov 参数或二次解析 `coverage.json` 的 branch 字段。
  3. `T-24-003` 依赖显式包含 branch 验收。

### F10-2 — `T-24-003` 依赖链不覆盖 property/concurrency/compile

- **规范引用**：§24.3 含 unit · property · concurrency · compile assertions · coverage · mutation · Miri。
- **缺失**：
  - `T-24-003` 依赖列仅 **`T-GATE-008…010`**（coverage/mutation/Miri）；
  - **未依赖** `T-CLK-014…017`（unit/property/concurrency/compile）；
  - 若有人只跑 W6 quality gate 就勾 24.3，会漏测试类型闭合。
- **建议补丁**：
  ```text
  T-24-003 依赖: T-CLK-014 T-CLK-015 T-CLK-016 T-CLK-017 T-GATE-008 T-GATE-009 T-GATE-010
  ```
  或 AC 强制 `I-DONE-24.3` 逐勾证据链接到上述 Task。

### F10-3 — `T-24-006` 依赖过窄（仅 T-EVID-001）

- **规范引用**：§24.6 七勾：RFC/ADR · API snapshot · archgate · test-graph-check · negative fixtures · CHANGELOG · Evidence。
- **缺失**：依赖仅 `T-EVID-001`；**未串** T-ARCH-006/010 · T-GATE-001/003/004 · T-CTC-012 · T-HUM-001。
- **建议补丁**：扩大依赖 DAG 或 AC 清单强制 I-DONE-24.6 七条各自 Evidence 指针（与 F10-2 同类「闭合 Task 依赖不完整」）。

### F10-4（观察·非主 FAIL）— §13.2 三分未进 §24.3 勾选

- 规范 §24.3 未点名 Clock suite 三分；缺口已在 Round 6 F6-1。本轮不双计。

## 本轮结论：FAIL

## fail_count: 3

> 主 FAIL：F10-1、F10-2、F10-3。  
> DEF 一致与 consumers 实测 **PASS**。  
> 实现 ABSENT / §24 未勾选 **不**计 FAIL。
