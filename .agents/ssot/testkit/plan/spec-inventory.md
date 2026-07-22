# Spec Inventory — SPEC-TESTKIT-002（防遗漏枚举）

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TESTKIT-002-v1-complete` |
| 用途 | Task AC 必须引用 I-*；十轮检查对照表 |

---

## I-1 组件与路径

| ID | 项 | 规范值 |
|----|-----|--------|
| I-1.1 | testkit path | `crates/testkit` |
| I-1.2 | package | `testkit` |
| I-1.3 | plane | test-support / T0 |
| I-1.4 | contract-testkit path | `crates/test-support/contracts` |
| I-1.5 | package | `contract-testkit` |
| I-1.6 | integration harness | tools/harness · scripts/integration · CI（非 core） |

---

## I-2 依赖白名单 / 黑名单

| ID | 项 |
|----|-----|
| I-2.1 | 生产 dep 仅 `kernel`（别名 `xhyper-kernel` 已废弃） |
| I-2.2 | 禁止 contracts/canonical/decimalx/evidence/observex/configx/tokio/futures/serde/rand/proptest/mockall/rstest/tracing/anyhow |
| I-2.3 | 测试 dep 允许 proptest/static_assertions/trybuild/loom |
| I-2.4 | features default=[]；禁止 mock/async/tokio/snapshot/serde/real/integration |
| I-2.5 | 消费者仅 dev-dependencies；禁 normal/build |

---

## I-3 Crate 属性与禁止项（§6）

| ID | 项 |
|----|-----|
| I-3.1 | `forbid(unsafe_code)` |
| I-3.2 | `deny(missing_docs)` |
| I-3.3 | `deny(unreachable_pub)` |
| I-3.4 | 禁 unsafe/todo!/unimplemented!/占位 public API/生产 panic!/静默回绕/真实 sleep/真实 SystemTime·Instant/env/网络文件 IO/全局 mut/隐式 runtime/隐式 tracing |

---

## I-CLK ManualClock API 清单（§7）

| ID | 符号 | 要求 |
|----|------|------|
| I-CLK-MODEL | State | Mutex；wall；monotonic_elapsed；wall_fault |
| I-CLK-FAULT | ManualClockFault | BeforeUnixEpoch / Overflow / Unavailable → ClockError |
| I-CLK-ERR | ManualClockError | WallOverflow / MonotonicOverflow / MonotonicRegression / Synchronization + Display+Error |
| I-CLK-SNAP | ManualClockSnapshot | wall/monotonic_elapsed/wall_fault getters |
| I-CLK-NEW | new / with_monotonic_elapsed | 无 Default |
| I-CLK-WALL | set_wall / advance_wall / rewind_wall | Result；checked；失败不改状态 |
| I-CLK-MONO | set_monotonic_elapsed / advance_monotonic | 无 rewind；regression/overflow |
| I-CLK-FAULT-CTL | set/clear/wall_fault | fault 不改 wall；不影响 mono |
| I-CLK-SNAP-API | snapshot() | 同锁 |
| I-CLK-TRAIT | Clock impl | now+fault；mono poison 恢复 |
| I-CLK-CLONE | 无 Clone | 共享 Arc |
| I-CLK-BOUNDS | Send+Sync | 编译断言 |
| I-CLK-DEP | deprecated 可选 | set_unix_nanos / advance_unix_nanos + 删除清单 |

---

## I-DEL 必须删除（§8）

| ID | 项 |
|----|-----|
| I-DEL-1 | xlib_test! |
| I-DEL-2 | mock! |
| I-DEL-3 | FixtureBuilder\<T\> |
| I-DEL-4 | provider_capability_contract_tests! from core |

---

## I-CTC Contract suites（§9）

| ID | Suite 模块 |
|----|------------|
| I-CTC-1 | market_data_source |
| I-CTC-2 | instrument_catalog |
| I-CTC-3 | account_source |
| I-CTC-4 | venue_time_source |
| I-CTC-5 | execution_venue |
| I-CTC-6 | key_value_store |
| I-CTC-7 | event_bus（可选/准入） |
| I-CTC-8 | 显式 Profile；禁硬编码 server_time==0 等 |
| I-CTC-9 | ContractFailure |
| I-CTC-10 | 隐藏依赖禁止 |
| I-CTC-11 | reference fake + broken fake |

---

## I-API 公开面预算（§12）

```rust
pub use clock::{
    ManualClock,
    ManualClockError,
    ManualClockFault,
    ManualClockSnapshot,
};
```

| ID | 禁止 |
|----|------|
| I-API-1 | prelude / 宏 / 第三方 re-export / assertion DSL / 通用 mock / fixture builder / runtime executor / global context / docker / network helpers |

---

## I-TEST 测试合同（§13）

| ID | 矩阵 |
|----|------|
| I-TEST-UNIT | §13.1 全部 bullet（构造/advance/rewind/overflow/mono/fault/snapshot/失败不变/无 Default/无 Clone/Send Sync） |
| I-TEST-CLK-CONTRACT | ClockCommon / ManualClockDeterminism / SystemClockSmoke 分离 |
| I-TEST-PROP | §13.3 property |
| I-TEST-CONC | §13.4 并发 |
| I-TEST-COMPILE | §13.5 compile assertions |
| I-TEST-MUT | §13.6 mutation ≥90%；禁存活列表 |
| I-TEST-COV | line≥95% branch≥90% |
| I-TEST-MIRI | cargo miri test -p testkit |
| I-TEST-CTC-NEG | broken kill 100% |

---

## I-GRAPH / I-ARCHGATE / I-CI

> **I-AG-***（archgate 侧）：**infra.rs OOS** — 本仓不移植 archgate / `.architecture`。  
> 规则 ID 仅作历史 monorepo 参考；本仓图隔离用 I-GRAPH + test-graph-check / 结构扫描（非 archgate）。

| ID | 规则 ID | 本仓状态 |
|----|---------|----------|
| I-GRAPH-1 | TESTKIT-GRAPH-001 | 可选机控（xtask/rg） |
| I-GRAPH-2 | TESTKIT-GRAPH-002 | 可选机控 |
| I-GRAPH-3 | TESTKIT-GRAPH-003 | 可选机控 |
| I-GRAPH-4 | TESTKIT-GRAPH-004 | 可选机控 |
| I-GRAPH-5 | TESTKIT-GRAPH-005 | 可选机控 |
| I-AG-LAYER | TESTKIT-LAYER-001 | **OOS**（文档 + cargo metadata 叙述 layer） |
| I-AG-DEP | TESTKIT-DEP-001 | **OOS**（语义可用 Cargo.toml/lint-deps 表达） |
| I-AG-FEAT | TESTKIT-FEATURE-001 | **OOS** |
| I-AG-API | TESTKIT-API-001 | **OOS**（public-api 快照可独立存在） |
| I-AG-MACRO | TESTKIT-MACRO-001 | **OOS** |
| I-AG-PLACE | TESTKIT-PLACEHOLDER-001 | **OOS** |
| I-AG-TIME | TESTKIT-TIME-001 | **OOS**（可用 rg 源码守卫） |
| I-AG-IO | TESTKIT-IO-001 | **OOS** |
| I-AG-GRAPH | TESTKIT-GRAPH-001（archgate 侧） | **OOS**（图隔离见 I-GRAPH，非 archgate） |
| I-AG-HIDDEN | TESTKIT-HIDDEN-DEP-001 | **OOS** |
| I-AG-NAME | TESTKIT-NAMING-001 | **OOS** |
| I-CI-CORE | §16.1 命令全集（**无** archgate） | 生效 |
| I-CI-CTC | §16.2 | 生效 |
| I-CI-PROD | §16.3 test-graph-check | 可选/生效视落地 |
| I-CI-NIGHTLY | §16.4 | 可选 |

---

## I-DOC / I-VER

| ID | 项 |
|----|-----|
| I-DOC-README | §17.1 全部 bullet |
| I-DOC-AGENTS | §17.2 全部 bullet |
| I-DOC-CL | §17.3 宏退役/Fixture/provider/V2/layer |
| I-VER-STATUS | incubating until 闭合 |
| I-VER-LAYER | test-support |
| I-VER-BUMP | 0.1.0→0.1.1 + RFC 删除流程 |
| I-VER-PUB | publish=false |

---

## I-PHASE 迁移阶段（§19）

| ID | Phase |
|----|-------|
| I-PH-0 | 冻结 + inventory |
| I-PH-1 | ManualClock V2 |
| I-PH-2 | 迁移消费者 |
| I-PH-3 | 删无价值 API |
| I-PH-4 | contract-testkit |
| I-PH-5 | 架构对齐 |
| I-PH-6 | 防回流 |

---

## I-PR 切分（§20）

PR-1…PR-6 见 plan §1.2。

---

## I-EVID Evidence 文件（§21）

manifest / commit / cargo-metadata / consumers / production-graph / public-api.diff / fmt / clippy / tests / coverage / mutants / miri / contract-negative / **archgate（N/A · OOS）** / verdict。

---

## I-METRICS（§23）

| 指标 | 目标 |
|------|------|
| testkit_production_dependents | 0 |
| contract_testkit_production_dependents | 0 |
| testkit_public_macro_count | 0 |
| testkit_placeholder_public_type_count | 0 |
| real_time_calls_in_testkit | 0 |
| sleep_calls_in_unit_contract_tests | 0 |
| manual_clock_unchecked_arithmetic_count | 0 |
| manual_clock_line_coverage | ≥95% |
| manual_clock_branch_coverage | ≥90% |
| manual_clock_mutation_score | ≥90% |
| contract_suite_broken_impl_kill_rate | 100% |
| hidden_macro_dependency_count | 0 |
| active_testkit_spec_count | 1 |
| flaky_retry_usage | 0 |

---

## I-DONE §24 勾选枚举

### I-DONE-24.1 定位

- [ ] layer=test-support  
- [ ] 不再声明 L0 runtime  
- [ ] active spec 仅一份  
- [ ] README/AGENTS/architecture 对齐  

### I-DONE-24.2 Core

- [ ] 只依赖 kernel  
- [ ] 无 feature  
- [ ] 无宏  
- [ ] 无 FixtureBuilder  
- [ ] 无 provider suite  
- [ ] ManualClock V2  
- [ ] 无真实时间  
- [ ] 无 sleep  
- [ ] 无 unchecked arithmetic  
- [ ] 无 Clone/Default  

### I-DONE-24.3 测试

- [ ] unit / property / concurrency / compile  
- [ ] line≥95% / branch≥90% / mutation≥90% / Miri  

### I-DONE-24.4 Contract

- [ ] trait suites / 无 adapter dep / profile / broken neg / fake·sandbox·real / 无隐藏 dep  

### I-DONE-24.5 图隔离

- [ ] 全 dev-dep / 无 build-dep / release graph 无 test-support / feature 不泄漏 / machine gate  

### I-DONE-24.6 治理

- [ ] RFC·ADR / API snapshot / test-graph-check / negative fixtures / CHANGELOG / Evidence  
- ~~archgate~~ → **OOS**（infra.rs 不移植；非勾选项）

---

## I-DIR 禁止新增模块（§4.1）

util.rs / common.rs / prelude.rs / mock.rs / fixture.rs / provider.rs / integration.rs / docker.rs  

---

## I-TERM 术语（§10）

Stub / Fake / Mock / Simulator 定义；无 interaction verification 不得名 Mock。  

---

## I-DET 确定性（§11）

无真实时间 / sleep / 顺序依赖 / 全局状态 / 未固定 seed / 时区 / 开发机 env / 默认端口 / 吞后台错误 / retry 当修 flaky。  

---

## I-26 统一禁止（计划级）

见 plan §1.0 十条 Forbidden。


---

## I-PATCH-v1.1 — 计划 10x pass1 补丁（关闭 R1–R10 FAIL）

### I-1-IMPLICIT 测试不稳定隐式输入（§1 全表）

1. 真实墙钟 2. 真实单调钟 3. sleep 4. 随机数 5. 线程调度
6. 全局状态 7. 环境变量 8. 外部服务 9. 网络 10. 磁盘
11. 未版本化 fixture 12. 默认值 mock 13. 吞错宏

### I-1-ARCH-DIAGRAM

正交平面：Production graph vs Test graph。Task：`T-DOC-004`。

### I-HARNESS-OOS（§3.3）

Docker/Compose · Redis/Kafka/PostgreSQL/TDengine · 交易所 testnet · 网络故障 · 进程 kill · 真实端口 · 真实凭据 · Evidence artifact。禁止进 testkit core。

### I-FIXTURE（§3.4）

I-FIXTURE-1 领域 crate 管理 · I-FIXTURE-2 两消费者才允许 test-support/fixtures/<schema> · I-FIXTURE-3 禁 FixtureBuilder 回流 · I-FIXTURE-4 builder 须真实字段+验证

### I-DIR-CORE / I-DIR-CTC 正例树

见规范 §4.1 / §4.2；含 tests/manual_clock_*.rs、compile_fail、production_graph_guard；CTC 含 suite_self_tests、compile_fail。

### I-DIR-RFC

新模块：准入八问 + RFC。Task T-GATE-015。

### I-CLK-SIG

new(Timestamp)；with_monotonic_elapsed；set/advance/rewind_wall → Result；set/advance_monotonic → Result；fault set/clear/get；snapshot；Clock::now/monotonic。

### I-CLK-CONSTRAINTS

I-CLK-NE non_exhaustive · I-CLK-NOSIGN 禁 signed fetch_add/delta · I-CLK-NOREWRAP 禁 release 回绕 · I-CLK-FAIL-ATOMIC 失败不改状态 · I-CLK-NO-ONESHOT 禁 one-shot 队列 · I-CLK-SCRIPTED 两消费者准入 · I-CLK-LOCK-UNAVAIL 锁失败 Unavailable · I-CLK-POISON mono 恢复分项 · I-CLK-NO-DEFAULT-CLONE 共享 Arc<ManualClock>

### I-DEL-GATES

I-DEL-WS0/EXT0/FIX/SPEC · I-DEL-MOCK-PATHS 五路径 · I-DEL-NO-REPL · I-DEL-HC 硬编码清除表

### I-CTC-EXTRA

I-CTC-PRIN/NO-ADAPTER/FAKE/SANDBOX/REAL/SEP/FAIL-FIELDS/NO-UNWRAP/MIN-PROFILE/HC-TABLE/MACRO-CFG

### I-TEST-MUT 禁存活 8 条

wrapping_add · wrapping_sub · regression 反转 · fault 忽略 · clear 无效 · snapshot 错配 · mono/wall 共用 · 失败仍改状态

### I-TEST-CLK-SPLIT

ClockCommonContract · ManualClockDeterminismContract · SystemClockSmokeContract

### I-TERM-AUDIT

Mock* 无 interaction verification 登记表；T-GATE-018

### I-CI-CTC / I-CI-NIGHTLY

§16.2 三条命令 · §16.4 五项 nightly

### I-SCHED

1d→W0 · 7d→W1–W4 · 30d→W5–W9

### I-EVID-FILES 15

含 contract-negative-tests.log

### I-SPEC-PATH

`.agents/ssot/testkit/spec/spec.md`

### I-RFC-DEL

Approved RFC + 消费者清单 + 同批迁移 + CHANGELOG + API diff + compatibility

### I-METRICS-FLAKY

flaky_retry_usage=0；T-GATE-016

---

## I-PATCH-v1.2 — 全文展开（关闭 pass2 OPEN 15 项）

### I-CTC-HC-TABLE 硬编码禁令全表（F1-3 / F4-3 / F5-4）

| ID | 禁止硬编码断言 | 迁移到 |
|----|----------------|--------|
| I-CTC-HC-1 | `stream` 必须为空 / 空流即合同 | Profile `StreamExpectation` |
| I-CTC-HC-2 | `server_time == 0` | `VenueTimeProfile.expected_relation` |
| I-CTC-HC-3 | `position` 必须为空 | Account profile 显式期望 |
| I-CTC-HC-4 | `balance` 必须为空 | Account profile 显式期望 |
| I-CTC-HC-5 | `query_order == Pending` | Execution profile 显式状态期望 |
| I-CTC-HC-6 | invalid venue cancel 必须失败（作为唯一硬编码） | Execution profile 错误期望 |
| I-DEL-HC | provider 拆分时 **必须删除** 上述 1–6 在 core 宏中的字面断言 | T-DEL-009 / T-CTC-005…009 |

### I-DIR-CORE 正例树原文（F2-1）

```text
crates/testkit/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs
│   └── clock.rs
└── tests/
    ├── manual_clock_contract.rs
    ├── manual_clock_concurrency.rs
    ├── compile_fail.rs
    └── production_graph_guard.rs
