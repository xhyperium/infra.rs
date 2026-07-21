# Source Inventory — PLAN-GATE-RETIRE-001

| 字段 | 值 |
|------|-----|
| Source | `xhyper-gate-retirement-complete-plan.md` |
| Plan package | `PLAN-GATE-RETIRE-001-v1-complete` |
| 目的 | 防遗漏枚举：每条源内容映射到 Task / residual / DEFER |
| 更新 | 2026-07-15 |

> 规则：每条 I-* 必须有 **Mapped**（Task ID 或 residual/DEFER ID）。禁止「via design」无 ID。

---

## I-1 元数据头

| 字段 | 源值 | Mapped |
|------|------|--------|
| Plan ID | PLAN-GATE-RETIRE-001 | plan.md 表头 |
| Target | gate @ crates/gate | T-DEL-001 |
| Decision | Retire and delete | residual DEF-GOV / approval A1 |
| Replacement | bootstrap + typed AppContext | T-BOOT-* |
| Status | Proposed | residual GOV-RFC/ADR OPEN |
| Method | Strangler Fig + small-batch | plan §1 / PR-1…5 |

---

## I-2 §0 最终裁定 — 删除列表

| Item | Mapped |
|------|--------|
| crates/gate | T-DEL-001 |
| package: gate / xhyper-gate | T-DEL-002 T-DEL-003 |
| runtime Gate / Capability | T-RM-002 T-DEL-001 |
| register / resolve | T-RM-003 T-MIG-003 |

---

## I-3 §0 最终裁定 — 保留列表

| Item | Mapped |
|------|--------|
| .agent/gates/ | T-KEEP-001 |
| tools/archgate/ | T-KEEP-002 |
| CI gate jobs | T-KEEP-003 |
| release gates | T-KEEP-004 |
| xlibgate / policy gate 概念 | T-KEEP-005 T-DOC-002 |
| 命名消歧叙述 | T-DOC-001 |

---

## I-4 §1 退役理由 D-1…D-8

| ID | 主题 | Mapped |
|----|------|--------|
| D-1 | Capability 仅 name | gap-matrix G-API-001 |
| D-2 | 字符串 resolve | G-API-002 |
| D-3 | 非事务 register | G-API-003 |
| D-4 | 默认无 evidence | G-API-004 |
| D-5 | build 后可 mutate | G-API-005 → T-BOOT-011 |
| D-6 | 错误语义 | G-API-006 → T-BOOT-005 |
| D-7 | L0 vs infra 路径 | G-LAY-001 → T-DEL-005 |
| D-8 | 双组合中心 | G-LAY-002 → T-BOOT-* / ADR |

---

## I-5 §2 目标架构

| Item | Mapped |
|------|--------|
| 分层终态无 runtime registry | T-BOOT-* T-GUARD-* |
| contracts / adapters / bootstrap / AppContext 分工 | T-GOV-002 ADR |
| 禁止 TypeId Service Locator | I-28 / T-GUARD-004 |

---

## I-6 §3.1 PlatformContext

| Field/API | Mapped |
|-----------|--------|
| instrumentation: Arc\<dyn Instrumentation\> | T-BOOT-001 |
| shutdown_signal: ShutdownSignal | T-BOOT-001 |
| evidence later（新 EvidenceAppender，非旧 Sink） | T-BOOT-020 DEFER(accepted) until evidence 稳定接入 |
| instrumentation() / shutdown_signal() | T-BOOT-003 |

---

## I-7 §3.2 AppContext

| Item | Mapped |
|------|--------|
| platform: PlatformContext | T-BOOT-002 |
| platform() accessor | T-BOOT-003 |
| 窄 accessor instrumentation/shutdown | T-BOOT-003 |
| 禁止 get/resolve/register/insert/Any/HashMap | T-BOOT-011 T-GUARD-* |

---

## I-8 §3.3 BootstrappedApp + ShutdownController

| Item | Mapped |
|------|--------|
| BootstrappedApp { context, shutdown } | T-BOOT-004 |
| ShutdownController { guard: Option\<ShutdownGuard\> } | T-BOOT-004 |
| context() / into_parts() | T-BOOT-004 |
| trigger() take+trigger，不可复制 | T-BOOT-004 T-BOOT-010 |

---

## I-9 §3.4 BootstrapBuilder

| Item | Mapped |
|------|--------|
| with_instrumentation / build | T-BOOT-006 |
| 默认可 TracingInstrumentation + system shutdown | T-BOOT-006 |
| 未来 with_evidence/config/market_data/storage 强类型 only | T-BOOT-021 文档约束 |
| 禁止通用 register API | T-BOOT-011 |

