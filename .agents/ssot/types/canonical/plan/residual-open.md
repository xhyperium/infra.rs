# Residual Open — SPEC-TYPES-CANONICAL-002

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TYPES-CANONICAL-002-v1` + `PLAN-TYPES-CANONICAL-PROD-001` |
| Baseline | `main@4fe8e988` · prod branch `feat/canonical-prod-upgrade` |
| 更新 | 2026-07-21（infra 对齐 + R6 双写） |

> OPEN 不是失败；**假装 CLOSED** 才是失败。  
> 生产晋级提案见 [production-upgrade.md](./production-upgrade.md) / [approval-packet-prod-m1.md](./approval-packet-prod-m1.md)。

---

## OPEN 语义（须人审或独立 RFC）

| ID | 摘要 | Sev | 默认 | 生产路径 |
|----|------|-----|------|----------|
| OPEN-ID-001 | Venue slug 规则 + adapter shape | P1 | **CLOSED** 规范 | `shape::is_plausible_*` |
| OPEN-ID-002 | OrderRef newtype 二期；OrderId **类型已删** | P2 | newtype Defer | 字段仍为 String wire |
| OPEN-TIME-001 | `ts: i64` = Unix **ns**（Approved 2026-07-17） | P0 | **CLOSED** | adapter 写 DTO 用 ns_from_unix_millis |
| OPEN-WIRE-001 | 未知字段策略（deny/ignore） | P1 | 默认 serde 忽略 | 矩阵已登记 OPEN |
| OPEN-WIRE-002 | 未覆盖 DTO 的跨版本 wire 承诺与 golden 目录 | P1 | 仅 RT / 实现细节 | cancel/ack golden；其余 Uncommitted |
| OPEN-WIRE-003 | 枚举新增兼容策略（non_exhaustive 等） | P2 | 未冻结 | Phase B |
| OPEN-VALID-001 | validation owner 表 v1 | P1 | **CLOSED** 原则 | 表仍可增补 consumer |
| OPEN-LAYOUT-001 | 新建 types/core / types/protocol 大搬迁 | GOV | 须 Approved RFC | 不在本路径 |
| OPEN-SERDE-001 | 是否移除 serde | GOV | 须消费与数据迁移证据 | 不在本路径 |
| OPEN-MIG-001 | legacy Order / OrderAck 字段迁结构化 ID | P1 | additive first | Phase C |

---

## REJECTED（持续禁止，非 OPEN）

| ID | 项 |
|----|-----|
| REJ-CODEC-001 | Canonical Encoding Core / schema registry / 通用 envelope |
| REJ-HASH-001 | 本 crate 内 hash/sign/evidence 链 |
| REJ-EMPTY-001 | `canonical → ∅`（切断 decimalx） |
| REJ-BIZ-001 | 订单状态机 / 盘口 diff / 风控校验进本 crate |

---

## DEFERRED（本战役明确不交付）

| ID | 项 | 备注 |
|----|-----|------|
| DEFER-M3-REST | 非 binance/okx 的全量 consumer / domain trait 迁移 | 主路径 DONE；其余见 m3 checklist |
| DEFER-WIRE-FULL | 未覆盖 DTO 的跨版本 golden / unknown-field 冻结 | OPEN-WIRE-001/002 |
| DEFER-STABLE | package quality stable / crates.io | HUMAN S2 |
| DEFER-NEWTYPE | OrderRef/Venue newtype 二期 | OPEN-ID-002 |

---

## CLOSED（计划/agent-safe 文档与测试面）

| ID | 说明 |
|----|------|
| PLAN-DOC-v1 | plan/gap/inventory/tasks/residual/approval 首版 |
| PROD-DOC-v1 | production-upgrade + wire 矩阵 + validation owners + M1 人审包 |
| PROD-TEST-v1 | 矩阵/owner 覆盖测 + legacy ack fixture + ts 形状测 |
| INFRA-ALIGN-20260721 | alignment 矩阵 + active/pipeline 对齐 + workspace 成员 + R6 双写 |
| （实现项随 PR 更新） | 见 todo.md 证据列 |

## 生产路径状态（诚实）

| 项 | 状态 |
|----|------|
| T1–T4 人审 | **DONE**（liukongqiang5 2026-07-17） |
| Phase B + contracts additive + adapter ns/shape | **DONE** |
| Phase C：OrderId 类型删除 / symbol:id 清除 / native ack | **DONE** |
| Spec Approved | **DONE**（S1） |
| package stable | **DEFER**（S2） |
| 全 DTO wire 承诺 | **OPEN**（矩阵 Uncommitted） |
