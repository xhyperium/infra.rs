# testkit SSOT 对齐与本仓落地状态

| 字段 | 值 |
|------|-----|
| 策略 | **B — 本仓移植 core testkit** |
| 日期 | 2026-07-21；**contract-testkit 加厚复核 2026-07-23** |
| 规范 | SPEC-TESTKIT-002（镜像 `.agents/ssot/testkit/spec/spec.md`） |
| package | **`testkit`** · lib `testkit`（Cargo 选择器 `-p testkit`；历史名 `xhyper-testkit` 已废弃） |
| 当前版本 | testkit 0.1.2；contract-testkit 0.1.2 候选（均为 test-support）|
| 内部生产层级 | **L1 ManualClock test-support**（**不是**生产 runtime；PR #159 · tag `v0.3.0-four-crates`） |

## 结论摘要

| 问题 | 状态 |
|------|------|
| 上游 SSOT 镜像 COMPLETE 叙事 | 仍是 xhyper 战役文档；**禁止**单独当作本仓交付证明 |
| 本仓 `crates/testkit` core（ManualClock 族） | **已闭合**（§7 / §13.1–§13.5 / §24.1–§24.3 core / §24.5 core → 见 clause matrix） |
| 内部生产 GO（声明层级） | **L1 test-support only**；证据 [`../plans/releases/2026-07-21-four-crates-internal-release.md`](../plans/releases/2026-07-21-four-crates-internal-release.md) |
| 本仓 `contract-testkit` | **0.1.2 候选**：Fake + 14 trait suite + 15 broken case + `FixtureNamespace` + **Batch-2 Fake** + **BackendProfile**；见 [contracts-ssot-alignment.md](./contracts-ssot-alignment.md) |
| integration harness | **PASS**：`crates/testkit/src/harness.rs` · `IntegrationHarness` / `StepRecord`（**仅测试**；非生产 runtime） |
| ClockDomain 跟随 | **PASS**：每 `ManualClock` 实例独立 domain；跨实例 `checked_duration_since` → `None` |
| 用户可见错误中文 | **PASS**：`ManualClockError` Display 中文 |
| `[lints] workspace = true` | **PASS** |
| public-api 棘轮 | **PASS**：`docs/api-baselines/testkit.txt` |
| 公开面 / 示例 / bench | **PASS**：`tests/public_api_surface.rs` · `examples/basic.rs` · `benches/hot_path` |
| 本仓质量证据 | **本仓实测** line-cov / mutants / miri；不复制上游 `2026-07-14-stable-gates` |

## 本仓可观察事实

```text
crates/testkit/                 EXISTS
Cargo.toml members              含 crates/testkit
package name                    testkit
lib name                        testkit
publish                         false
prod deps                       kernel only
dev deps                        proptest, static_assertions
features.default                []
examples                        examples/basic.rs
API baseline                    docs/api-baselines/testkit.txt
IntegrationHarness              src/harness.rs（多步确定性集成 harness）
```

## 验证命令（本仓可复现）

```bash
# 功能与合同
cargo test -p testkit --all-targets
cargo clippy -p testkit --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p testkit --example basic
cargo bench -p testkit --bench hot_path -- --quick
node scripts/quality-gates/check-public-api.mjs

# §13.7 line coverage（≥95%；CI 为 100% gate）
cargo llvm-cov -p testkit --fail-under-lines 95 --summary-only

# §13.6 mutants（目标 score≥90%；本仓期望 missed=0）
mkdir -p .cargo/cache/mutants
cargo mutants -p testkit --timeout 60

# §13.8 Miri
cargo +nightly miri test -p testkit
```

CI 入口（与 kernel 同级 paths 过滤）：

| Workflow | 触发 | 路径 |
|----------|------|------|
| `.github/workflows/testkit-coverage.yml` | push/PR（paths） | line ≥95% |
| `.github/workflows/testkit-miri.yml` | schedule + dispatch | miri test |
| `.github/workflows/testkit-mutants.yml` | schedule + dispatch | cargo mutants |

## Clause matrix（本仓证据，非镜像勾选）

> 标记：`PASS` = 本仓源码 + 可运行测试/命令证明；`GAP` = core 必选缺口；`DEFER` = 显式范围外或工具/依赖阻塞。  
> **core 必选 GAP 必须为 0** 才可宣称 core 对齐。