---

## I-10 §3.5 BootstrapError

| Variant | Mapped |
|---------|--------|
| MissingDependency | T-BOOT-005 |
| InvalidConfiguration | T-BOOT-005 |
| DependencyUnavailable | T-BOOT-005 |
| → XError Missing/Invalid/Unavailable | T-BOOT-005 |
| 若 kernel 新 API 未落地用 XResult 并登记 | residual DEFER-ERR-MAP |

---

## I-11 §4 Bounded Context

| Item | Mapped |
|------|--------|
| MarketDataContext 设计 | residual **DEFER-BOUND-CTX**（accepted；本战役不实现） |
| ExecutionContext 设计 | residual **DEFER-BOUND-CTX**（accepted；本战役不实现） |
| 原则：最小上下文 / 无动态注册 / build 后不可替换 | plan §0.3 + T-GUARD-001…010 |

---

## I-12 §5.1 删除对象（逐项）

见 plan §4.1；Mapped → T-DEL-* T-RM-* T-MIG-*。

含：Cargo.toml/src/README/AGENTS/CHANGELOG of gate、workspace member、bootstrap dep、全部 Gate/Capability API、mock feature。

---

## I-13 §5.2 不删除 + CI rename

| Item | Mapped |
|------|--------|
| 保留列表 | T-KEEP-* |
| CI job gate → policy-gates 建议 | residual DEFER-CI-RENAME（非阻塞） |

---

## I-14 §6 治理

| Item | Mapped |
|------|--------|
| RFC: Retire Runtime Gate Service Locator | T-GOV-001 |
| ADR: Bootstrap Is the Sole Composition Root | T-GOV-002 |
| Issue GATE-RETIRE-00…06 | tasks.md Wave 表 |

RFC 必答 8 问（源 §6.1）→ T-GOV-001 AC 清单。

---

## I-15 §7 Phase 0 冻结与盘点

| Item | Mapped |
|------|--------|
| 禁止新增 gate dep / use gate / Gate:: / Capability / register_capability / 字符串 resolve | T-FREEZE-001（登记规则） |
| no-new-gate guard **已启用**（源 §7.5 Exit） | **T-FREEZE-002**（CI/xtask 落地） |
| rg 扫描命令 | T-FREEZE-001 / T-FREEZE-002 / T-INV-003 |
| cargo metadata | T-INV-002 |
| cargo tree -i **xhyper-gate**（修正：非 `gate`） | T-INV-002 |
| 盘点面：prod/dev/tests/examples/benches/docs/specs/arch/CI/codegen/lock/external | T-INV-001 |
| 已知面：bootstrap only | consumer-inventory.md |
| external downstream 搜索 | T-INV-004 |
| Exit Gate 6 项 | plan §3.1（启用 → T-FREEZE-002） |

---

## I-16 §8 Phase 1 typed 替代

| Item | Mapped |
|------|--------|
| 新增不删旧 | T-BOOT-* PR-2 |
| deprecated 兼容仅当确有下游 | T-COMPAT-001 条件触发 |
| 禁止新通用容器 | T-BOOT-011 |
| 测试清单 6 项 | T-BOOT-010 |
| Exit Gate 6 项 | plan §3.2 |

---

## I-17 §9 Phase 2 迁移

| Item | Mapped |
|------|--------|
| 删 DummyCap / register / ctx.gate / len / resolve 测试 | T-MIG-001…004 |
| e2e 强化 MockBinance/MockKv contracts | T-MIG-005 |
| 服务迁移模式：typed field 或构造注入 | T-MIG-006 |
| 禁止 downcast 迁移 | T-MIG-007 / Forbidden |
| Exit Gate 7 项 | plan §3.3 |

---

## I-18 §10 Phase 3 移除旧 API

| Item | Mapped |
|------|--------|
| 删 use gate / gate field / register / AppContext.gate | T-RM-001…003 |
| Cargo.toml 删 gate path dep（禁 dev-dep 回落） | T-RM-001 |
| compat crate 条件 | T-COMPAT-001 |
| Exit Gate 5 项 | plan §3.4 |

---

## I-19 §11 Phase 4 删除 crate

