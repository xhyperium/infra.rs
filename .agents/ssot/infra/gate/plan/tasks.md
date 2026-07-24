# Tasks — PLAN-GATE-RETIRE-001 原子任务表

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-GATE-RETIRE-001-v1-complete` |
| Status enum | `TODO` · `IN_PROGRESS` · `DONE` · `BLOCKED` · `DEFER` · `CANCELLED` |
| Baseline | `main@41c59584` |

> 完成定义：每条 Task 的 AC 必须可机器或文件证据验证；`DONE` 禁止无输出。  
> 依赖列使用 Task ID。路径互斥见 plan §1.1。  
> **本战役（计划包）**仅将 W0 计划文档类与 W6 验收标 DONE；实现 Task 保持 TODO/OPEN。

---

## W0 — 计划包 / 冻结登记 / 盘点 / 治理起草

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-PLAN-001 | 落盘 plan.md | 含 §0–§12；delete/keep；Forbidden；Exit Gate 映射 | — | Planner | **DONE** |
| T-PLAN-002 | 落盘 source-inventory.md | I-1…I-28 全 Mapped | — | Planner | **DONE** |
| T-PLAN-003 | 落盘 gap-matrix.md | D-1…D-8 + 实现缺口 + 治理缺口 | — | Planner | **DONE** |
| T-PLAN-004 | 落盘 tasks.md | 覆盖 Phase0–5 Exit + §19 | — | Planner | **DONE** |
| T-PLAN-005 | 落盘 residual-open.md | OPEN/CLOSED/DEFER only | T-PLAN-003 | Planner | **DONE** |
| T-PLAN-006 | 落盘 approval-packet.md | 人审闸清晰；AI 不可独断 | — | Planner | **DONE** |
| T-INV-001 | consumer inventory 文档 | prod/dev/test/docs/specs/CI 面 | — | Inventory | **DONE**（plan-time live） |
| T-INV-002 | cargo tree -i xhyper-gate 存证 | 文件 evidence + consumer-inventory | — | Inventory | **DONE**（plan-time） |
| T-INV-003 | source search 存证 | 排除 VenueSafetyGate 误报 | — | Inventory | **DONE**（plan-time） |
| T-INV-004 | external downstream 检查 | 私有仓内无外部 package 依赖证据；文档登记 | T-INV-001 | Inventory | **DONE**（plan-time：仓内 only） |
| T-FREEZE-001 | 登记 no-new-gate 规则 + 扫描命令 | residual + plan；**启用**脚本/门禁属实现 | — | Freeze | **DONE**（登记）；启用 **TODO**→T-FREEZE-002 |
| T-FREEZE-002 | 落地 no-new-gate 临时门禁（rg/CI；**不**依赖 archgate） | `cargo xtl no-new-gate` PASS；CI lint-deps 步；runner P1；archgate **N/A（OOS）** | T-FREEZE-001 | Freeze | **DONE** |
| T-GOV-001 | 起草 RFC Retire Runtime Gate | 答满源 §6.1 八问 | T-INV-001 | Gov | **DONE**（Proposed） |
| T-GOV-002 | 起草 ADR Bootstrap Sole Composition Root | 源 §6.2 七点 | T-GOV-001 | Gov | **DONE**（Proposed） |
| T-GOV-003 | RFC/ADR 人审 Approved | approval-packet 签字 | T-GOV-001 T-GOV-002 | Human | **DONE**（A12 除外 Keep-OPEN） |
| T-TODO-001 | 更新 `.worktrees/gate-todo.md` | Waves/PR/residual/人审/Next | T-PLAN-* | Planner | **DONE** |
| T-ALIGN-001 | 对齐文档包 | audits + CLAUDE/AGENTS 诚实状态 | T-PLAN-001 | Doc | **DONE** |
| T-V10-000 | 计划完备性十轮 | fail_rounds=0；verdict 文件 | T-PLAN-* T-TODO-001 | Verifier | **DONE** |
| T-BRANCH-001 | 非 main 分支/worktree | docs/gate-retirement-plan-package | — | Lead | **DONE** |
| T-PROC-001 | PR 纪律写入 plan | worktree/非 main/Evidence/rollback | — | Lead | **DONE** |
| T-KEEP-001 | 保留 `.agent/gates/` 写入 plan + residual | plan §4.2 + residual 明示 KEEP | — | Planner | **DONE** |
| T-KEEP-002 | ~~保留 `tools/archgate/`~~ | **CANCELLED / OOS**：**infra.rs 不引入** `tools/archgate`（非本仓 KEEP） | — | Planner | **CANCELLED（OOS）** |
| T-KEEP-003 | 保留 CI gate / policy gate jobs | plan §4.2 + residual | — | Planner | **DONE** |
| T-KEEP-004 | 保留 release gates | plan §4.2 + residual | — | Planner | **DONE** |
| T-KEEP-005 | 保留 xlibgate / policy gate 概念叙述 | plan §4.2 + T-DOC-002 | — | Planner | **DONE** |
| T-EVID-000 | plan-package Evidence 快照 | evidence/gate-retirement/plan-package-2026-07-15/ | T-INV-002 | Inventory | **DONE** |

---

## W1 / Phase 1 — Typed Bootstrap（PR-2）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-BOOT-001 | PlatformContext 结构 | instrumentation + shutdown_signal 字段 | T-GOV-003 | Boot | **DONE** |
| T-BOOT-002 | AppContext { platform } | 无 Gate 字段（可并存过渡期见注） | T-BOOT-001 | Boot | **DONE** |
| T-BOOT-003 | 只读 accessors | platform/instrumentation/shutdown；无 get/resolve | T-BOOT-002 | Boot | **DONE** |
| T-BOOT-004 | BootstrappedApp + ShutdownController | into_parts；trigger take guard | T-BOOT-002 | Boot | **DONE** |
| T-BOOT-005 | BootstrapError 三态 + 映射 | Missing/Invalid/Unavailable | T-BOOT-001 | Boot | **DONE** |
| T-BOOT-006 | BootstrapBuilder 或 Bootstrap 扩展 | with_instrumentation；build→BootstrappedApp | T-BOOT-004 T-BOOT-005 | Boot | **DONE** |
| T-BOOT-010 | 单元测试 6 项（源 §8.5） | 只读/instrument/shutdown/无 mutation/缺依赖失败/无字符串 lookup | T-BOOT-006 | Boot | **DONE** |
| T-BOOT-011 | 静态审查无新 Service Locator | 无 TypeId/Any/HashMap registry | T-BOOT-006 | Boot | **DONE** |
| T-BOOT-012 | public API diff 审阅 | 文档+diff 入 Evidence | T-BOOT-006 | Boot | **DONE** |
| T-BOOT-020 | PlatformContext 接 EvidenceAppender | 新接口；禁旧 EvidenceSink | T-BOOT-001 | Boot | **DONE** until evidence 接入窗口 |
| T-BOOT-021 | 文档：未来 with_* 仅强类型 contract | README/AGENTS | T-BOOT-006 | Doc | **DONE** |

> 过渡期：PR-2 允许 AppContext **临时仍含** `gate` 字段（源：新增替代不删旧）；PR-4 删除。

---

## W2 / Phase 2 — 迁移消费者（PR-3）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-MIG-001 | 清 bootstrap 单测 gate 使用 | 无 DummyCap/register/ctx.gate/resolve | T-BOOT-010 | Mig | **DONE** |
| T-MIG-002 | 清 Capability 生产实现 | 仓内 impl Capability = 0（除 gate 自测待删） | T-MIG-001 | Mig | **DONE** |
| T-MIG-003 | register_capability 调用 = 0 | rg 存证 | T-MIG-001 | Mig | **DONE** |
| T-MIG-004 | AppContext::gate / .gate() = 0 | rg 存证 | T-MIG-001 | Mig | **DONE** |
| T-MIG-005 | e2e 改 typed contracts | MockBinance/MockKv 路径保留强化 | T-BOOT-010 | Mig | **DONE** |
| T-MIG-006 | 若有其他消费者则逐个迁 | inventory 增补 | T-INV-001 | Mig | TODO |
| T-MIG-007 | 禁止 downcast 迁移路径 | code review AC | — | Mig | **DONE** |
| T-MIG-010 | cargo test -p bootstrap 全绿 | log 入 Evidence | T-MIG-005 | Mig | **DONE** |

---

## W3 / Phase 3 — 移除旧 API（PR-4）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-RM-001 | bootstrap Cargo.toml 去 xhyper-gate | 非 dev-dep；cargo tree 证明 | T-MIG-010 | Rm | **DONE** |
| T-RM-002 | 删 Bootstrap/AppContext Gate 字段 | 源码无 gate:: | T-RM-001 | Rm | **DONE** |
| T-RM-003 | 删 register_capability / gate() | public API 无 | T-RM-002 | Rm | **DONE** |
| T-RM-004 | cargo tree -i xhyper-gate 无依赖者或 package 仅自身 | 存证 | T-RM-001 | Rm | **DONE** |
| T-RM-005 | 确认无 compat 或登记期限 | residual | T-RM-004 | Rm | **DONE** |
| T-COMPAT-001 | 仅当外部下游需要时建 gate-compat | 非 L0；deprecated；expires；禁新消费者 | T-INV-004 | Compat | **DEFER** unless downstream |

---

## W4 / Phase 4 — 删除 crate（PR-5 前半）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-DEL-001 | 删除 crates/gate/ | 路径不存在 | T-RM-004 T-GOV-003 | Del | TODO |
| T-DEL-002 | 根 Cargo.toml 去 member | metadata 无 | T-DEL-001 | Del | TODO |
| T-DEL-003 | cargo metadata 无 xhyper-gate/gate pkg | jq 证明 | T-DEL-002 | Del | TODO |
| T-DEL-004 | Cargo.lock 更新 | lock 无 workspace gate | T-DEL-002 | Del | TODO |
| T-DEL-005 | architecture registry 去 active gate | 非 archived 挂起 | T-DEL-001 | Del | TODO |
| T-DEL-006 | active specs 移出/Superseded | 路径处理+ADR 记 commit | T-DEL-001 | Del | TODO |
| T-DEL-007 | gen-structure / STRUCTURE 更新 | 无 runtime gate | T-DEL-002 | Del | TODO |
| T-DEL-008 | docs/architecture/spec.md L0 改写 | bootstrap 唯一组合根；禁 locator | T-DEL-005 | Doc | TODO |
| T-DOC-004 | CHANGELOG 根+bootstrap | Removed runtime gate… | T-DEL-001 | Doc | TODO |

---

## W5 / Phase 5 — 防回流（PR-5 后半）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-GUARD-001 | ARCH-COMPOSITION-001…005 | composition 规则（rg/CI/结构扫描；**非** archgate — OOS） | T-DEL-003 | Guard | TODO |
| T-GUARD-002 | source guard 模式 | 禁 use gate:: 等；**不**全局禁单词 gate | T-DEL-003 | Guard | TODO |
| T-GUARD-003 | fixture1 新增 gate workspace → fail | negative log | T-GUARD-001 | Guard | TODO |
| T-GUARD-004 | fixture2 Bootstrap HashMap String/Any → fail | | T-GUARD-001 | Guard | TODO |
| T-GUARD-005 | fixture3 AppContext register → fail | | T-GUARD-001 | Guard | TODO |
| T-GUARD-006 | fixture4 具体 adapter 类型外泄 → fail | | T-GUARD-001 | Guard | TODO |
| T-GUARD-007 | fixture5 字符串选依赖 → fail | | T-GUARD-001 | Guard | TODO |
| T-GUARD-008 | bootstrap public API 快照 | 无 Gate/Capability/register/resolve | T-RM-003 | Guard | **DONE** |
| T-GUARD-009 | architecture drift check | **N/A（OOS）**：不要求 archgate JSON；本仓用结构扫描/CI | T-GUARD-001 | Guard | **CANCELLED（OOS）** |
| T-GUARD-010 | CI policy gates 无误伤 | `.agent/gates` / CI·release policy gates 仍绿；archgate **N/A** | T-GUARD-002 | Guard | TODO |
| T-MET-001 | §18 十项指标验收 | 全 0/100%（archgate 项 **N/A**） | T-GUARD-010 | Verifier | TODO |
| T-VER-001 | §13.1 静态验证命令集执行 | fmt/clippy/check/test/lint-deps 等 logs；**无** `cargo run -p archgate` | T-DEL-003 | Verifier | TODO |
| T-VER-002 | §13.2 聚焦 bootstrap 测试 | cargo test -p bootstrap (+ e2e) log | T-MIG-010 | Verifier | **DONE** |
| T-VER-003 | §13.3 删除证明（xhyper-gate 名） | metadata/jq + rg 无 runtime 使用 | T-DEL-003 | Verifier | TODO |
| T-VER-004 | §13.4 依赖证明 | cargo tree -p bootstrap；-i xhyper-gate 应不存在 | T-DEL-003 | Verifier | TODO |
| T-VER-005 | §13.5 行为验证 | build/instrument/shutdown/无 mutation/contracts | T-BOOT-010 T-MIG-010 | Verifier | TODO |
| T-EVID-010 | Phase 0 Evidence 目录 | 源 §14 文件齐 | T-INV-002 T-FREEZE-002 | Inventory | TODO |
| T-EVID-011 | Phase 1 Evidence 目录 | 源 §14 文件齐 | T-BOOT-012 | Inventory | TODO |
| T-EVID-012 | Phase 2 Evidence 目录 | 源 §14 文件齐 | T-MIG-010 | Inventory | **DONE** |
| T-EVID-013 | Phase 3 Evidence 目录 | 源 §14 文件齐 | T-RM-004 | Inventory | **DONE** |
| T-EVID-014 | Phase 4 Evidence 目录 | 源 §14 文件齐 | T-DEL-003 | Inventory | TODO |
| T-EVID-015 | Phase 5 Evidence 目录 | 源 §14 文件齐 | T-GUARD-010 | Inventory | TODO |
| T-RB-001 | Phase 1–2 回滚：revert 当前 PR | PR 模板写明 | — | Lead | TODO |
| T-RB-002 | Phase 3 回滚：revert bootstrap removal | PR 模板写明 | — | Lead | TODO |
| T-RB-003 | Phase 4 回滚：revert deletion PR | PR 模板写明 | — | Lead | TODO |
| T-RB-004 | 禁止第三种中间架构回滚路径 | 文档 + residual；无临时 registry | — | Lead | TODO |
| T-RB-005 | 隐藏状态消费者则停止删除 | PR-5 检查清单 | T-INV-004 | Lead | TODO |
| T-DOC-001 | 命名消歧（runtime vs CI gate） | CLAUDE/AGENTS/spec | T-DEL-008 | Doc | TODO |
| T-DOC-002 | policy gate 概念保留说明 | docs | T-KEEP-005 | Doc | TODO |
| T-DOC-003 | architecture/spec 终态叙述 | 与 T-DEL-008 一致 | T-DEL-008 | Doc | TODO |

---

## W6 — 计划战役验收（本包）

| Task ID | 内容 | AC | 依赖 | Owner | Status |
|---------|------|-----|------|-------|--------|
| T-V10-001 | Round 01 findings | round-01-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-002 | Round 02 findings | round-02-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-003 | Round 03 findings（含 Task ID 可解析） | round-03-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-004 | Round 04 findings | round-04-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-005 | Round 05 findings | round-05-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-006 | Round 06 findings | round-06-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-007 | Round 07 findings | round-07-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-008 | Round 08 findings | round-08-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-009 | Round 09 findings | round-09-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-010 | Round 10 findings（含全量 ID 可解析扫描） | round-10-findings.md PASS | T-PLAN-* | Verifier | **DONE** |
| T-V10-011 | fail_rounds=0 裁决 | gate-plan-10x-verdict.md | T-V10-001…010 | Verifier | **DONE** |
| T-HONEST-001 | 非声称检查 | 文档无「crate 已删/DONE retirement」假声明 | T-ALIGN-001 | Verifier | **DONE** |
| T-IDSCAN-001 | Task ID 可解析扫描 | inventory/plan 引用的 T-* 均在 tasks.md 有独立行 | T-PLAN-004 | Verifier | **DONE** |

---

## Issue 映射（源 §6.3）

| Issue | Wave | Tasks |
|-------|------|-------|
| GATE-RETIRE-00 | W0 | T-FREEZE-* T-INV-* T-GOV-001 |
| GATE-RETIRE-01 | W1 | T-BOOT-* |
| GATE-RETIRE-02 | W2 | T-MIG-* |
| GATE-RETIRE-03 | W3 | T-RM-* T-COMPAT-* |
| GATE-RETIRE-04 | W4 | T-DEL-* |
| GATE-RETIRE-05 | W5 | T-GUARD-* T-DOC-* |
| GATE-RETIRE-06 | W5 | T-VER-001…005 T-MET-001 T-EVID-010…015 |
