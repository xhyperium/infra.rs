> **历史执行计划（2026-07-14，非当前权威）**：本文件保留 `xhyper-testkit 0.1.1` 战役与当时验收计数，不描述当前 `testkit 0.1.3` runner/allocator 边界。当前入口见 [../README.md](../README.md)。

# Plan — SPEC-TESTKIT-002 完整执行计划（v1）

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-TESTKIT-002-v1-complete` |
| Plan version | **v1.2.0** |
| Source Spec | [`.agents/ssot/testkit/spec/spec.md`](../spec/spec.md) |
| Spec ID | `SPEC-TESTKIT-002` · Status **Approved** → Target **Stable** |
| Goal | `GOAL-DETERMINISTIC-TEST-SUPPORT` |
| Package | `testkit` @ `crates/testkit` **0.1.1**（`publish = false`） |
| Plane | **T0 / Test Support Plane**（非生产分层；正交于 L0–L2.5） |
| Baseline | `main` HEAD 对齐点 + 本战役分支（见 §1.1） |
| Gap Matrix | [`gap-matrix.md`](./gap-matrix.md) |
| Spec Inventory | [`spec-inventory.md`](./spec-inventory.md)（I-1…I-N 防遗漏枚举） |
| Tasks | [`tasks.md`](./tasks.md) |
| Residual | [`residual-open.md`](./residual-open.md) |
| Approval Packet | [`approval-packet.md`](./approval-packet.md) |
| Work Todo | [`.worktrees/testkit-todo.md`](../../../../.worktrees/testkit-todo.md) |
| 10x Verdict | [`testkit-plan-10x-verdict.md`](./testkit-plan-10x-verdict.md) |
| Strategy | **诚实台账 → 冻结错误扩散 → ManualClock V2 → 退役无价值 API → contract-testkit 拆分 → 图隔离门禁 → 架构 SSOT 去重 → 十轮验收 → 人审** |
| Campaign status | **STABLE** · W0–W6 · 0.1.1 · Spec **Stable** · mutation missed=0 · Miri PASS · CI testkit-quality |
| Forbidden | 见 §1.0 十条；假 Done / registry stable / SKIP=PASS / testkit 进 normal graph / 空 mock 当合同 |

---

## 0. 深度分析结论（对照完整规范 §0–§25）

### 0.1 testkit 是什么

`testkit` 的生产终态不是「测试工具大全」，也不是当前的：

```text
xlib_test! + mock! + FixtureBuilder + provider 大宏 + Atomic ManualClock
```

而是与生产依赖图正交的 **T0 deterministic test support**：

```text
testkit
  = runtime-neutral deterministic primitives
  = 当前只保留 ManualClock（Mutex 状态模型 + checked API + fault + snapshot）

contract-testkit
  = 面向 contracts trait 的可复用一致性套件（独立 crate）

integration harness
  = 真实基础设施 / 网络 / 进程 / 故障（不在 testkit core）