| Item | Mapped |
|------|--------|
| 删目录 / workspace / metadata / lock | T-DEL-001…004 |
| architecture registry | T-DEL-005 |
| active specs 移出 / Superseded | T-DEL-006 |
| architecture/spec.md L0 改写 | T-DEL-008 / T-DOC-003 |
| gen-structure | T-DEL-007 |
| CHANGELOG | T-DOC-004 |
| Exit Gate 7 项 | plan §3.5 |

---

## I-20 §12 Phase 5 防回流

| Guard | Mapped |
|-------|--------|
| ARCH-COMPOSITION-001…005 | T-GUARD-001 |
| source forbidden patterns（勿全局禁单词 gate） | T-GUARD-002 |
| negative fixtures 1–5 | T-GUARD-003…007 |
| public API snapshot | T-GUARD-008 |
| Exit Gate 6 项 | plan §3.6 |

---

## I-21 §13 验证矩阵

| 块 | Mapped |
|----|--------|
| 静态命令集 | T-VER-001 |
| 聚焦 bootstrap | T-VER-002 |
| 删除证明（含 xhyper-gate 名） | T-VER-003 |
| 依赖证明 | T-VER-004 |
| 行为验证 | T-VER-005 |

---

## I-22 §14 Evidence 目录布局

| Item | Mapped |
|------|--------|
| plan-package 快照布局 | T-EVID-000 |
| Phase 0 Evidence 目录 | T-EVID-010 |
| Phase 1 Evidence 目录 | T-EVID-011 |
| Phase 2 Evidence 目录 | T-EVID-012 |
| Phase 3 Evidence 目录 | T-EVID-013 |
| Phase 4 Evidence 目录 | T-EVID-014 |
| Phase 5 Evidence 目录 | T-EVID-015 |

---

## I-23 §15 回滚

| 阶段 | Mapped |
|------|--------|
| Phase 1–2 revert PR | T-RB-001 |
| Phase 3 revert bootstrap removal | T-RB-002 |
| Phase 4 revert deletion | T-RB-003 |
| 禁止第三种中间架构 | T-RB-004 文档 + residual |
| 无数据回滚；隐藏状态则停删 | T-RB-005 |

---

## I-24 §16 PR-1…PR-5

| PR | Mapped Tasks |
|----|--------------|
| PR-1 | T-GOV-* T-FREEZE-* T-INV-* |
| PR-2 | T-BOOT-* |
| PR-3 | T-MIG-* |
| PR-4 | T-RM-* T-COMPAT-* |
| PR-5 | T-DEL-* T-GUARD-* T-EVID-* T-DOC-* |

PR 纪律（worktree/非 main/green/rollback/Evidence/不混重构）→ T-PROC-001。

---

## I-25 §17 1/7/30 天

Mapped → plan §10 + gate-todo 时间盒；**非**单独 Task 阻塞。

---

## I-26 §18 衡量指标（10 项）

全部 Mapped → T-MET-001（终态度量脚本/检查清单）在 PR-5。

---

## I-27 §19 Done 定义 19.1–19.5

| 节 | checkbox 数 | Mapped |
|----|-------------|--------|
| 19.1 | 4 | plan §3.7 + T-INV/MIG/RM/COMPAT |
| 19.2 | 6 | T-BOOT/GUARD |
| 19.3 | 6 | T-DEL-* |
| 19.4 | 9 | T-GOV/DOC/GUARD/EVID |
| 19.5 | 4 | T-KEEP/DOC/RB |

**全部 checkbox 在实现战役关闭前保持未勾**；计划包只映射不虚假勾选。

---

## I-28 §20 Forbidden（8 项）+ 正确终态

| Forbidden | residual / guard |
|-----------|------------------|
| 移到 crates/gate | FORBID-001 |
| 字符串→TypeId | FORBID-002 |
| Capability+Any/downcast | FORBID-003 |
| sealed 后保留 registry | FORBID-004 |
| 合并进 kernel | FORBID-005 |
| 原样复制进 bootstrap | FORBID-006 |
| 为动态插件预建 | FORBID-007 |
| Big Bang 删后修 | FORBID-008 |

正确路径 PR-1…5 与终态 5 句 → plan §11。

---

## 覆盖证明

| 源章节 | Inventory |
|--------|-----------|
| 头 + §0 | I-1 I-2 I-3 |
| §1 | I-4 |
| §2 | I-5 |
| §3 | I-6…I-10 |
| §4 | I-11 |
| §5 | I-12 I-13 |
| §6 | I-14 |
| §7–12 Phases | I-15…I-20 |
| §13–18 | I-21…I-26 |
| §19 | I-27 |
| §20 | I-28 |

**遗漏数：0**（计划包口径）。