```

禁止新增：util.rs · common.rs · prelude.rs · mock.rs · fixture.rs · provider.rs · integration.rs · docker.rs  
Task：`T-CLK-001` AC **必须** 对照本树；新增模块走 I-DIR-RFC + T-GATE-015。

### I-DIR-CTC 正例树原文（F2-3）

```text
crates/test-support/contracts/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── src/
│   ├── lib.rs
│   ├── market_data_source.rs
│   ├── instrument_catalog.rs
│   ├── account_source.rs
│   ├── venue_time_source.rs
│   ├── execution_venue.rs
│   └── key_value_store.rs
└── tests/
    ├── suite_self_tests.rs
    └── compile_fail.rs
```

Task：`T-CTC-001` / `T-CTC-023` AC 绑本树。

### I-CLK-DERIVE 枚举派生（F3-1）

| 类型 | 要求 |
|------|------|
| ManualClockFault | `#[non_exhaustive]` + `Debug, Clone, Copy, PartialEq, Eq` |
| ManualClockError | `#[non_exhaustive]` + `Debug` + 手写 `Display + Error`（禁 anyhow） |
| ManualClockSnapshot | `Debug, Clone, Copy, PartialEq, Eq`（字段私有） |

### I-CLK-SCRIPTED Task 绑定（F3-5）