### §7 ManualClock 合同

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| 7.1 | 独立墙钟/单调钟、fault、snapshot、多线程、无真实时间、无静默溢出 | PASS | `src/clock.rs` + unit/contract/concurrency |
| 7.2 | `Mutex<State>{wall, mono, fault}` 一致模型 | PASS | `clock.rs` State |
| 7.3 | `ManualClockFault` 三变体映射 ClockError | PASS | unit `fault_variants_map_to_clock_error` |
| 7.4 | `ManualClockError` + 中文 Display/Error，无 anyhow | PASS | unit `error_display_*` |
| 7.14 | 独立 `ClockDomain`；跨实例间隔不可靠 | PASS | `domain()` + `cross_manual_clock_domain_*` |
| 7.5 | Snapshot 私有字段 + 只读 getter | PASS | unit `snapshot_getters_*` |
| 7.6 | `new` / `with_monotonic_elapsed`；无 Default | PASS | unit 构造 + `api_compile` `!Default` |
| 7.7 | wall set/advance/rewind checked；失败不改状态；可回拨 | PASS | unit wall_* + property advance/rewind |
| 7.8 | mono set/advance；regression；overflow；无 rewind | PASS | unit mono_* + property mono_advance_checked |
| 7.9 | fault set/clear/query；不改 wall；不影响 mono | PASS | unit fault_* + property fault_set_clear_sequence |
| 7.10 | snapshot 同锁读三字段 | PASS | unit snapshot_* + concurrency |
| 7.11 | `Clock` impl；poison 恢复文档语义 | PASS | `impl Clock` + lock_recover 文档 |
| 7.12 | 无 Clone；共享用 Arc | PASS | `api_compile` `!Clone` + concurrency Arc |
| 7.13 | Send + Sync compile assertion | PASS | `tests/api_compile.rs` |

### §13 测试合同（core）

| ID | 条款 | 状态 | 本仓证据 |
|----|------|------|----------|
| 13.1 | unit：构造/advance/rewind/边界/overflow/fault/snapshot/失败不变/!Default/!Clone/Send+Sync | PASS | `src/clock.rs` unit_tests + api_compile |
| 13.2 | ManualClockDeterminism：now/error、可 rewind、mono 非减、独立、无漂移、无 control 值不变 | PASS | `tests/manual_clock_contract.rs` |
| 13.3 | property：wall±、mono+、失败 snapshot 不变、fault sequence | PASS | `tests/manual_clock_properties.rs` |
| 13.4 | concurrency：多线程读/控、Arc、无撕裂 | PASS | `tests/manual_clock_concurrency.rs` |
| 13.5 | compile assertions：!Default/!Clone/Send+Sync；不导出退役符号 | PASS | `tests/api_compile.rs` + `tests/public_surface.rs` |
| 13.6 | mutation score ≥90% | PASS | 本仓 `cargo mutants`：missed=0（caught=10, unviable=20） |
| 13.7 | line ≥95%；branch ≥90% OPTIONAL | PASS / DEFER | line **99.65%** PASS；branch 本工具 summary 无分支数据 → **OPTIONAL/DEFER**（与上游 residual 一致） |
| 13.8 | Miri | PASS | `cargo +nightly miri test -p testkit`（见 evidence） |
| 13.9 | contract-testkit 自测 | PASS（候选分支） | reference `suite_self_tests.rs` + 14 trait / 15 case `negative_implementations.rs`；逐项断言精确 contract/case |

### §24 验收清单（core 相关）

| ID | 条款 | 状态 | 说明 |
|----|------|------|------|
| 24.1 | layer=test-support；非 L0 runtime；单 active spec；README/AGENTS 对齐 | PASS | `publish=false`、README/AGENTS、镜像只读 |
| 24.2 | 只依赖 kernel；无 feature/宏/FixtureBuilder/provider；ManualClock V2；无真实时间/sleep/unchecked；无 Clone/Default | PASS | Cargo.toml + public_surface + api_compile + 实现 |
| 24.3 | unit / property / concurrency / compile | PASS | 见 §13 |
| 24.3 | line ≥95% | PASS | llvm-cov |
| 24.3 | branch ≥90% | DEFER | OPTIONAL（上游 residual；本仓不升强制） |
| 24.3 | mutation ≥90% | PASS | missed=0 |
| 24.3 | Miri | PASS | 本仓 miri 日志 |
| 24.4 | Contract 闭合 | PARTIAL→加厚 | **contract-testkit 0.1.2 候选**覆盖 14 trait；ObjectStore 精确 payload、TimeSeries 点包含、Analytics/Instrumentation observer-aware；EventBus/PubSub 可移植 surface 仅 smoke，旧 EventBus profile 保持兼容，真实后端深度仍 OPEN |
| 24.0-h | Integration harness | **PASS** | `src/harness.rs` · `IntegrationHarness::{new,step,run,clock}` + unit；导出见 `src/lib.rs` |
| 24.5 | 消费为 dev-dep / 无 build-dep / 无 normal graph 泄漏 | PASS（候选分支） | `check-test-support-graph.mjs` 基于 cargo metadata 检查 default/all-features normal/build 闭包、完整路径与 inventory fail-closed |
| 24.5 | feature 不泄漏 | PASS | `default=[]` 无其它 feature |
| 24.6 | 治理（RFC / xtask…） | PARTIAL | CHANGELOG + Evidence 本仓已有；经 archgate 的治理机控 **OOS**（本仓明确不移植 archgate / `.architecture`） |

