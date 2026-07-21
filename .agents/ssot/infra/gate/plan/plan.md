# Plan — PLAN-GATE-RETIRE-001 完整执行计划（v1）

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-GATE-RETIRE-001-v1-complete` |
| Source Plan | [`.agents/ssot/infra/gate/xhyper-gate-retirement-complete-plan.md`](../plan/xhyper-gate-retirement-complete-plan.md) |
| Source Spec | [`.agents/ssot/infra/gate/gate-spec.md`](../gate-spec.md)（active；退役后 Superseded） |
| Source Plan ID | `PLAN-GATE-RETIRE-001` · Status **Proposed** |
| Goal | `GOAL-GATE-RETIRE-PLAN-PACKAGE`（本包：计划完备性 + 10x + todo + 对齐） |
| Package | `xhyper-gate` / lib `gate` @ `crates/gate` **0.1.0** |
| Decision | **Retire and delete** runtime gate；**保留** CI/arch/policy gates |
| Replacement | `bootstrap` + typed `PlatformContext` / `AppContext` / `BootstrappedApp` / `BootstrapBuilder` |
| Method | Strangler Fig + small-batch（PR-1…PR-5） |
| Baseline | `main@41c59584`（开战役）· 本包分支 `docs/gate-retirement-plan-package` |
| Source Inventory | [`source-inventory.md`](./source-inventory.md)（I-1…I-28 防遗漏枚举） |
| Gap Matrix | [`gap-matrix.md`](./gap-matrix.md) |
| Consumer Inventory | [`consumer-inventory.md`](./consumer-inventory.md)（live `cargo tree` + source search） |
| Tasks | [`tasks.md`](./tasks.md) |
| Residual | [`residual-open.md`](./residual-open.md) |
| Approval Packet | [`approval-packet.md`](./approval-packet.md) · **人审签字 OPEN** |
| Work Todo | [`.worktree/gate-todo.md`](../../../../.worktree/gate-todo.md) |
| 10x Verdict | [`gate-plan-10x-verdict.md`](./gate-plan-10x-verdict.md) |
| Alignment | [`docs/audits/gate-plan-alignment-2026-07-15.md`](../../../../../docs/audits/gate-plan-alignment-2026-07-15.md) |
| Strategy | **冻结 → 批准 RFC/ADR → typed bootstrap → 迁移消费者 → 移除 API → 删 crate → 防回流** |
| Campaign status | **PLAN PACKAGE COMPLETE · 10x fail_rounds=0** · **≠ crate deleted** · **≠ RFC/ADR Approved** · **≠ production retirement DONE** |
| Forbidden | 见 **I-28 / 源 §20**（移动目录 / TypeId registry / Any downcast / sealed 后留 registry / 并入 kernel / 复制进 bootstrap / 为插件预建 / Big Bang） |

---

## 0. 深度分析结论（对照 PLAN-GATE-RETIRE-001 §0–§20）

### 0.1 最终裁定（源 §0 · 不可改写）

```text
只删除：
  crates/gate
  package: gate（xhyper-gate）
  runtime Gate / Capability / register / resolve

必须保留：
  .agent/gates/
  tools/archgate/
  CI gate jobs
  release gates
  xlibgate / policy gate 概念
