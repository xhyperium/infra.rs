# Residual Open — PLAN-GATE-RETIRE-001

| 字段 | 值 |
|------|-----|
| SSOT | 本文件 + `.worktrees/gate-todo.md` |
| Baseline | main（#355 退役 + #357 收尾 + #360–#369 CI 闭环） |
| 更新 | 2026-07-15 v1.5 quality/loom required |

## 战役状态

```text
runtime gate crate 退役：DONE（物理删除 + 防回流）
RFC / ADR-016：Accepted
bootstrap typed composition：active
CI / policy / release gates：保留
archgate / .architecture：OOS（infra.rs 不引入）
```

## CLOSED defects（实现/治理）

| ID | 摘要 | Status |
|----|------|--------|
| DEF-API-001…003 | Service Locator / register 语义 | **CLOSED** crate 已删 |
| DEF-LAY-001/002 | 分层与双组合中心 | **CLOSED** |
| DEF-GOV-001…003 | Plan/RFC/ADR | **CLOSED** Accepted + Implemented |
| DEF-PHYS-001/002 | crate / workspace member | **CLOSED** 已删除 |
| DEF-DEP-001 | bootstrap→gate | **CLOSED** |
| DEF-DOC-001 | architecture L0 gate | **CLOSED** |
| DEF-FREEZE-001 | no-new-gate CI | **CLOSED** |
| PLAN-GAP-001…011 | 计划包完备性缺口 | **CLOSED** |
| PLAN-10X-001 | fail_rounds | **CLOSED** =0 |

## DEFER（仍有效 · accepted）

| ID | 项 | Status |
|----|-----|--------|
| DEFER-BOUND-CTX | MarketDataContext / ExecutionContext 实现 | **CLOSED** 最小 typed 结构（无 Risk port） |
| DEFER-EVID-FIELD | PlatformContext.evidence: EvidenceAppender | **CLOSED** optional `with_evidence` |
| DEFER-COMPAT | gate-compat crate | DEFER（无外部下游） |
| DEFER-CI-RENAME | CI job `gate` → `policy-gates` | **CLOSED** job id `policy-gates` |
| DEFER-ERR-MAP | BootstrapError → kernel 新错误 API | **CLOSED** Missing/Invalid/Unavailable 映射 |

## Forbidden（政策 · 永久）

FORBID-001…008 仍有效（见 plan §11 / residual 历史）。

## 诚实边界

```text
runtime gate retired on main
≠ package quality stable for unrelated crates
DEFER-BOUND-CTX 最小 typed 已合入 ≠ RiskDecisionPort / MarketDataStore 已进 contracts
DEFER-COMPAT 仍开放 ≠ 退役战役未完成
ci.yml 深度 job（miri/quality/loom）已 required ≠ DEFER-COMPAT 已关闭
```