### 退役 API（§8 / §3.1）

| 符号 | 状态 |
|------|------|
| `xlib_test!` | PASS（源码无定义；public_surface 守卫） |
| `mock!` | PASS |
| `FixtureBuilder` | PASS |
| `provider_capability_contract_tests!` | PASS |

## 与镜像文档的关系

- `.agents/ssot/testkit/**`：只读镜像；禁止本地改 CLOSED/COMPLETE 叙事冒充同步
- 实现 SSOT 以 **源码 + 本仓测试/覆盖率/mutants/miri 输出** 为准
- 详见 `.agents/ssot/SSOT.md` R6 / R7 与根 `AGENTS.md`
- Workspace 总览（members 地图、依赖方向）：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| integration harness | DEFER | **PASS** | `crates/testkit/src/harness.rs` · `IntegrationHarness` |
| contract-testkit Batch-2 / backend profile | DEFER（挂 contracts 侧） | **PASS**（见 contracts 对齐） | `crates/test-support/contracts/src/fakes/batch2.rs` · `backend.rs` |

## 未做（follow-up / 诚实边界）

- Sandbox / Real/Testnet 后端合同与 live evidence；Fake/self-test 不升级为 readiness
- EventBus/PubSub 的可移植 delivery、replay、order、ack、backpressure、投递次数；当前 portable surface 只做 subscribe/publish smoke，旧 EventBus profile 不外推
- ObjectStore 覆盖/删除/列表/跨进程持久化与 TimeSeries 排序/重复语义；端点闭合只属于兼容 ClosedPoint profile，不外推到可移植窗口入口
- branch coverage ≥90% 强制（OPTIONAL residual）
- 上游 SSOT 文档内部 STALE 收口（应在 xhyper.rs 修，再镜像同步）
- **Agent L5 / Production Ready 人签** — 未填

## Core 必选 GAP 计数

```text
core 必选 GAP = 0
```

## 跟进（2026-07-21 生产就绪）

| 项 | 状态 |
|----|------|
| contracts 平面 | **已存在**（纠正旧文「缺 contracts」） |
| 独立 contract-testkit | **已落地**：`crates/test-support/contracts` |
| contracts 生产语义 | **部分闭合**（Tx/消息 + live helpers）；全 trait 深度仍 OPEN |
| ManualClock × ClockDomain | **PASS**；跨实例比较 → `None` |
| 中文 Display / workspace lints | **PASS** |
| 四包内部 GO（#159） | **L1 test-support**；tag `v0.3.0-four-crates`；**≠** 生产 runtime |
| 整体 Production Ready | **否**；见 [core-crates-production-readiness.md](../report/2026-07-21/core-crates-production-readiness.md) §11 |

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-23 | contract-testkit 0.1.2 候选：14 trait / 15 broken case、确定性 fixture、图门禁与 API baseline；待 PR/CI/人工批准 |
| 2026-07-22 | **defer-close**：`IntegrationHarness` PASS；contract-testkit Batch-2/backend 交叉引用 |
| 2026-07-22 | SSOT 同步文档纠偏：与 #178 落地叙事全仓对齐 |
| 2026-07-21 | 独立 `contract-testkit` crate 落地（`crates/test-support/contracts`） |
| 2026-07-21 | 生产就绪文档同步：contracts 存在性、domain、中文错误、PARTIAL contract-testkit；PR #98 **合入 main** |
| 2026-07-21 | 四包内部 GO：package 名对齐 `testkit`；全部 `cargo -p testkit`；examples/surface/baseline；PR #159 · tag `v0.3.0-four-crates` |