```

核心价值：

```text
把隐式输入变成显式、可注入、可推进、可检查的测试状态。
```

### 0.2 准入八问（任一否决则不得进 testkit core）

1. 至少两个独立 workspace 消费者需要？
2. 测试语义在消费者之间完全一致？
3. 不依赖具体业务领域？
4. 不连接外部服务？
5. 不要求具体 async runtime？
6. 能显著提高确定性或故障路径覆盖？
7. 直接手写在消费者中会导致重复错误？
8. API 可以长期稳定？

不满足 → 留消费方 / `contract-testkit` / integration harness。

### 0.3 现状一句话

当前 `crates/testkit` 是 **incubating prototype**：Atomic 双钟 `ManualClock`（可控 wall/mono，但无 checked、无 fault、无 snapshot）+ 无价值宏/占位 + 隐藏依赖的 provider 大宏。依赖图上 **仅 kernel** 正确；**身份错误地挂在 L0/kernel layer**；**不得**标记 stable。

### 0.4 章节级现状（摘要；明细见 gap-matrix）

| 区间 | 主题 | 综合状态 |
|------|------|----------|
| §0–§3 | 定位 / 组件划分 / Fixture 所有权 | **WRONG**（仍写 L0；无 contract-testkit；无 harness 边界落地） |
| §4–§6 | 目录 / 依赖合同 / crate 规则 | **PARTIAL**（仅 lib.rs 单文件；无 forbid/deny；依赖白名单碰巧正确） |
| §7 | ManualClock 完整合同 | **WRONG**（Atomic 模型；unchecked；无 fault/snapshot/Result 控制 API；有 Clone 风险未禁 Default） |
| §8 | 宏退役 | **ABSENT 迁移**（宏与 placeholder 仍导出） |
| §9 | Contract Testkit | **ABSENT** |
| §10 | Mock/Fake/Stub 术语 | **ABSENT**（下游 Mock* 命名未审计） |
| §11–§13 | 确定性 / API 预算 / 测试合同 | **PARTIAL**（少量 unit；无 property/concurrency/compile_fail/mutation/Miri） |
| §14–§16 | 图隔离 / Archgate(OOS) / CI | **PARTIAL**（lint-deps 有 testkit dev-only 粗检；test-graph-check 可选；**archgate 规则全集 = OOS**，本仓不移植） |
| §17–§18 | 文档 / 版本 | **WRONG**（README 仍 L0 + 宏清单；layer=kernel；status incubating 正确） |
| §19–§20 | 迁移 / PR 切分 | **文档有；实现无** |
| §21–§24 | Evidence / 时间表 / 指标 / 完成定义 | **文档有；实现无** |
| §25 | 最终裁定 | **方向正确；未执行** |

### 0.5 硬阻断（未关则禁止 stable）

1. **DEF-001** layer 仍为 `kernel` / 文档仍写 L0 runtime  
2. **DEF-002** ManualClock 非 Mutex 一致快照模型；unchecked 算术  
3. **DEF-003** 无 wall fault / snapshot / 控制 API `Result`  
4. **DEF-004** `xlib_test!` / `mock!` / `FixtureBuilder` 仍公开  
5. **DEF-005** `provider_capability_contract_tests!` 隐藏依赖 + 硬编码 mock 行为仍在 core  
6. **DEF-006** 无 `contract-testkit` 与 trait-level suite / broken fake 负测  
7. **DEF-007** 图隔离机控不全（test-graph-check 等）；**archgate TESTKIT-* = OOS**（本仓不移植，不作为阻断）  
8. **DEF-008** 双/三份 active-looking specs（testkit-spec / complete-spec / testkitx）未收敛  
9. **DEF-009** Spec 仍 Proposed；人审未批  
10. **DEF-010** 覆盖率/mutation/Miri/API snapshot 未达标

### 0.6 消费者冻结清单（W0 实测 · 2026-07-14）

| 符号 / 依赖 | 调用点 | 路径 |
|-------------|--------|------|
| `provider_capability_contract_tests!` | 2 | `crates/adapters/exchange/binance/src/lib.rs` · `okx/src/lib.rs` |
| `xlib_test!` | 仅 testkit 自测 | `crates/testkit/src/lib.rs` |
| `mock!` | 仅 testkit 自测 | `crates/testkit/src/lib.rs` |
| `FixtureBuilder` | 仅 testkit 自测 | `crates/testkit/src/lib.rs` |
| `ManualClock` 外部 | **0** | （仅 crate 内 unit） |
| Cargo `testkit` dep | 2 consumers | binance / okx **dev-dependencies** ✓ |
| normal graph 污染 | **0 已知** | 需机控持续保证 |

**结论**：宏删除对外部主要阻塞是 provider 大宏（Binance/OKX）；ManualClock V2 可在无外部调用点时较安全推进（仍须 deprecated 迁移路径 + API 快照）。

### 0.7 kernel Clock 对齐

`kernel` 已稳定提供：

```text
Timestamp::{from_unix_nanos, as_unix_nanos, checked_add, checked_sub}
MonotonicInstant::from_clock_elapsed
ClockError::{BeforeUnixEpoch, Overflow, Unavailable}
Clock::{now -> Result, monotonic -> MonotonicInstant}
```

`ManualClockFault` 映射表与 kernel 已一致；testkit **不得**自创平行错误语义。

### 0.8 与旧规范冲突

| 旧文档 | 问题 | 处理 |
|--------|------|------|
| `.agents/ssot/testkit/spec/TESTKIT-SPEC-001.superseded.md` | 仍要求 `xlib_test!`/`mock!` 为职责 | Superseded → 指向 002（原 `testkit-spec.md`） |
| `.agents/ssot/testkitx/testkitx-spec.md` | 历史 L1 路径幽灵 | Superseded / archive |
| `docs/architecture/spec.md` | L0 列表含 testkit；职责写宏 | W5 对齐修订 |
| ADR-010 | 批准宏最小范围 | W5 修订备注：002 退役宏 |
| ADR-012 | harness/testkitx 合并叙述 | 交叉引用 002 终态 |
| `tools/xtask` classify | testkit → Layer::Kernel | 改为 test-support / 专用类（若本仓有 xtask） |
| `.architecture/workspace.toml` | monorepo 曾 `layer = "kernel"` | **OOS**：infra.rs **不维护** `.architecture`；layer=test-support 以文档 + cargo metadata 叙述 |

---

## 1. 执行策略与原则

### 1.0 Forbidden（十条）

1. 假 Done / 无命令输出勾选  
2. registry / package 标 stable 而 §24 未全勾  
3. SKIP 计 PASS  
4. `testkit` / `contract-testkit` 进入 production normal/build graph  
5. 空 `mock!` / 零字段 `FixtureBuilder` 回流  
6. 隐藏宏依赖（调用方偶然提供 tokio/canonical/contracts）  
7. 用 SeqCst 多 atomic 宣称 snapshot 一致  
8. 真实 sleep / SystemTime / Instant 进入 testkit core  
9. AI 独断 Spec Approved / 0.1.1 发布决策  
10. 在 main 直接开发  

### 1.1 分支与 worktree

```text
plan 分支:   docs/testkit-002-plan
实现分支:    feat/testkit-002-clock-v2  （PR 栈后续）
worktree:    .worktrees/workspaces/<branch-name>
base:        origin/main HEAD
```

禁止在 `main` 或无关战役分支（如 evidence-002 实现枝）混写 testkit 实现。

### 1.2 PR 栈（对齐规范 §20）

| PR | 内容 | 退出 |
|----|------|------|
| **PR-1** | Spec 冻结 · inventory · layer proposal · no-new-placeholder · plan 包 | 文档可审；机控冻结可选 |
| **PR-2** | ManualClock V2 + tests | `cargo test -p testkit`；旧 API deprecated |
| **PR-3** | 消费者迁移（若有）+ 删 deprecated | 调用点 0 |
| **PR-4** | 删 xlib_test!/mock!/FixtureBuilder + API snapshot | public macro count=0 |
| **PR-5** | contract-testkit + Binance/OKX 迁移 + broken fakes | suite 可杀 broken |
| **PR-6** | 架构 SSOT · test-graph-check · Evidence（**archgate OOS**） | §24 候选 |

每个 PR：独立 worktree、可回滚、main green、不混入 kernel/evidence 大改。

### 1.3 Spec Inventory 强制引用

实现 Task 的 AC **不得**只写「见 §xx」。必须引用 `spec-inventory.md` 中的 **I-*** 项（ManualClock API 表、测试矩阵、门禁 ID、指标、完成勾选项）。

### 1.4 P0–P6 ↔ Wave

| Spec 阶段 | Wave | 前缀 |
|-----------|------|------|
| Phase 0 冻结 | W0 | T-FREEZE / T-INV / T-PLAN / T-DOC |
| Phase 1 ManualClock V2 | W1 | T-CLK |
| Phase 2 时钟迁移 | W2 | T-MIG |
| Phase 3 删宏/placeholder | W3 | T-DEL |
| Phase 4 contract suites | W4 | T-CTC |
| Phase 5 架构对齐 | W5 | T-ARCH / T-DOC |
| Phase 6 防回流 | W6 | T-GATE / T-CI |
| 验收 | W7 | T-V10 |
| 人审 | W8 | T-HUM |
| §24 闭合 | W9 | T-24 |

---

## 2. 波次（Waves）与依赖 DAG

```text
W0  台账/冻结/消费者清单/计划 10x     ──┐
W1  ManualClock V2（Mutex + checked） ──┼──→ W2  迁移旧 API 调用点
W3  删除宏/FixtureBuilder              ──┤
W4  contract-testkit + 负测            ──┤
W5  架构/文档/spec 去重                ──┼──→ W6 图隔离门禁 + CI
W7  十轮实现验收 ×10                   ──┤
W8  人审 Approved + 0.1.1              ──┘
W9  §24 全勾 + 可选 stable 决策
```

| Wave | 名称 | 可并行 | Owner | 退出条件 |
|------|------|--------|-------|----------|
| **W0** | 冻结 + 计划包 + 10x plan | docs 并行 | Planner | inventory 完整；forbid 新增宏；plan/todo 一致；计划 10x fail_rounds=0 |
| **W1** | ManualClock V2 | 否 | Clock Agent | Mutex 模型；checked；fault；snapshot；无 Default/Clone；§13.1 单测矩阵 |
| **W2** | 迁移时钟消费者 | 依赖 W1 | Migrate | 旧 API 调用点 0 或仅 deprecated 内部 |
| **W3** | 删除无价值 API | 依赖 W2；与 W4 可部分并行若 provider 已迁出 | Delete | macro=0；placeholder=0；API snapshot |
| **W4** | contract-testkit | 可与 W1 文档并行；代码依赖 provider 迁出路径 | Contract | trait suites；profile；broken kill rate 100% |
| **W5** | 架构对齐 | 依赖 W3/W4 方向明确 | Doc/Arch | active spec=1；layer=test-support；README/AGENTS |
| **W6** | 门禁 + CI | 依赖 W1–W5 | Gate | TESTKIT-* 机控；test-graph-check；production graph=0 |
| **W7** | 十轮实现验收 | 串行 10 轮 | Verify | fail_rounds=0；Evidence 目录 |
| **W8** | 人审 | 人 | Owner | Spec Approved；version 策略 |
| **W9** | §24 闭合 | 人+机 | Owner | 24.1–24.6；stable 单独决策 |

### 2.1 本会话 AI 默认可执行范围

| 可立即执行 | 需人决策 / 外部 |
|------------|-----------------|
| W0 完整计划包 + todo + inventory | Spec Status → Approved |
| W0 消费者冻结扫描 | registry stable |
| 计划完备性十轮检查 | mutation CI 资源 / Miri nightly 策略 |
| 标记旧 spec Superseded（文档） | 0.1.1 tag 策略 |
| 起草 ManualClock V2 模块骨架（feature 分支） | 下游 Mock* 全量 rename 节奏 |
| residual 台账 | contract-testkit 首批 suite 优先级（KV vs venue） |

**本轮默认目标**：落盘完整计划 + todo + **十轮无遗漏检查（计划完备性）**。**不**宣称 §24 闭合或实现完成。

---

## 3. 路径互斥与 Agent Team 分片

| 分片 | 可写路径 | 禁写 |
|------|----------|------|
| A Planner | `.agents/ssot/testkit/plan/**` · `.worktrees/testkit-todo.md` | `crates/**` 实现 |
| B Clock | `crates/testkit/src/clock.rs` · `crates/testkit/tests/**` | adapters |
| C Delete/Migrate | `crates/testkit/src/lib.rs` · binance/okx 测试入口 | kernel |
| D Contract | `crates/test-support/contracts/**`（新建） | testkit core 宏回流 |
| E Gate | `tools/xtask/**` · `.agent/gates/**`（**`.architecture/**` OOS：本仓不维护**） | domain 业务 |
| F Doc | `README` · `AGENTS` · `docs/architecture/**` · CHANGELOG | 改 API 语义 |

单 writer：同一路径不同时并行写。

---

## 4. ManualClock V2 实现要点（W1 摘要）

```text
pub struct ManualClock { state: Mutex<State> }
struct State {
  wall: Timestamp,
  monotonic_elapsed: Duration,
  wall_fault: Option<ManualClockFault>,
}
```

必须交付（I-CLK 全集）：

- 构造：`new(Timestamp)` · `with_monotonic_elapsed`；**无 Default**
- 墙钟：`set_wall` / `advance_wall` / `rewind_wall` → `Result`；checked；失败不改状态
- 单调：`set_monotonic_elapsed` / `advance_monotonic`；禁 rewind；regression 错误
- Fault：`set_wall_fault` / `clear_wall_fault` / `wall_fault`；不影响 mono / 不改 wall 值
- Snapshot：同锁临界区
- Clock：`now` 映射 fault；`monotonic` poison 恢复语义文档化
- **无 Clone**；共享用 `Arc`
- Send+Sync 编译断言
- 可选短期 `#[deprecated] set_unix_nanos / advance_unix_nanos` + 删除清单

禁止：多 atomic 拼 snapshot；真实 sleep 验证；静默 wrapping。

---

## 5. contract-testkit 要点（W4 摘要）

```text
path:    crates/test-support/contracts
package: contract-testkit
deps:    testkit, kernel, contracts, canonical, futures-util, tokio
```

- 按 trait 拆 suite；禁「provider capability」一锅宏  
- 优先普通函数 + 显式 Profile；薄宏仅生成测试入口  
- 每个 suite：reference fake 过 + broken fake 必须失败  
- 隐藏依赖禁止  
- 仅 dev-dependency 消费  

首批建议优先级：

1. `key_value_store`（存储适配器多）  
2. 从现有 provider 宏拆出的 market/instrument/account/time/execution（Binance/OKX 阻塞）  

---

## 6. 门禁与 CI（W6 摘要）

### 6.1 图隔离（§14）

```text
TESTKIT-GRAPH-001..005
```

`cargo xtl test-graph-check`（新建）输出：package / consumer / kind / target / feature path / verdict。

### 6.2 Archgate（§15）— **infra.rs OOS**

> **本仓不移植** `tools/archgate` / `.architecture`。下列 TESTKIT-* 仅为历史 monorepo 规则 ID 参考，**不**强制机控、**不**构成本仓验收。  
> 可选残留：结构扫描 / lint-deps / test-graph-check / CI / public-api（均非 archgate）。

历史规则 ID 参考：

```text
TESTKIT-LAYER-001, DEP-001, FEATURE-001, API-001, MACRO-001,
PLACEHOLDER-001, TIME-001, IO-001, GRAPH-001, HIDDEN-DEP-001, NAMING-001
```

### 6.3 Core CI 命令（§16.1）

```bash
cargo fmt -- --check
cargo clippy -p testkit --all-targets -- -D warnings
cargo test -p testkit
cargo llvm-cov -p testkit --fail-under-lines 95
cargo mutants -p testkit
cargo miri test -p testkit
# N/A (infra.rs OOS): cargo run -p archgate -- --json  — 本仓不引入 archgate
cargo run -p xtask -- lint-deps
cargo run -p xtask -- crate-standard --check
cargo run -p xtask -- test-graph-check
```

---



### 6.3b Contract-testkit CI（§16.2 · I-CI-CTC）

```bash
cargo clippy -p contract-testkit --all-targets -- -D warnings
cargo test -p contract-testkit
cargo test -p contract-testkit --test negative_implementations
```

Task：T-GATE-013。

### 6.3c Nightly（§16.4 · I-CI-NIGHTLY）

1. full mutation  2. Miri  3. property extended  4. broken-impl matrix  5. workspace production graph audit  
Task：T-GATE-014。

## 7. Evidence 合同（§21）

每次 testkit 变更：

```text
evidence/testkit/<date>-<change-id>/
  manifest.json, commit.txt, cargo-metadata.json, consumers.json,
  production-graph.json, public-api.diff, fmt.log, clippy.log, tests.log,
  coverage.json, mutants.json, miri.log, contract-negative-tests.log,
  archgate.json  # N/A（infra.rs 不引入 archgate；不构成本仓验收）,
  verdict.md
```
（I-EVID-FILES：archgate.json 本仓 **N/A/OOS**；其余齐全）

证明：commit；未进 production graph；broken fake 可杀；无真实时间；无宏回流；SKIP≠PASS。

---

## 8. 指标与完成定义

见规范 §23–§24 与 inventory **I-METRICS** / **I-DONE-24.***。

仅当 **24.1–24.6 全勾 + 人审** 才允许：

```text
Progress = 3/3 · Quality = 5/5 · Status = stable
```

本计划 W0 结束时允许的最大声明：

```text
Campaign = PLANNING
Plan 10x = PASS (fail_rounds=0)
Implementation = NOT STARTED / ABSENT
```

---

## 9. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| Binance/OKX provider 宏删除过早 | 测试红 | PR-5 先迁 contract-testkit 再删 core 宏 |
| 误改 kernel Clock | 全仓回归 | testkit 只消费；不改 trait 签名 |
| layer 改 test-support 破坏 classify | lint-deps 假红 | W6 同步 classify + 矩阵 |
| Atomic→Mutex 性能恐慌 | 无 | 测试路径非热路径；规范已裁定 |
| 双 spec 并存误导 agent | 错误实现 | W0 Superseded 页眉 + active 唯一路径 |
| 在 evidence 分支混提 | 审查污染 | 独立 `docs/testkit-002-plan` / feat 枝 |

---

## 10. 验收（本计划包自身）

- [x] 覆盖规范 §0–§25 映射到 Wave/Task  
- [x] 消费者 inventory 有实测路径  
- [x] DEF / residual 可追踪  
- [x] 十轮计划检查可重复  
- [ ] 实现 §24（W7–W9）  

---

## 11. 变更日志

完整变更日志见 [§15](#15-变更日志)（v1.0.0 → v1.3.0）。

## 12. 附录：§22 时间表 ↔ Wave

| 窗 | Wave |
|----|------|
| 1 天 | W0 |
| 7 天 | W1–W4 |
| 30 天 | W5–W9 |

## 13. 附录：Clock 合同三分（§13.2）

ClockCommonContract · ManualClockDeterminismContract · SystemClockSmokeContract  
Task T-CLK-021 · 共享 `Arc<ManualClock>`

## 14. Ship record（2026-07-14）

| 项 | 值 |
|----|-----|
| Spec | **Approved** |
| Package | testkit **0.1.1** |
| Tag | `testkit-v0.1.1` |
| Main merges | #247 docs · #254 impl stack · #255 release |
| Active SSOT | `.agents/ssot/testkit/spec/spec.md` |
| Stable | **NOT** claimed — residual §24 |

## 15. 变更日志

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0.0 | 2026-07-14 | 初版 |
| v1.1.0 | 2026-07-14 | 计划 10x pass1 补丁后 |
| v1.2.0 | 2026-07-14 | pass2 OPEN 15 项全文展开 |
| v1.3.0 | 2026-07-14 | Spec Approved · ship W0–W6 · testkit 0.1.1 |