```

**运行时 gate crate** 与 **CI/架构门禁 gate** 是两个不同概念。退役前者，不得误删后者。

### 0.2 为什么必须退役（源 §1）

| ID | 缺陷 | 证据 |
|----|------|------|
| D-1 | `Capability` 仅 `name()`，无业务方法 | `crates/gate/src/lib.rs` |
| D-2 | 字符串 `resolve` 失去编译期完整性 | 同左 |
| D-3 | register 非事务：先写 HashMap 再 evidence | 源 §1.3 |
| D-4 | `Gate::new()` evidence=None；bootstrap 默认绕过 | 源 §1.4 |
| D-5 | build 后仍可 `register`（共享引用） | 源 §1.5 |
| D-6 | 错误语义全落 Invalid | 源 §1.6 |
| D-7 | path=infra 却 layer=L0 | 源 §1.7 |
| D-8 | 与 bootstrap 双组合中心 | 源 §1.8；R3.1 bootstrap 唯一组装豁免 |

### 0.3 目标架构（源 §2–§4）

```text
contracts 定义「能做什么」
adapters 定义「谁来实现」
bootstrap 决定「本次进程使用哪个实现」
AppContext 只读暴露「已组装完成的依赖」
```

**正确替代（typed only）**：

```text
PlatformContext { instrumentation, shutdown_signal [, evidence later] }
AppContext { platform: PlatformContext }
BootstrappedApp { context, shutdown: ShutdownController }
BootstrapBuilder / Bootstrap { with_instrumentation → build → BootstrappedApp }
```

**禁止替代**：

```text
HashMap<TypeId, Box<dyn Any>>
HashMap<String, Arc<dyn Capability>>
get(name) / resolve(name) / register(...) / insert / downcast
```

Bounded contexts（`MarketDataContext` / `ExecutionContext`）按真实服务需要后置，**不**一次塞进全局 AppContext。

### 0.4 现状一句话（2026-07-15 live inventory）

| 项 | 状态 |
|----|------|
| package | `xhyper-gate` 0.1.0 @ `crates/gate`（workspace member） |
| 生产依赖者 | **仅** `xhyper-bootstrap`（`cargo tree -i xhyper-gate`） |
| 运行时调用 | bootstrap `src/lib.rs` + unit tests + `tests/e2e.rs` |
| 真实 service 经 gate 取 Binance/Redis | **未发现**（e2e 已直接用 contracts trait） |
| 同源词干扰 | `VenueSafetyGate`（domain）≠ runtime `gate`；`tools/archgate` ≠ crate |
| Source Plan Status | **Proposed** |
| RFC / ADR 退役正文 | **未 Approved**（人审 OPEN） |
| crate 物理删除 | **未执行**（本战役 out of scope） |

### 0.5 本战役边界（诚实）

| 在范围内 | 不在范围内 |
|----------|------------|
| 完整执行计划包 + inventory + tasks | 物理删除 `crates/gate` |
| 10 轮计划完备性（adversarial） | RFC/ADR 人审签字关闭 |
| `.worktree/gate-todo.md` | PR-2…PR-5 代码 cutover |
| 对齐文档（诚实状态） | CI job 改名 `policy-gates` |
| live 消费者存证 | 宣称 retirement DONE |

```text
plan 10x PASS ≠ crate deleted ≠ RFC/ADR Approved ≠ production retirement DONE
```

---

## 1. 执行策略与原则

```text
1. 证据优先：PASS 绑定命令输出或 evidence/ 文件；禁止手写 PASS
2. 冻结先于重构：no-new-gate 在任何 API 删除前启用
3. Strangler Fig：先加 typed 替代，再迁消费者，再删旧 API，最后删 crate
4. 单 writer：plan/ 与 gate-todo 路径互斥；verifier 只写 round-N / verdict
5. residual 纪律：OPEN / CLOSED / DEFER(accepted) only
6. 禁止：TypeId/Any registry、Big Bang、误删 .agent/gates / archgate
7. 人审闸门：RFC/ADR/物理删除不得由 AI 独断 CLOSED
8. 十轮验收：fail_rounds 必须为 0 才可宣称「十轮通过」
9. 分支纪律：禁止 main 直接开发；实现波用 `.worktree/workspaces/<branch>`
10. 命名消歧：文档始终区分 runtime gate crate vs CI/arch gates
```

### 1.1 路径互斥（agent team）

| Writer 角色 | 独占写路径 |
|-------------|------------|
| Planner | `plan/plan.md`, `source-inventory.md`, `gap-matrix.md`, `tasks.md` |
| Inventory | `consumer-inventory.md`, Evidence 快照 |
| Residual/Approval | `residual-open.md`, `approval-packet.md` |
| Todo | `.worktree/gate-todo.md` |
| Alignment | `docs/audits/gate-plan-alignment-*.md` + CLAUDE/AGENTS 指针 |
| Verifier R1–R10 | `round-NN-findings.md`, `gate-plan-10x-verdict.md`（只读其余） |

---

## 2. Wave / Phase 映射

| Wave | 源 Phase | 内容 | PR | 实现状态（本战役） |
|------|----------|------|-----|-------------------|
| W0 | Phase 0 | 冻结 + 完整盘点 + RFC 起草 | PR-1 | **计划包 DONE**；guard/RFC 正文 **OPEN** |
| W1 | Phase 1 | typed PlatformContext / BootstrappedApp | PR-2 | **计划 DONE**；代码 **OPEN** |
| W2 | Phase 2 | 迁移 bootstrap 测试与消费者 | PR-3 | **计划 DONE**；代码 **OPEN** |
| W3 | Phase 3 | 移除 gate API / bootstrap 去依赖 | PR-4 | **计划 DONE**；代码 **OPEN** |
| W4 | Phase 4 | 删除 crate + workspace + registry/spec | PR-5 前半 | **计划 DONE**；代码 **OPEN** |
| W5 | Phase 5 | 防回流 guards + negative fixtures | PR-5 后半 | **计划 DONE**；代码 **OPEN** |
| W6 | — | 计划 10x + 对齐 + residual 诚实台账 | 本包 | **DONE**（fail_rounds=0） |

Issue 前缀（源 §6.3）：

```text
GATE-RETIRE-00 … GATE-RETIRE-06
```

映射 Task ID 见 [`tasks.md`](./tasks.md)。

---

## 3. Phase Exit Gates → Task 映射（无遗漏）

### 3.1 Phase 0 Exit Gate（源 §7.5）

| Exit checkbox | Task ID |
|---------------|---------|
| consumer inventory 完成 | T-INV-001 |
| cargo tree 结果存证 | T-INV-002 |
| source search 结果存证 | T-INV-003 |
| external downstream 检查完成 | T-INV-004 |
| no-new-gate guard 已启用 | **T-FREEZE-002**（规则登记见 T-FREEZE-001；Exit 字面「已启用」= CI/xtask 落地） |
| RFC 已起草 | T-GOV-001 |

### 3.2 Phase 1 Exit Gate（源 §8.6）

| Exit checkbox | Task ID |
|---------------|---------|
| typed AppContext 已存在 | T-BOOT-001…003 |
| BootstrappedApp 管理 shutdown owner | T-BOOT-004 |
| 新 API 有单元测试 | T-BOOT-010 |
| 没有引入新 Service Locator | T-BOOT-011 + T-GUARD 负向 |
| 旧 API 未新增调用点 | T-FREEZE-001 持续 |
| public API diff 已审阅 | T-BOOT-012 |

### 3.3 Phase 2 Exit Gate（源 §9.5）

| Exit checkbox | Task ID |
|---------------|---------|
| `use gate::` 生产调用为 0 | T-MIG-001 |
| `Gate::` 生产调用为 0 | T-MIG-001 |
| `Capability` 生产实现为 0 | T-MIG-002 |
| register_capability 调用为 0 | T-MIG-003 |
| AppContext::gate 调用为 0 | T-MIG-004 |
| bootstrap e2e 使用 typed contracts | T-MIG-005 |
| 所有消费者测试通过 | T-MIG-010 |

### 3.4 Phase 3 Exit Gate（源 §10.4）

| Exit checkbox | Task ID |
|---------------|---------|
| bootstrap 不依赖 gate | T-RM-001 |
| AppContext 不含 Gate | T-RM-002 |
| public bootstrap API 无 register/resolve | T-RM-003 |
| cargo tree -i 无生产 gate 依赖 | T-RM-004 |
| compat 使用点为 0 或有期限清单 | T-RM-005 / T-COMPAT-001 |

### 3.5 Phase 4 Exit Gate（源 §11.8）

| Exit checkbox | Task ID |
|---------------|---------|
| crates/gate 不存在 | T-DEL-001 |
| workspace member 不存在 | T-DEL-002 |
| cargo metadata 无 package gate / xhyper-gate | T-DEL-003 |
| Cargo.lock 无 workspace gate | T-DEL-004 |
| active architecture registry 无 gate | T-DEL-005 |
| active spec 无 gate（移出/Superseded） | T-DEL-006 |
| STRUCTURE.md 无 runtime gate | T-DEL-007 |

### 3.6 Phase 5 Exit Gate（源 §12.5）

| Exit checkbox | Task ID |
|---------------|---------|
| dependency guard | T-GUARD-001 |
| source guard | T-GUARD-002 |
| negative fixtures | T-GUARD-003…007 |
| public API snapshot | T-GUARD-008 |
| architecture drift check | T-GUARD-009 |
| no false positive on CI policy gates | T-GUARD-010 |

### 3.7 Done §19.1–19.5 → Task 覆盖

| §19 | 主题 | 覆盖 Task 前缀 |
|-----|------|----------------|
| 19.1 消费者闭合 | tree/source/downstream/compat | T-INV-* T-MIG-* T-RM-* T-COMPAT-* |
| 19.2 架构闭合 | sole bootstrap / typed / no locator | T-BOOT-* T-GUARD-* |
| 19.3 物理闭合 | 删除 crate/member/lock/registry/spec/STRUCTURE | T-DEL-* |
| 19.4 治理闭合 | RFC/ADR/CHANGELOG/API/guards/Evidence | T-GOV-* T-DOC-* T-EVID-* T-GUARD-* |
| 19.5 语义闭合 | CI 保留 / 命名消歧 / policy / rollback | T-KEEP-* T-DOC-* T-RB-* |

---

## 4. 删除清单 vs 保留清单（源 §5 · 强制）

### 4.1 删除（runtime only）

```text
crates/gate/          # 整树
package xhyper-gate / lib gate
workspace member "crates/gate"
bootstrap → xhyper-gate dependency
gate::Capability, gate::Gate
Gate::new / with_evidence / with_evidence_and_clock
Gate::register / resolve
Bootstrap::register_capability
AppContext::gate
gate mock feature
active .agents/ssot/infra/gate/  # 移出 active；非物理 shred 历史
architecture registry active gate unit
```

### 4.2 必须保留

```text
.agent/gates/               # harness 门禁规格
tools/archgate/             # 架构门禁工具
CI gate / policy gate jobs
release gates
quality / evidence gates
docs 中关于 CI gate 的叙述
VenueSafetyGate（domain 风险门）
xlibgate 历史叙述（只读）
```

CI job `gate` → `policy-gates` **非阻塞**（源 §5.2）；单独 backlog，**不**阻塞 runtime 删除。

---

## 5. 目标 API 摘要（源 §3 · 实现波必须遵守）

见源 §3.1–§3.5。硬约束：

```text
- PlatformContext / AppContext 字段全强类型
- BootstrappedApp 拥有 ShutdownController（不可复制 guard）
- BootstrapBuilder 无通用 register API
- BootstrapError → Missing / Invalid / Unavailable 映射
- 禁止 get/resolve/register/insert/Any/downcast/HashMap registry
```

---

## 6. 验证矩阵（源 §13 · 实现波执行）

### 6.1 静态

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace   # 或 cargo nextest run
cargo xtl lint-deps
cargo run -p archgate -- --json
cargo machete
cargo deny check
```