首稳定版 **禁止** scripted / one-shot fault 队列。  
仅当 ≥2 独立消费者需要时，经准入八问 + RFC 新增。  
Task：`T-CLK-023`（文档+负向：无 one-shot API）· `T-GATE-015`（未来准入）。

### I-CLK-POISON 五条（F3-7）

`Clock::monotonic` 锁中毒可恢复策略：

1. 不在持锁期间执行调用方代码  
2. poison 时恢复 inner state  
3. 不伪造零值 `MonotonicInstant`  
4. 不 panic  
5. 文档明确恢复语义  

Task：`T-CLK-024` AC 必须逐条测/文档。

### I-CLK-SIG 完整签名矩阵（F3-8）

| 方法 | 完整签名 |
|------|----------|
| new | `pub fn new(initial_wall: Timestamp) -> Self` |
| with_monotonic_elapsed | `pub fn with_monotonic_elapsed(initial_wall: Timestamp, monotonic_elapsed: Duration) -> Self` |
| set_wall | `pub fn set_wall(&self, wall: Timestamp) -> Result<(), ManualClockError>` |
| advance_wall | `pub fn advance_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError>` |
| rewind_wall | `pub fn rewind_wall(&self, delta: Duration) -> Result<Timestamp, ManualClockError>` |
| set_monotonic_elapsed | `pub fn set_monotonic_elapsed(&self, elapsed: Duration) -> Result<(), ManualClockError>` |
| advance_monotonic | `pub fn advance_monotonic(&self, delta: Duration) -> Result<MonotonicInstant, ManualClockError>` |
| set_wall_fault | `pub fn set_wall_fault(&self, fault: ManualClockFault) -> Result<(), ManualClockError>` |
| clear_wall_fault | `pub fn clear_wall_fault(&self) -> Result<(), ManualClockError>` |
| wall_fault | `pub fn wall_fault(&self) -> Result<Option<ManualClockFault>, ManualClockError>` |
| snapshot | `pub fn snapshot(&self) -> Result<ManualClockSnapshot, ManualClockError>` |
| Clock::now | `fn now(&self) -> Result<Timestamp, ClockError>` |
| Clock::monotonic | `fn monotonic(&self) -> MonotonicInstant` |
| Snapshot::wall | `pub const fn wall(&self) -> Timestamp` |
| Snapshot::monotonic_elapsed | `pub const fn monotonic_elapsed(&self) -> Duration` |
| Snapshot::wall_fault | `pub const fn wall_fault(&self) -> Option<ManualClockFault>` |

