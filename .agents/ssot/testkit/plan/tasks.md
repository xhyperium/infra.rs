# Tasks — SPEC-TESTKIT-002 原子任务表

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TESTKIT-002-v1-complete` |
| Status enum | `TODO` · `IN_PROGRESS` · `DONE` · `BLOCKED` · `DEFER` · `CANCELLED` |
| Baseline | 2026-07-14 inventory |

> 完成定义：每条 Task 的 AC 必须可机器或文件证据验证；`DONE` 禁止无输出。  
> 依赖列使用 Task ID。路径互斥见 plan §3。

---

## W0 — 台账 / 冻结 / 计划 10x

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-PLAN-001 | 落盘 plan.md | 含 §0–§11；Forbidden 十条 | — | Planner | **DONE** |
| T-PLAN-002 | 落盘 gap-matrix.md | §0–§25 + DEF-001…010 + 实现差距 | — | Planner | **DONE** |
| T-PLAN-003 | 落盘 tasks.md | 覆盖 §24 勾选项 + 迁移 Phase | — | Planner | **DONE** |
| T-PLAN-004 | 落盘 approval-packet.md | 人审闸门清晰 | — | Planner | **DONE** |
| T-PLAN-005 | 落盘 residual-open.md | DEF 全登记 OPEN/CLOSED | T-PLAN-002 | Planner | **DONE** |
| T-PLAN-006 | 落盘 spec-inventory.md | I-* 枚举齐全防遗漏 | T-PLAN-001 | Planner | **DONE** |
| T-TODO-001 | 更新 `.worktrees/testkit-todo.md` | 全 Wave/DEF 可追踪 | T-PLAN-* | Planner | **DONE** |
| T-INV-001 | 消费者扫描（宏/ManualClock/Cargo） | consumers 表写入 plan §0.6 + residual | — | Planner | **DONE** |
| T-FREEZE-001 | 冻结：禁新增 xlib_test/mock/FixtureBuilder/provider 宏/normal dep | residual + README 冻结节 | T-INV-001 | Doc | **DONE** |
| T-DOC-001 | complete-spec 页眉 Status 保持 Proposed；标注 plan 路径 | 链接 plan/ | — | Doc | **DONE** |
| T-DOC-002 | 旧 testkit-spec.md Superseded 页眉 | 指向 complete-spec + plan | — | Doc | **DONE** |
| T-DOC-003 | 旧 testkitx-spec.md Superseded 页眉 | 不得被读成第二实现 | — | Doc | **DONE** |
| T-V10-PLAN | 计划完备性十轮检查 | fail_rounds=0；verdict 文件 | T-PLAN-* T-TODO-001 T-PLAN-006 | Verifier | **DONE** (pass3 fail_rounds=0) |
| T-BRANCH-001 | 创建 docs/testkit-002-plan（或等价） | 非 main；不与 evidence 实现混提 | — | Lead | **DONE** (`docs/testkit-002-plan` worktree) |

---

## W1 — ManualClock V2

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-CLK-001 | 拆分 `src/clock.rs`；lib.rs 仅 re-export | **I-DIR-CORE 正例树** 完全一致 | T-BRANCH-001 | Clock | TODO |
| T-CLK-002 | Mutex State 模型 | wall + monotonic_elapsed + wall_fault | T-CLK-001 | Clock | TODO |
| T-CLK-003 | ManualClockFault 三态 + 映射 ClockError | I-CLK-FAULT + **I-CLK-DERIVE** | T-CLK-002 | Clock | TODO |
| T-CLK-004 | ManualClockError + Display+Error | 无 anyhow；I-CLK-ERR + **I-CLK-DERIVE** | T-CLK-002 | Clock | TODO |
| T-CLK-005 | ManualClockSnapshot + getters | 私有字段；const getter | T-CLK-002 | Clock | TODO |
| T-CLK-006 | new / with_monotonic_elapsed | 无 Default | T-CLK-002 | Clock | TODO |
| T-CLK-007 | set/advance/rewind wall checked | 失败不改状态；I-CLK-WALL | T-CLK-006 | Clock | TODO |
| T-CLK-008 | set/advance monotonic | regression/overflow；无 rewind | T-CLK-006 | Clock | TODO |
| T-CLK-009 | fault set/clear/get | 不影响 mono；不改 wall 值 | T-CLK-003 T-CLK-007 | Clock | TODO |
| T-CLK-010 | snapshot 同锁 | 撕裂禁止 | T-CLK-005 | Clock | TODO |
| T-CLK-011 | impl Clock | now fault 映射；mono poison 恢复文档 | T-CLK-009 | Clock | TODO |
| T-CLK-012 | 无 Clone；Send+Sync 断言 | compile assertion | T-CLK-002 | Clock | TODO |
| T-CLK-013 | deprecated 旧 nanos API（可选） | 删除版本+调用点清单 | T-CLK-007 | Clock | TODO |
| T-CLK-014 | 单元测试 §13.1 全矩阵 | I-TEST-UNIT 全勾 | T-CLK-011 | Test | TODO |
| T-CLK-015 | property tests §13.3 | proptest dev-dep | T-CLK-014 | Test | TODO |
| T-CLK-016 | concurrency tests §13.4 | Arc 共享；无撕裂 | T-CLK-014 | Test | TODO |
| T-CLK-017 | compile_fail / static_assertions §13.5 | !Default !Clone Send Sync；无旧宏导出（W3 后） | T-CLK-012 | Test | TODO |
| T-CLK-018 | 无真实时间/sleep 源码守卫 | rg + 可选 archgate | T-CLK-001 | Gate | TODO |
| T-CLK-019 | forbid(unsafe) deny missing_docs unreachable_pub | §6 | T-CLK-001 | Clock | TODO |
| T-CLK-020 | CHANGELOG [Unreleased] ManualClock V2 | 中文 | T-CLK-011 | Doc | TODO |

---

## W2 — 时钟消费者迁移

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-MIG-001 | 扫描 ManualClock 旧 API 调用点 | inventory 更新 | T-CLK-013 | Migrate | TODO |
| T-MIG-002 | 迁移 crate 内测试至 V2 API | 无 set/advance/advance_mono 裸用 | T-MIG-001 | Migrate | TODO |
| T-MIG-003 | 外部调用点迁移（若出现） | 调用点=0 | T-MIG-001 | Migrate | TODO |
| T-MIG-004 | 删除 deprecated 旧 API | 公共面仅 V2 | T-MIG-002 T-MIG-003 | Clock | TODO |

---

## W3 — 删除宏与 placeholder

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-DEL-001 | 删除 xlib_test! | workspace 调用点=0；compile fixture | T-MIG-002 | Delete | TODO |
| T-DEL-002 | 删除 mock! | 调用点=0 | T-DEL-001 | Delete | TODO |
| T-DEL-003 | 删除 FixtureBuilder | 调用点=0；无 ZST placeholder | T-DEL-002 | Delete | TODO |
| T-DEL-004 | provider 宏移出 core | core 无该宏；依赖 W4 入口可用 | T-CTC-010 | Delete | TODO |
| T-DEL-005 | public API 仅 clock re-export | I-API-BUDGET；snapshot | T-DEL-004 | Delete | TODO |
| T-DEL-006 | 更新 README/AGENTS 去宏职责 | §17 | T-DEL-005 | Doc | TODO |

---

## W4 — contract-testkit

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-CTC-001 | 建 crates/test-support/contracts 包 | workspace member；publish=false；**I-DIR-CTC 树** | T-BRANCH-001 | Contract | TODO |
| T-CTC-002 | Cargo.toml 允许依赖白名单 | testkit/kernel/contracts/canonical/futures-util/tokio | T-CTC-001 | Contract | TODO |
| T-CTC-003 | ContractFailure / ContractResult | §9.6 | T-CTC-001 | Contract | TODO |
| T-CTC-004 | key_value_store suite + profile | 普通函数优先 | T-CTC-003 | Contract | TODO |
| T-CTC-005 | market_data_source suite | 从 provider 宏语义迁移+去硬编码 | T-CTC-003 | Contract | TODO |
| T-CTC-006 | instrument_catalog suite | | T-CTC-003 | Contract | TODO |
| T-CTC-007 | account_source suite | | T-CTC-003 | Contract | TODO |
| T-CTC-008 | venue_time_source suite | | T-CTC-003 | Contract | TODO |
| T-CTC-009 | execution_venue suite | | T-CTC-003 | Contract | TODO |
| T-CTC-010 | 薄声明宏（可选）只生成入口 | 无隐藏依赖；deps 在 Cargo.toml | T-CTC-005…009 | Contract | TODO |
| T-CTC-011 | reference fake 自测 | 每 suite 至少 1 通过 | T-CTC-004…009 | Contract | TODO |
| T-CTC-012 | broken fake 负测 | kill rate 100% | T-CTC-011 | Contract | TODO |
| T-CTC-013 | Binance 改用 contract-testkit | dev-dep；删 testkit provider 宏调用 | T-CTC-010 | Adapter | TODO |
| T-CTC-014 | OKX 改用 contract-testkit | 同上 | T-CTC-010 | Adapter | TODO |
| T-CTC-015 | fake/sandbox/real 分层文档 | 禁止混断言 | T-CTC-011 | Doc | TODO |
| T-CTC-016 | event_bus suite（若 contracts 有） | 按需；两消费者准入 | T-CTC-003 | Contract | DEFER |
| T-CTC-017 | README/AGENTS contract-testkit | §9 原则 | T-CTC-001 | Doc | TODO |

---

## W5 — 架构对齐 / SSOT

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-ARCH-001 | `.architecture/workspace.toml` layer=test-support | TESTKIT-LAYER-001 | T-CLK-001 | Arch | TODO |
| T-ARCH-002 | dependency.toml 路径策略更新 | testkit 不在 kernel paths 误导 | T-ARCH-001 | Arch | TODO |
| T-ARCH-003 | xtask classify Layer::TestSupport | lint-deps 不破 | T-ARCH-001 | Gate | TODO |
| T-ARCH-004 | docs/architecture/spec.md 测试平面叙述 | 非 L0 runtime | T-ARCH-001 | Doc | TODO |
| T-ARCH-005 | STRUCTURE/TECH 对齐或标滞后 | 不粉饰 | T-ARCH-004 | Doc | TODO |
| T-ARCH-006 | ADR-010 修订备注：宏退役 | 链接 002 | T-DEL-005 | Doc | TODO |
| T-ARCH-007 | active spec 唯一：I-SPEC-PATH `spec/spec.md` | active_testkit_spec_count=1 | T-DOC-002 T-DOC-003 T-DOC-005 | Doc | TODO |
| T-ARCH-008 | crates/testkit README §17.1 全点 | | T-DEL-006 | Doc | TODO |
| T-ARCH-009 | AGENTS §17.2 全点 | | T-ARCH-008 | Doc | TODO |
| T-ARCH-010 | CHANGELOG 宏退役/V2/layer | §17.3 | T-DEL-005 T-CLK-020 | Doc | TODO |
| T-ARCH-011 | publish=false 显式 | Cargo.toml | T-CLK-001 | Clock | TODO |
| T-ARCH-012 | description 去「L0 测试宏」 | | T-ARCH-008 | Doc | TODO |

---

## W6 — 防回流门禁 / CI

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-GATE-001 | xtask test-graph-check 命令 | §14 输出列齐全 | T-ARCH-003 | Gate | TODO |
| T-GATE-002 | TESTKIT-GRAPH-001..005 实现 | 五条均 fail-closed exit≠0 | T-GATE-001 | Gate | TODO |
| T-GATE-003 | archgate TESTKIT-* 规则 | §15 列表 | T-ARCH-001 | Gate | TODO |
| T-GATE-004 | public API snapshot | 超出预算 fail | T-DEL-005 | Gate | TODO |
| T-GATE-005 | production_graph_guard 测试 | crates/testkit/tests | T-GATE-002 | Test | TODO |
| T-GATE-006 | source guard：生产 src 禁 use testkit | 允许 cfg(test)/tests | T-GATE-002 | Gate | TODO |
| T-GATE-007 | CI 接线 core 命令 §16.1 | 文档或 workflow | T-GATE-003 | CI | TODO |
| T-GATE-008 | line≥95% 且 branch≥90% | llvm-cov 双阈值 | T-CLK-014 | Quality | TODO |
| T-GATE-009 | mutants ≥90% 且 I-TEST-MUT 8 禁存活 | T-GATE-017 | T-CLK-014 | Quality | TODO |
| T-GATE-010 | miri test -p testkit | 定期/CI | T-CLK-014 | Quality | TODO |
| T-GATE-011 | NAMING-001 warning 扫描 Mock* | 登记下游 | T-CTC-015 | Gate | TODO |
| T-GATE-012 | crate-standard --check 通过 | | T-ARCH-008 | Gate | TODO |

---

## W7 — 十轮实现验收

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-V10-001 | Round1 定位/layer/spec 唯一 | fail→fix | W5 W6 | Verify | TODO |
| T-V10-002 | Round2 ManualClock 合同 | I-CLK | W1 W2 | Verify | TODO |
| T-V10-003 | Round3 无宏/placeholder | I-API | W3 | Verify | TODO |
| T-V10-004 | Round4 contract-testkit 负测 | kill rate | W4 | Verify | TODO |
| T-V10-005 | Round5 图隔离 | production_dependents=0 | W6 | Verify | TODO |
| T-V10-006 | Round6 测试质量 coverage/mutation/miri | §13 | W1 W6 | Verify | TODO |
| T-V10-007 | Round7 文档 README/AGENTS/CHANGELOG | §17 | W5 | Verify | TODO |
| T-V10-008 | Round8 CI/archgate/xtask | §15–16 | W6 | Verify | TODO |
| T-V10-009 | Round9 指标 §23 全量 | I-METRICS | W6 | Verify | TODO |
| T-V10-010 | Round10 §24 交叉一致 + Evidence | fail_rounds=0 | T-V10-001…009 | Verify | TODO |
| T-EVID-001 | evidence/testkit/<date>-… | I-EVID-FILES 15 含 contract-negative-tests.log | T-V10-010 | Verify | TODO |

---

## W8 — 人审

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-HUM-001 | Spec Proposed → Approved | 自然人 | T-V10-010 | Owner | **DONE** (Approved 2026-07-14) |
| T-HUM-002 | 0.1.1 version bump 策略确认 | scripts/version.mjs | T-HUM-001 | Owner | **DONE** (0.1.1 shipped) |
| T-HUM-003 | approval-packet 决策项关闭 | A1… | T-HUM-001 | Owner | **DONE** (A1–A7 closed; A8 stable OPEN) |

---

## W9 — §24 闭合 / stable 决策

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-24-001 | 24.1 定位闭合全勾 | | T-HUM-001 | Owner | TODO |
| T-24-002 | 24.2 Core 闭合全勾 | | T-DEL-005 T-CLK | Owner | TODO |
| T-24-003 | 24.3 测试闭合全勾 | unit/property/concurrency/compile+cov/mut/miri | T-CLK-014 T-CLK-015 T-CLK-016 T-CLK-017 T-CLK-021 T-GATE-008 T-GATE-009 T-GATE-010 T-GATE-017 | Owner | TODO |
| T-24-004 | 24.4 Contract 闭合全勾 | | T-CTC-012 | Owner | TODO |
| T-24-005 | 24.5 图隔离闭合全勾 | | T-GATE-002 | Owner | TODO |
| T-24-006 | 24.6 治理闭合全勾 | RFC/ADR/snapshot/archgate/graph/neg/CHANGELOG/Evidence | T-EVID-001 T-GATE-003 T-GATE-004 T-DOC-RFC-DEL T-ARCH-006 T-ARCH-010 T-GATE-001 | Owner | TODO |
| T-24-007 | status=stable 单独决策 | 可 DEFER | T-24-001…006 | Owner | TODO |

---



## W0/W1/W4/W5/W6 补丁任务（v1.1）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-DOC-004 | 正交 Test Support 架构图 | I-1-ARCH-DIAGRAM | T-DOC-001 | Doc | TODO |
| T-DOC-005 | 终态 path I-SPEC-PATH | active=1 | T-DOC-002 T-DOC-003 | Doc | TODO |
| T-DOC-RFC-DEL | 删除 RFC/approval I-RFC-DEL | 六步 | T-DEL-005 | Doc | TODO |
| T-INV-002 | I-1-IMPLICIT + harness OOS 入 residual | residual 全表 | T-PLAN-006 | Planner | DONE |
| T-INV-003 | inventory v1.1 段齐全 | I-PATCH-v1.1 | — | Planner | DONE |
| T-CLK-021 | Clock suite 三分 | I-TEST-CLK-SPLIT | T-CLK-014 | Test | TODO |
| T-CLK-022 | non_exhaustive + 禁 signed + 禁回绕 | I-CLK-NE/NOSIGN/NOREWRAP | T-CLK-007 | Clock | TODO |
| T-CLK-023 | 失败不改状态 + 禁 one-shot | I-CLK-FAIL-ATOMIC/NO-ONESHOT | T-CLK-009 | Clock | TODO |
| T-CLK-024 | 锁→Unavailable；poison 分项 | I-CLK-LOCK-UNAVAIL/POISON | T-CLK-011 | Clock | TODO |
| T-CLK-025 | I-CLK-SIG 签名对齐 | 逐方法 | T-CLK-006 | Clock | TODO |
| T-DEL-007 | 删除门槛 WS0/EXT0/FIX/SPEC | I-DEL-* | T-DEL-001 | Delete | TODO |
| T-DEL-008 | mock 五路径 + 禁空壳宏 | I-DEL-MOCK-PATHS/NO-REPL | T-DEL-002 | Doc | TODO |
| T-DEL-009 | 硬编码清除清单 | I-DEL-HC | T-CTC-005 | Contract | TODO |
| T-DEL-010 | builder 命名规则 | I-FIXTURE-3/4 | T-DEL-003 | Doc | TODO |
| T-CTC-018 | 薄宏 cfg(test)+fixture | I-CTC-MACRO-CFG | T-CTC-010 | Contract | TODO |
| T-CTC-019 | ContractFailure 三字段 + 禁 unwrap | I-CTC-FAIL-FIELDS/NO-UNWRAP | T-CTC-003 | Contract | TODO |
| T-CTC-020 | Fake/Sandbox/Real 矩阵 | I-CTC-FAKE/SANDBOX/REAL/SEP | T-CTC-015 | Doc | TODO |
| T-CTC-021 | 禁 adapter dep + 反原则 | I-CTC-NO-ADAPTER/PRIN | T-CTC-002 | Contract | TODO |
| T-CTC-022 | 最小 profile 禁 DSL | I-CTC-MIN-PROFILE | T-CTC-004 | Contract | TODO |
| T-CTC-023 | suite_self_tests + compile_fail | I-DIR-CTC | T-CTC-011 | Contract | TODO |
| T-GATE-013 | §16.2 contract-testkit CI | I-CI-CTC | T-GATE-007 | CI | TODO |
| T-GATE-014 | §16.4 Nightly 五项 | I-CI-NIGHTLY | T-GATE-007 | CI | TODO |
| T-GATE-015 | 新模块准入八问+RFC | I-DIR-RFC | T-FREEZE-001 | Gate | TODO |
| T-GATE-016 | flaky_retry_usage=0 | I-METRICS-FLAKY | T-V10-009 | Gate | TODO |
| T-GATE-017 | mutation 禁存活 8 条 | I-TEST-MUT | T-GATE-009 | Quality | TODO |
| T-GATE-018 | Mock* 审计表 | I-TERM-AUDIT | T-GATE-011 | Gate | TODO |

### AC 收紧

- T-GATE-008: line≥95% 且 branch≥90%
- T-GATE-009: ≥90% 且 I-TEST-MUT 8 条
- T-ARCH-007: 强制 I-SPEC-PATH
- T-24-003: 依赖 T-CLK-014…017,021 + T-GATE-008…010,017
- T-24-006: 依赖 Evidence+archgate+snapshot+RFC+CHANGELOG+graph-check
- T-EVID-001: I-EVID-FILES 15 文件


## 任务统计（W0 落盘时）

| 状态 | 约数 |
|------|------|
| DONE | 8（计划包核心） |
| IN_PROGRESS | 1（T-V10-PLAN） |
| TODO | 其余实现与文档任务 |
| DEFER | T-CTC-016 等 |

实现完成前 **禁止** 将 Campaign 标为 COMPLETE 或 §24 闭合。


> v1.2：AC 绑 I-PATCH-v1.2 展开表（HC/DIR/SIG/POISON/MOCK-PATHS/LAYER/CI/SCHED）。