### 6.2 聚焦

```bash
cargo test -p bootstrap
cargo test -p bootstrap --test e2e
```

### 6.3 删除证明（Phase 4+）

```bash
# package name 是 xhyper-gate；lib name 是 gate — 两者均不得残留为 runtime member
cargo metadata --format-version 1 | jq -e '.packages[] | select(.name == "xhyper-gate" or .name == "gate")' && exit 1 || true
rg -n 'use gate::|gate::Gate|gate::Capability|register_capability' --glob '*.rs' .
```

注意：`cargo tree -i gate` **会失败**——package ID 是 `xhyper-gate`。计划与脚本一律使用 `xhyper-gate`。

### 6.4 行为

```text
AppContext build 成功；instrumentation 可用；shutdown 可触发；
build 后无 mutation 入口；contracts trait 可用；缺依赖 build 失败。
```

---

## 7. Evidence 布局（源 §14）

```text
evidence/architecture/gate-retirement/<phase>/
├── manifest.json
├── commit.txt
├── cargo-metadata-before.json / after.json
├── cargo-tree-before.txt / after.txt
├── consumer-inventory.md
├── source-search-before.txt / after.txt
├── public-api.diff
├── architecture.diff
├── test.log · clippy.log · archgate.json
├── negative-fixtures.log
├── downstream-impact.md
└── verdict.md
```

