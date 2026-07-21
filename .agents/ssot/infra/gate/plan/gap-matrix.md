# Gap Matrix — PLAN-GATE-RETIRE-001

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-GATE-RETIRE-001-v1-complete` |
| Baseline | `main@41c59584` |
| 更新 | 2026-07-15 |

图例：

| 状态 | 含义 |
|------|------|
| ABSENT | 目标不存在 |
| WRONG | 存在但语义错误 / 与目标冲突 |
| PARTIAL | 部分存在 |
| PRESENT | 已符合目标 |
| N/A | 本战役不交付（已 DEFER 或政策） |
| PLAN-OK | 计划包已覆盖；实现未做 |

---

## 1. 运行时 API / 行为

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-API-001 | Capability 业务方法 | 仅 name() | 删除 Capability | WRONG | T-DEL-001 |
| G-API-002 | 字符串 resolve | HashMap\<String,_\> | 删除 | WRONG | T-RM-* T-DEL-* |
| G-API-003 | register 原子性 | 非事务 | 删除路径 | WRONG | 随删除 |
| G-API-004 | 默认 evidence | None | 删除；新 evidence 走 typed | WRONG | T-BOOT-020 DEFER |
| G-API-005 | build 后 mutate | 可 register | 只读 AppContext | WRONG | T-BOOT-003/011 T-RM-* |
| G-API-006 | 错误分类 | 全 Invalid | BootstrapError 三态 | WRONG | T-BOOT-005 |
| G-API-007 | mock feature | 存在 | 删除 | WRONG | T-DEL-001 |

---

## 2. 组合根 / 分层

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-LAY-001 | gate layer | 文档 L0 + path infra | 删除；无 L0 gate | WRONG | T-DEL-008 |
| G-LAY-002 | 双组合中心 | bootstrap + gate | bootstrap only | WRONG | T-BOOT-* T-RM-* |
| G-LAY-003 | PlatformContext | ABSENT | PRESENT typed | ABSENT | T-BOOT-001 |
| G-LAY-004 | BootstrappedApp | ABSENT | PRESENT | ABSENT | T-BOOT-004 |
| G-LAY-005 | BootstrapBuilder 语义 | Bootstrap 含 gate | 无 gate；typed with_* | WRONG | T-BOOT-006 T-RM-* |
| G-LAY-006 | TypeId locator 风险 | 未引入 | 永不引入 | PRESENT（尚未错） | T-GUARD-004 防回流 |

---

## 3. 消费者 / 依赖

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-DEP-001 | cargo tree 依赖者 | bootstrap only | 0 dependents + 无 package | PARTIAL 面小 | T-RM-001 T-DEL-* |
| G-DEP-002 | workspace member | 有 crates/infra/gate | 无 | WRONG | T-DEL-002 |
| G-DEP-003 | 生产 service 经 gate | 0 发现 | 0 | PRESENT（好） | 维持 T-FREEZE |
| G-DEP-004 | 测试 DummyCap/E2ECap | 有 | 0 | WRONG | T-MIG-* |
| G-DEP-005 | external downstream | 仓内未发现 | 确认 | PLAN-OK inventory | T-INV-004 实现前复核 |

---

## 4. 治理 / 文档

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-GOV-001 | Source Status | Proposed | Accepted 后实现 | PARTIAL | A1–A3 人审 |
| G-GOV-002 | RFC | ABSENT | Approved | ABSENT | T-GOV-001/003 |
| G-GOV-003 | ADR sole root | 部分 ADR-012 相关叙述 | 正式 ADR | PARTIAL | T-GOV-002/003 |
| G-GOV-004 | architecture/spec L0 gate | 仍列出 | 删除并加强 bootstrap | WRONG | T-DEL-008 |
| G-GOV-005 | CLAUDE/AGENTS gate 路径 | 仍描述存在 | 退役中/已删诚实 | PARTIAL | T-ALIGN-001 + T-DOC-001 |
| G-GOV-006 | gate-spec active | active | Superseded/移出 | WRONG | T-DEL-006 |
| G-GOV-007 | 命名消歧 | 部分散落 | 显式 SSOT | PARTIAL | T-DOC-001 |

---

## 5. 门禁 / 防回流

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-GRD-001 | no-new-gate | 仅计划登记 | CI/xtask 启用 | ABSENT 落地 | T-FREEZE-002 |
| G-GRD-002 | ARCH-COMPOSITION-001…005 | ABSENT | PRESENT | ABSENT | T-GUARD-001 |
| G-GRD-003 | source guard | ABSENT | PRESENT | ABSENT | T-GUARD-002 |
| G-GRD-004 | negative fixtures | ABSENT | 5 fixtures | ABSENT | T-GUARD-003…007 |
| G-GRD-005 | API snapshot | ABSENT | 无 Gate 符号 | ABSENT | T-GUARD-008 |
| G-GRD-006 | CI policy 误伤防护 | N/A | 保留 .agent/gates | PLAN-OK 保留策略 | T-KEEP-* T-GUARD-010 |

---

## 6. Evidence / 验证

| ID | 主题 | 现状 | 目标 | Gap | Close path |
|----|------|------|------|-----|------------|
| G-EVD-001 | Phase Evidence 树 | 仅 plan-package 快照 | 每 Phase 全文件 | PARTIAL | T-EVID-010…P5 |
| G-EVD-002 | §13 验证执行 | 未做实现验证 | 全命令绿 | ABSENT | T-VER-* |
| G-EVD-003 | §18 指标 | 未测 | 全达标 | ABSENT | T-MET-001 |
| G-EVD-004 | §19 Done | 全未勾 | 全勾 | ABSENT | 实现战役终态 |

---

## 7. 计划包完备性（本战役目标）

| ID | 主题 | 现状 | Gap |
|----|------|------|-----|
| G-PLAN-001 | plan.md | PRESENT | PLAN-OK |
| G-PLAN-002 | source-inventory I-1…28 | PRESENT | PLAN-OK |
| G-PLAN-003 | tasks 覆盖 Exit+§19 | PRESENT | PLAN-OK |
| G-PLAN-004 | residual OPEN/DEFER/FORBID | PRESENT | PLAN-OK |
| G-PLAN-005 | approval 人审 OPEN 诚实 | PRESENT | PLAN-OK |
| G-PLAN-006 | consumer live 存证 | PRESENT | PLAN-OK |
| G-PLAN-007 | gate-todo | PRESENT | PLAN-OK |
| G-PLAN-008 | 10x fail_rounds=0 | PRESENT | PLAN-OK |
| G-PLAN-009 | alignment 诚实非声称 | PRESENT | PLAN-OK |
| G-PLAN-010 | Forbidden 完整 | PRESENT | PLAN-OK |

---

## 8. 优先级关闭序（实现）

```text
1. 人审 A2/A3/A4/A5/A6（治理边界）
2. T-FREEZE-002 no-new-gate 落地
3. PR-2 typed bootstrap
4. PR-3 migrate tests
5. PR-4 remove API
6. 人审 A12 + PR-5 delete + guards
7. §19 + §18 终态证明
```
