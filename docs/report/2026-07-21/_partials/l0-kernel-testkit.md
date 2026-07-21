# L0/T0 Partial：kernel · testkit 生产就绪审计

| 字段 | 值 |
|------|-----|
| 审计对象 | `crates/kernel`（Cargo package **`kernel`** 0.3.0，lib `kernel`）· `crates/testkit`（Cargo package **`testkit`** 0.1.1，lib `testkit`） |
| 审计角色 | production readiness 只读审计 |
| HEAD | `9174840`（`fix(hooks): address Codex P2 on nice/timeout gate (#155)`） |
| 对照基线 | `docs/report/2026-07-21/core-crates-production-readiness.md`（§1/§5/§11/§12）· `STATUS.md` 完成度 98% · SSOT 对齐文 · `0.3.0-signoff.md` |
| 本会话验证 | `cargo test -p kernel -p testkit --all-targets` → **exit 0**（约 88 + 40 = **128** 项 all-targets；loom 默认 0 测） |
| 写出口 | 仅本 partial；未改源码/其他文档 |

> **口径**：`STATUS.md` 完成度是**结构/可观测进度**，**不是** Production Ready 签字。  
> 分层定义沿用 PLAN-CORE-PROD-002：L1 Internal / L2 Wire / L3 Contract / L4 Platform / L5 Release。  
> 文档中的历史名 `xhyper-kernel` / `xhyper-testkit` **与当前 Cargo package 名不一致**（见 P1）。

---

## 1. 模块结论表

| 模块 | 判定 | 目标层级（本仓可宣称） | 不可宣称 | 关键证据 |
|------|------|------------------------|----------|----------|
| **kernel** | **有条件就绪**（L0 语义库；非业务应用） | **L1 Internal Ready** + **L4 Platform Ready**（Linux x86_64 + MSRV 1.85 + public-api baseline） | 整体 Production Ready；crates.io 发布（`publish=false`）；非 Linux 官方支持；L2 Wire（设计上无 serde）；业务编排 / 健康检查 | SPEC-KERNEL-002 可移植子集 SSOT **无残留 FAIL**；`ClockDomain` + `wait_timeout` + loom CI 已闭合原报告 §5.1–§5.3；`docs/api-baselines/kernel.txt`；签核 `kernel → L1+L4` |
| **testkit** | **有条件就绪**（**仅** ManualClock core 测试支持） | **L1**（test-support 平面） | **生产 runtime**；integration harness；独立 contract-testkit 全套件；L2–L5 生产交付物 | SPEC-TESTKIT-002 core GAP=0；独立 domain / 中文错误 / workspace lints 已跟随 kernel；`docs/api-baselines/testkit.txt`；签核 `testkit → L1（测试支持）` |

### 1.1 七维判据速查

| 判据 | kernel | testkit |
|------|--------|---------|
| **正确性** | 强：checked 时间算术、状态机全矩阵、Shutdown Mutex+Condvar + poison 恢复 + loom 模型 + 组合根 deadline 测 | 强：checked 控制、失败不改状态、fault/snapshot 同锁、并发读控、property |
| **契约完整性** | L0 三类语义（error/clock/lifecycle）闭合；无 Component trait（刻意）；无 wire | ManualClock 族 4 类型闭合；退役宏已删；contract-testkit **不在本 crate** |
| **兼容性** | public-api baseline 门禁；`#[non_exhaustive]` ErrorKind/ComponentState/ClockError；`publish=false`；0.x | baseline 门禁；`!Default`/`!Clone`；`publish=false`；仅 dev-dep 契约（文档 + public_surface） |
| **可运维性** | 库级关停原语完备；**组合根**必须显式 `trigger` + 持有 deadline（drop 不触发） | 非 runtime；运维面 N/A |
| **安全性** | `forbid(unsafe_code)`；生产依赖仅 `thiserror`；Display/Debug 不展开 source | `forbid(unsafe_code)`；生产依赖仅 `kernel`；无真实时间/IO |
| **可验证性** | 单元 + 集成 + proptest + compile_fail/static_assertions + loom CI + coverage/miri/mutants workflows | 单元 + 合同 + property + 并发 + public_surface + coverage/miri/mutants workflows |
| **治理合规** | SSOT 对齐文 + AGENTS/CHANGELOG/README 齐；中文用户可见错误；**包名文档漂移** | 同左；明确「非生产 runtime」 |