本战役（plan package）最小存证：

```text
evidence/gate-retirement/plan-package-2026-07-15/
├── baseline-commit.txt
├── cargo-tree-xhyper-gate.txt
├── source-search-runtime.txt
└── plan-package-verdict.md
```

---

## 8. 回滚（源 §15）

| 阶段 | 回滚 |
|------|------|
| Phase 1–2 | revert 当前 PR |
| Phase 3 | revert bootstrap removal PR |
| Phase 4 | revert deletion PR |
| 禁止 | 临时再实现 registry / 复制 Gate 进 bootstrap / 关 archgate / 永久 exception |

gate **无生产持久化状态** → 无数据迁移/回滚。若发现隐藏状态消费者：停止删除阶段并重新评估。

---

## 9. PR 切分（源 §16）

| PR | 内容 | 行为变化 |
|----|------|----------|
| PR-1 | RFC/ADR · inventory · no-new-gate · status retiring | 无行为变化 |
| PR-2 | PlatformContext · BootstrappedApp · typed tests | 旧 gate 仍可用 |
| PR-3 | 迁 unit/e2e/消费者 · 去掉 register/gate 调用 | 测试路径变 |
| PR-4 | bootstrap 去 gate dep · 删 fields/accessors | 破坏性 API |
| PR-5 | 删 crate · registry/spec/docs · STRUCTURE · negative fixtures · final evidence | 物理闭合 |