Task：`T-CLK-025`。

### I-DEL-MOCK-PATHS 五路径展开（F4-2）

| # | 场景 | 迁移 |
|---|------|------|
| 1 | 简单 fake | 消费方手写结构体 |
| 2 | trait fake | 消费方实现目标 trait |
| 3 | 多实现共同契约 | `contract-testkit` suite |
| 4 | 调用记录 | 消费方 `Arc<Mutex<Vec<Call>>>` |
| 5 | 复杂 expectation | 先证明需求，再评审专用工具 |

禁止：用另一个生成空壳的宏替代 `mock!`。Task：`T-DEL-008`。

### I-CTC-LAYER-MATRIX Fake / Sandbox / Real（F5-3）

| 层 | 必须验证 |
|----|----------|
| **Fake** | trait object 可调用；明确配置输入输出；错误注入；生命周期；调用记录；无外部 IO |
| **Sandbox** | 本地真实服务；真实协议；可重复 fixture；隔离 namespace；cleanup |
| **Real/Testnet** | 真实远端；网络；凭据；延迟；限流；API 漂移 |

三类 **不得** 同一组默认断言混用。Task：`T-CTC-020`。

### I-TERM-AUDIT 冻结表（F6-3 · 2026-07-14 workspace 扫描）

| 符号模式 | 扫描结论 | 处置 |
|----------|----------|------|
| `MockBinance*` / adapter Mock* | 实现波扫描登记 | T-GATE-018 产出表；首期 warning |
| `MockKv*` / `Mock*Store` | 同上 | 改名 Fake/InMemory 候选 |
| `testkit::mock!` 生成类型 | 仅 testkit 自测 DummyMock | W3 删除 |

