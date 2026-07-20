# Approval Packet — SPEC-TYPES-CANONICAL-002

> **SUPERSEDED for current-state authority（2026-07-21）**  
> 本文件是 **2026-07-17 agent-safe 战役** 的原始人审包（Draft 台账）。  
> **生产 M1 / S1 权威**： [approval-packet-prod-m1.md](./approval-packet-prod-m1.md)（T1–T4 / S1 **Approved**；S2 stable **Defer**）。  
> **当前 residual / 语义**： [residual-open.md](./residual-open.md) · [alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md) · `spec/spec.md`。  
> 下文 A4「CAN-ID/TIME 保持 OPEN」、A5「保留 OrderId」、A6「Spec Approved Defer」为 **历史快照**，已被 prod-m1 否决/闭合，**不得**当作现行权威。

| 字段 | 值 |
|------|-----|
| Packet ID | `APPR-TYPES-CANONICAL-002-v1` |
| Spec | `SPEC-TYPES-CANONICAL-002`（**历史 Draft 包**；现行 S1 见 approval-packet-prod-m1） |
| Plan | `PLAN-TYPES-CANONICAL-002-v1` |
| 日期 | 2026-07-17 |
| 基线 | `main@4fe8e988` |
| 实现分支 | `docs/types-canonical-002-closure` |
| 请求复核人 | `@liukongqiang5` |

---

## 1. 本战役请求裁定事项

| # | 事项 | 建议 | 本战役 AI 权限 |
|---|------|------|----------------|
| A1 | 接受 plan 包与 todo 作为 Draft 执行台账 | Approve 台账 | AI 可落盘；**不**升 Spec Approved |
| A2 | agent-safe：active 链接/API 对齐、测试加固、文档记实 | Approve 实施 | AI 可做 |
| A3 | 保持 CAN-BND/NUM/LAYER；CAN-CODEC REJECTED | Confirm | AI 持续执行 |
| A4 | CAN-ID/TIME/WIRE/VALID 保持 OPEN | Confirm OPEN | AI **不得**写 Approved |
| A5 | 保留 legacy OrderId 直至 consumer=0 | Confirm | AI 禁止删除 |
| A6 | Spec Draft → Approved | **Defer** | **HUMAN_ONLY** |
| A7 | package stable / publish | **Defer** | **HUMAN_ONLY** |
| A8 | tip-bound PR APPROVE（`@liukongqiang5`） | 对 PR tip 做 durable APPROVE | token + API；失败 → HUMAN_ACTION_REQUIRED |

---

## 2. 明确 **不** 请求在本 PR 批准的事项

- 时间戳单位、Venue/Instrument 字符集最终规范  
- 全量 DTO 跨版本/跨语言 wire 稳定  
- 删除 serde 或新建 types/core·protocol  
- Goal ACHIEVED / Production Ready  

---

## 3. AI 权限边界

| AI 可做 | AI 不可做 |
|---------|-----------|
| 修死链、补 inventory、补测、门禁、10x 日志 | 将 Spec 标 Approved |
| 记实 CHANGELOG/README/alignment | 宣称 stable / 伪造 wire 承诺 |
| 调用 approve helper（token 已 export） | 无 API readback 手写 APPROVED |
| residual 诚实登记 | 关闭 HUMAN_ONLY 阻塞 |

---

## 4. 人审签字区

| 角色 | handle | 日期 | 裁决 | 证据 |
|------|--------|------|------|------|
| Agent-safe 战役 tip APPROVE | `liukongqiang5` | 2026-07-17 | **历史**（PR #507 等） | `evidence/types-canonical-002/*` — **非**当前 tip 绑定 |
| 生产路径 tip APPROVE | `liukongqiang5` | 2026-07-17 | **PR #510** tip-bound | 本地/SCRATCH `evidence/types-canonical-prod/approval-readback.json`（避免 dismiss 环） |
| Spec Approved（S1） | `liukongqiang5` | 2026-07-17 | **Approve** | [approval-packet-prod-m1.md](./approval-packet-prod-m1.md) |
| package stable（S2） | — | — | **Defer** | — |

> 本文件是 agent-safe 战役包指针；**当前 tip 权威**以 GitHub PR #510 最新 `liukongqiang5` APPROVE + prod-m1 包为准。

---

## 5. 附件

- [plan.md](./plan.md)
- [gap-matrix.md](./gap-matrix.md)
- [tasks.md](./tasks.md)
- [spec-inventory.md](./spec-inventory.md)
- [residual-open.md](./residual-open.md)
- [../todo.md](../todo.md)
- Goal / Spec：`../20260717/`