每个 PR：独立 worktree · 非 main · main green · 明确 rollback · Evidence · 不混 evidence/kernel 重构。

---

## 10. 时间盒（源 §17）与指标（源 §18）

| 窗口 | 目标 |
|------|------|
| 1 天 | RFC/ADR 草案 · inventory · freeze · retiring 标记 · 最小 typed API · **不删 crate** |
| 7 天 | typed 完成 · 调用为 0 · bootstrap 去 dep · API/arch 检查 · crate 待删 |
| 30 天 | 删 crate · negative fixtures · evidence 接入 PlatformContext · 清零 compat |

目标指标（源 §18，全部 = 终态）：

```text
runtime_gate_package_count = 0
runtime_service_locator_count = 0
string_capability_lookup_count = 0
typeid_registry_count = 0
gate_runtime_dependents = 0
post_build_dependency_mutation_entrypoints = 0
compat_gate_consumers = 0
typed_required_dependency_coverage = 100%
negative_fixture_pass_rate = 100%
active_specs_claiming_gate_is_L0 = 0
```

---

## 11. Forbidden（源 §20 · 完整保留）

**不要执行**：

```text
- 将 gate 移到 crates/gate；
- 将字符串 key 改成 TypeId；
- 给 Capability 增加 Any/downcast；
- 添加 sealed/frozen 后继续保留 registry；
- 把 gate 合并进 kernel；
- 把 gate 原样复制进 bootstrap；
- 为尚不存在的动态插件需求预建框架；
- 一次性 Big Bang 删除后再修编译。
```

**执行路径**：

```text
PR-1 冻结 → PR-2 Typed Bootstrap → PR-3 迁移 → PR-4 删 API → PR-5 删 crate + 防回流
```

终态：

```text
contracts 表达依赖
adapters 实现依赖
bootstrap 组装依赖
typed context 暴露依赖
compiler 验证依赖
```

---

## 12. 本包交付物清单（计划战役 Done）

| # | 产物 | 路径 |
|---|------|------|
| 1 | 主计划 | `plan/plan.md` |
| 2 | 源节 inventory | `plan/source-inventory.md` |
| 3 | gap matrix | `plan/gap-matrix.md` |
| 4 | 消费者 live 存证 | `plan/consumer-inventory.md` |
| 5 | 原子任务表 | `plan/tasks.md` |
| 6 | residual | `plan/residual-open.md` |
| 7 | 人审 packet | `plan/approval-packet.md` |
| 8 | 10 轮 findings | `plan/round-01…10-findings.md` |
| 9 | 10x verdict | `plan/gate-plan-10x-verdict.md` |
| 10 | work todo | `.worktree/gate-todo.md` |
| 11 | 对齐审计 | `docs/audits/gate-plan-alignment-2026-07-15.md` |
| 12 | Evidence 快照 | `evidence/gate-retirement/plan-package-2026-07-15/` |

**不**将下列项标为本包 DONE：物理删除、RFC/ADR Approved、§19 全勾、生产 retirement。