### 1.2 相对既有报告（HEAD=9174840）是否仍适用

| 原报告项（§5 等） | 2026-07-21 #98 / post-W5 后 | 本会话核验（源码） | 结论 |
|-------------------|------------------------------|--------------------|------|
| P1 单调时钟域 | 已补 `ClockDomain` | `clock.rs` PROCESS + 跨 domain → `None`；testkit 每实例独立 domain | **已闭合**；原 P1 不再阻断 |
| P1 loom 进 CI | `kernel-loom.yml` | workflow 存在；paths 过滤 kernel/** | **已闭合**；本地默认 0 tests（需 `RUSTFLAGS='--cfg loom'`） |
| P1 关停 deadline | `wait_timeout` + 组合根测 | `lifecycle.rs` + unit 测 | **已闭合**（库级）；进程级关停策略仍归组合根 |
| P1 中文错误 | 已补 | ClockError / LifecycleError / ManualClockError Display 中文 | **已闭合** |
| P1 testkit 仅 ManualClock 评级 | 维持 | 公开面仅 4 类型；README 明示非 runtime | **仍适用** |
| P1 contract-testkit | 迁至 contracts 最小 Fake | 不在 testkit 源码 | **范围外 DEFER**（勿把 testkit 98% 当成 contract 平面） |
| §12 签核 L1+L4 / L1 | 见 `0.3.0-signoff.md` | 分层声明仍有效；整体 PR 仍否 | **仍适用** |

---

## 2. 公开 API 面与生产语义缺口

### 2.1 kernel 冻结面（`src/lib.rs` re-export）

| 模块 | 公开类型 / 能力 | 生产语义要点 |
|------|-----------------|--------------|
| `error` | `ErrorKind`（9，`non_exhaustive`）、`XError`（字段私有）、`XResult`、`BoxError` | 按**反应**分类；`is_retryable` **仅** Transient；`is_bug` **仅** Invariant；禁 `not_found`/`other`/`From<str>`；Display 不泄 source |
| `clock` | `Timestamp`、`MonotonicInstant`、`ClockDomain`、`Clock`、`SystemClock`、`ClockError` | 墙钟可回退；单调不可跨 domain 静默比较；无 Default；无 serde；SystemClock 进程共享原点 |
| `lifecycle` | `ComponentState`、`LifecycleError`、`ShutdownSignal`、`ShutdownGuard` | 合法边全矩阵；关停一次触发多方观察；**Guard drop 不触发**；`wait_timeout` 供组合根 deadline |

**`#[doc(hidden)]` 构造器**：`MonotonicInstant::from_clock_elapsed` / `from_clock_elapsed_in` — 仅供 kernel/testkit 时钟实现；**无 archgate TIME-004 机控**（SSOT DEFER），依赖约定 + 结构扫描。

### 2.2 testkit 冻结面

```text
ManualClock | ManualClockError | ManualClockFault | ManualClockSnapshot
```

- 实现 `kernel::Clock`；墙钟/单调独立；fault 不改 wall、不影响 mono  
- 无 `Default` / `Clone`；共享用 `Arc`  
- 每个实例独立 `ClockDomain`（自 100 起原子分配）

### 2.3 生产语义缺口（在**声明范围内**仍须知悉）

| ID | 缺口 | 影响 | 处置 |
|----|------|------|------|
| G-K1 | 无异步 `ShutdownSignal`（无 tokio 依赖，刻意） | 异步组合根需自适配（线程 wait / 专用任务） | 可接受设计边界 |
| G-K2 | `ShutdownGuard` drop 静默不触发 | 组合根漏 `trigger` → 关停永不发生 | 文档 + `#[must_use]`；集成测应在上层补 |
| G-K3 | `Unavailable` **不** `is_retryable` | 与部分「依赖故障可重试」直觉不一致 | 文档合同；上层策略自决 |
| G-K4 | 无 `Component` trait / 健康检查 / 重启 | 不能当服务编排框架 | 刻意非目标 |
| G-K5 | 无 wire/serde（L2 不适用） | 跨进程时间/错误格式由协议层负责 | 刻意；L2 对 kernel **不要求** |
| G-T1 | 无 integration harness / 真实时间禁止 | 不能测墙钟漂移等系统行为 | DEFER 战役 |
| G-T2 | contract-testkit 不在本 crate | 不能用 testkit 98% 证明 contracts 生产闭合 | contracts 平面单独审计 |

---

## 3. 阻断项 P0 / 改进项 P1 / 可接受风险

### 3.1 P0（阻断「按声明层级签字」）

| 项 | 说明 |
|----|------|
| **无** | 在 **kernel = L0 库 · L1+L4** 与 **testkit = ManualClock core · L1 test-support** 的声明范围内，本会话**未发现**新的正确性/安全类 P0 阻断。 |
| **边界声明** | 若将目标上调为「可 crates.io 发布的稳定平台库 / 含完整关停编排的应用框架 / 生产 runtime 时钟」，则当前 **未就绪**（`publish=false`、无 Component 编排、testkit 明确非 runtime）。 |

> 原 core 五件套报告中的 **decimalx / canonical / contracts P0** 不在本 partial 范围，但**仍阻断 workspace 级整体 Production Ready**。

### 3.2 P1（应修，不阻断 L1/L4 声明但损害可运维与诚实性）

| ID | 项 | 证据 | 建议 |
|----|----|------|------|
| P1-1 | **Cargo package 名 vs 文档名漂移** | `Cargo.toml`：`name = "kernel"` / `"testkit"`；README/AGENTS/SSOT 写 `xhyper-kernel` / `xhyper-testkit`；`cargo test -p xhyper-kernel` → **package not found** | 统一为实际 package 名，或 `package.name` 改回 xhyper-* 并全仓改依赖；CI/文档命令同步 |
| P1-2 | **CHANGELOG 与 0.3.0 签核不同步** | `ClockDomain` / loom / poison 等列在 kernel `[Unreleased]`，而 workspace 已签 `0.3.0` | 将已合入能力迁入 `0.3.0` 节或发 `0.3.1` 并清空 Unreleased 语义债 |
| P1-3 | **examples / benches 仅 `.gitkeep`** | STATUS 布局 8/8 记 ✅，但无可运行 example/bench | 接受公式虚高，或补最小 example 再刷 STATUS |
| P1-4 | **`from_clock_elapsed` 无机器 allowlist** | SSOT TIME-004 DEFER；仅结构约定 | 后续 archgate 或 `rg` CI 门禁限制调用点 |
| P1-5 | **miri / mutants 以 schedule 为主** | `kernel-miri.yml` / `*-mutants.yml`；本会话未重跑 | 重大 lifecycle/时钟 PR 建议 `workflow_dispatch` 手跑并留证据 |
| P1-6 | **testkit 生产图泄漏依赖机器门禁** | public_surface 查 Cargo.toml；全仓 xtask graph **DEFER** | 有 infra-xtask 后补 production-graph 检查 |

### 3.3 可接受风险（Accept）

| 风险 | 理由 |
|------|------|
| 仅 Linux x86_64 官方支持 | `docs/governance/support-matrix.md` + 签核 DEFER-6 Accept |
| `publish = false` | 内部 workspace 消费；非 crates.io 产品 |
| loom 不在默认 `cargo test` | 专用 workflow + 文档命令；避免拖慢主路径 |
| 锁中毒 `into_inner` 恢复 | SPEC §10.2 合同；不把 poison 当对外 panic |
| ManualClock domain 计数从 100 起 | 避开 PROCESS=1；测试隔离足够 |
| contract-testkit 不在 testkit | 平面分离正确；勿混评级 |

---

## 4. 测试与 CI 证据

### 4.1 本会话实测

```bash
# 任务指定的 xhyper-* 选择器在当前 HEAD 失败：
cargo test -p xhyper-kernel -p xhyper-testkit --all-targets
# → error: package ID specification `xhyper-kernel` did not match any packages

# 实际可复现命令（权威）：
cargo test -p kernel -p testkit --all-targets
# → exit 0
```

| 包 | 大致分解 | 合计（all-targets） |
|----|----------|---------------------|
| kernel | lib 单元 65 + `api_compile` 2 + `clock_contract` 12 + `lifecycle_concurrency` 4 + `lifecycle_concurrency_loom` **0**（无 loom cfg）+ `public_api` 5 | **~88** |
| testkit | unit 20 + `api_compile` 2 + concurrency 2 + contract 7 + properties 5 + `public_surface` 4 | **~40** |

> 未在本会话重跑：doctest、`clippy -D warnings`、llvm-cov、miri、mutants、loom release。  
> 签核会话（`70544d0`）与 SSOT 对齐文曾记录五 crate 全绿 + cov/loom；以当时证据 + 本会话 test 绿为交叉验证。

### 4.2 CI 入口（路径触发 / 调度）

| Workflow | 包 | 触发 | 作用 |
|----------|-----|------|------|
| `kernel-coverage.yml` | kernel | PR paths | 行 cov 100% gate + branch ≥90% |
| `kernel-loom.yml` | kernel | PR paths | `RUSTFLAGS=--cfg loom` lifecycle 模型 |
| `kernel-miri.yml` | kernel | schedule + dispatch | Miri |
| `kernel-mutants.yml` | kernel | schedule + dispatch | mutants |
| `testkit-coverage.yml` | testkit | PR paths | line ≥95% |
| `testkit-miri.yml` / `testkit-mutants.yml` | testkit | schedule + dispatch | Miri / mutants |
| 公共 | 两者 | `ci-rust.yml` 等 | fmt/clippy/test/MSRV；`check-public-api.mjs` baselines |

### 4.3 测试合同覆盖摘要

**kernel**

- error：9 构造器、source 链、Display/Debug 脱敏、ClockError→Unavailable  
- clock：i64 边界 checked_*、跨 domain、SystemClock 非减单调、ControlledClock 墙钟回退不牵连 mono  
- lifecycle：合法/非法边、多 observer、stress、poison、`wait_timeout`、guard drop 不触发  
- 负向：rustdoc `compile_fail` + `static_assertions`（!Default、!serde、!From&lt;str&gt;、!Component…）

**testkit**

- unit：溢出/回归不改状态、fault 映射、poison 路径、跨实例 domain  
- contract / property / concurrency / public_surface（退役符号 + prod deps 仅 `kernel`）

---

## 5. 与 STATUS 完成度（98%）的落差说明

`STATUS.md`（生成时间 2026-07-21T11:24:53Z）对两 crate 均为：

| 字段 | kernel | testkit |
|------|--------|---------|
| 完成度 | **98%** | **98%** |
| 成熟度 | `active` | `active` |
| 布局 | 8/8 | 8/8 |
| 测试 | 5i+u | 5i+u |
| 层标签 | L0 | T0 |

**公式**：`layout×50% + has_tests×25% + content×25%`（scaffold 上限 content）。

### 落差

| 维度 | 98% 完成度含义 | 生产就绪含义 | 落差 |
|------|----------------|--------------|------|
| 指标性质 | 目录齐 + 有测试 + LOC 桶 | 语义闭合 + 兼容 + 运维 + 安全 + 平台 | **不同目标函数** |
| STATUS 自声明 | 「**不是** Production Ready 签字」 | 人签 + 分层 GO-with-Accepts | 公式高分 **不**推出 PR |
| examples/benches | `.gitkeep` 即可点亮布局项 | 可运行示例/性能基线 | 结构满分 ≠ 示例交付 |
| 包名 | 不进入公式 | 可复现命令 / 跨仓引用 | **文档写 xhyper-***，**cargo 认 kernel/testkit** |
| testkit 角色 | 与 kernel 同档 98% | 测试支持 vs 生产 runtime | 同完成度 **不同生产许可** |
| DEFER | 不减分 | archgate、全仓 graph、非 Linux、miri 当场证据 | 生产签字仍要 Accept 清单 |
| 上层依赖 | 不评分 | decimalx/canonical/contracts 等仍可拖垮 **应用级** PR | L0 就绪 ≠ 应用可上线 |

**一句话**：98% 表示「crate 骨架与测试厚度接近满分」；本审计的「有条件就绪」表示「在 **L1（+kernel L4）** 分层下可作为内部依赖使用」，二者不可互换。

---

## 6. 生产签字前 checklist

### 6.1 kernel（L1 Internal + L4 Platform）

- [x] 无残留 SPEC-KERNEL-002 可移植 FAIL（对齐文）
- [x] `forbid(unsafe_code)`；生产依赖仅 `thiserror`；`default=[]`
- [x] ClockDomain 合同 + 跨 domain 不可静默可靠
- [x] Shutdown：trigger/wait/多观察者/poison/`wait_timeout`
- [x] 用户可见错误中文（Clock/Lifecycle）
- [x] public-api baseline 存在（`docs/api-baselines/kernel.txt`）
- [x] 支持矩阵声明 Linux x86_64 + MSRV 1.85
- [x] loom CI workflow 存在
- [x] 本会话 `cargo test -p kernel --all-targets` 绿
- [ ] **统一 package 名文档/命令**（P1-1，签字前建议修）
- [ ] **CHANGELOG 与 0.3.0 叙事对齐**（P1-2）
- [ ] 组合根（bootstrap/应用）集成：持有 Guard、显式 trigger、deadline 超时升级路径有测
- [ ] 关键 PR 后可选：dispatch miri/mutants 并附日志
- [ ] **禁止**勾选：crates.io 发布、非 Linux 官方、整体五 crate PR

### 6.2 testkit（L1 test-support only）

- [x] 公开面仅 ManualClock 族；退役宏守卫
- [x] 仅依赖 `kernel`；`publish=false`；`default=[]`
- [x] 独立 ClockDomain；中文 ManualClockError
- [x] unit/contract/property/concurrency/compile 绿
- [x] public-api baseline（`docs/api-baselines/testkit.txt`）
- [x] 本会话 `cargo test -p testkit --all-targets` 绿
- [ ] 文档命令改为 `-p testkit`（或更名 package）
- [ ] 消费方审计：无 normal/`build-dependencies` 引用 testkit
- [ ] **禁止**勾选：生产 runtime、integration harness 已完成、contract 平面已闭合

### 6.3 分层放行对照（签核一致性）

| 层级 | kernel | testkit |
|------|--------|---------|
| L1 Internal | **是**（有条件） | **是**（仅测试支持） |
| L2 Wire | **N/A（刻意无 wire）** | N/A |
| L3 Contract | N/A（非 trait 出口 crate） | N/A（contract-testkit 他处） |
| L4 Platform | **是**（矩阵 + baseline + loom/cov 门禁） | 部分（baseline + cov；非平台运行时） |
| L5 Release | **否**（`publish=false`；GO-with-Accepts ≠ crates.io） | **否** |

---

## 7. 最终意见（本 partial）

1. **kernel** 在 HEAD `9174840` 上仍符合「**L0 语义信任根 · 有条件就绪**」：正确性与契约在声明范围内强，平台门禁（baseline / MSRV / Linux / loom / cov）齐备；原报告时钟域与 loom P1 **已闭合且源码仍在**。剩余主要是 **治理诚实性**（包名文档漂移、CHANGELOG）与 **组合根职责**（显式 trigger + deadline），不是库内核算法 P0。  
2. **testkit** 仍只能评「**ManualClock core · 有条件就绪 · 非生产 runtime**」：core GAP=0、跟随 domain/中文/lints；不得用 STATUS 98% 或与 kernel 同档完成度暗示可进生产依赖图。  
3. **STATUS 98% ≠ 生产签字**；workspace **整体 Production Ready 仍为否**（他 crate Accept/P0 仍在）。对 kernel/testkit 的诚实对外表述应是：**内部 L1（kernel 另加 L4）可用；L5 发布与应用级生产上线另议。**

---

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初版 partial：核实 #98/post-W5 后 P1 闭合仍成立；记录 package 名漂移与 STATUS 98% 落差；本会话 test 证据 |