基线命令：`rg -n 'struct Mock|mock!' --glob '*.rs' crates/`  
无 interaction verification 的 Mock 命名 → NAMING-001。

### I-CI-CTC 三条命令字面（F7-1）

```bash
cargo clippy -p contract-testkit --all-targets -- -D warnings
cargo test -p contract-testkit
cargo test -p contract-testkit --test negative_implementations
```

Task：`T-GATE-013`。Plan §6 必须列本节。

### I-CI-NIGHTLY 五项字面（F7-2）

1. full mutation（`cargo mutants -p testkit` 全量）  
2. Miri（`cargo miri test -p testkit`）  
3. property test extended cases  
4. contract suite broken-implementation matrix  
5. workspace production graph audit（`test-graph-check` 全 binary）  

Task：`T-GATE-014`。

### I-SCHED 里程碑展开（F9-2）

| 窗 | 规范交付 bullet | Wave | gap 状态 |
|----|-----------------|------|----------|
| **1 天** | incubating/2.5；冻结宏与 FixtureBuilder；消费者清单；批准 T0；起草 ManualClock V2 API；production graph 检查原型 | W0 | PARTIAL→目标 CLOSED 于 W0 退出 |
| **7 天** | V2 完成；消费者迁移；xlib_test/mock/Fixture 删除；provider 移出 core；contract-testkit 骨架+KV suite；broken fake 可杀 | W1–W4 | PARTIAL |
| **30 天** | provider suites 完整；Binance/OKX fake·sandbox·real；Mock 命名审计；test graph gate 完整；mutation/Miri/property 入 CI；release graph 无 test-support；stable 验收候选 | W5–W9 | PARTIAL |

gap-matrix §22 必须为 **PARTIAL**（非 N/A）。
