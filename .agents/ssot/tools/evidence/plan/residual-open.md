# Residual Open — SPEC-EVIDENCE-002

| 字段 | 值 |
|------|-----|
| SSOT | 本文件（仓内）+ `.worktrees/evidence-todo.md`（本地镜像） |
| Baseline | `main@007ca7b5` |
| 更新 | 2026-07-14 v1.1 |

## OPEN defects（DEF）

| ID | 摘要 | Sev | Wave | Status |
|----|------|-----|------|--------|
| DEF-001 | runtime 在 tools/ | P0 | W1/W6 | OPEN |
| DEF-002 | anyhow 依赖 | P0 | W1 | OPEN |
| DEF-003 | 通用 hash_bytes | P0 | W1/W6 | OPEN |
| DEF-004 | 全零 genesis | P0 | W1 | OPEN |
| DEF-005 | 公开可变 record 字段 | P0 | W1 | OPEN |
| DEF-006 | domain_macro Debug hash | P0 | W3 | OPEN |
| DEF-007 | Mock verify 恒成功 | P0 | W2/W6 | OPEN |
| DEF-008 | lock poison 静默 | P1 | W2 | OPEN |
| DEF-009 | chain corrupt → Invalid | P1 | W1 | OPEN |
| DEF-010 | 「不可篡改」措辞 | P1 | W0 | OPEN |
| DEF-011 | 无 chain_id/sequence/event_id | P0 | W1 | OPEN |
| DEF-012 | 无 durable adapter | P1 | W4 | OPEN |
| DEF-013 | 无 checkpoint/anchor | P1 | W5 | OPEN |
| DEF-014 | 无 golden vectors | P0 | W1 | OPEN |
| DEF-015 | 无 evidence-policy.toml | P1 | W0/W3 | OPEN |
| DEF-016 | ADR-010/旧 spec 冲突 | GOV | W8 | OPEN |
| DEF-017 | fail-closed 未落地 | P0 | W3 | OPEN |
| DEF-018 | LE+拼接歧义 | P0 | W1 | OPEN |
| DEF-019 | ADR-012 auditx 路径冲突 | GOV | W0/W8 | OPEN |
| DEF-020 | 双包同名 `evidence` 风险 | P0 | W1 | OPEN |

## OPEN plan gaps（首轮 10x 后已登记修补）

| ID | 摘要 | Status |
|----|------|--------|
| PLAN-GAP-001 | 幽灵 T-ATOM | **CLOSED** v1.1 → T-ATOM-001…006 |
| PLAN-GAP-002 | EVIDENCE-* 不全 | **CLOSED** v1.1 → T-ARCH-010…019 |
| PLAN-GAP-003 | preimage 25 步未枚举 | **CLOSED** → I-1 |
| PLAN-GAP-004 | Error 24 variant 未枚举 | **CLOSED** → I-4 |
| PLAN-GAP-005 | metrics 11 名未枚举 | **CLOSED** → I-11 |
| PLAN-GAP-006 | §22 隐私无 Task | **CLOSED** → T-PRIV-* |
| PLAN-GAP-007 | CI 草案桶 | **CLOSED** → T-MUT/MIRI/FUZZ/NIGHTLY |
| PLAN-GAP-008 | residual-open 缺失 | **CLOSED** 本文件 |
| PLAN-GAP-009 | inventory 缺失 | **CLOSED** spec-inventory.md |
| PLAN-ALIGN-001 | 架构/导航文档未标注迁移中 | **CLOSED** 2026-07-14 alignment |
| PLAN-10X-001 | 计划完备性 fail_rounds≠0 | **CLOSED** pass3 fail_rounds=0 |

## DEFER candidates（须 accepted 才不算 OPEN 漏洞）

| ID | 项 | 默认 |
|----|-----|------|
| DEFER-ATOM-004 | 订单/交易所 Attempted+terminal 全落地 | 可 DEFER(accepted) 本战役 |
| DEFER-ANCHOR-IMPL | 真实 KMS/OSS WORM 联调 | 合同接口必须有；实现可后置 |
| DEFER-STABLE | registry stable | 人审后 |

## CLOSED（计划包）

| ID | 说明 |
|----|------|
| PLAN-DOC-v1 | plan/gap/tasks/approval 首版落盘 |
| PLAN-DOC-v1.1 | inventory + residual + 任务补全 |
