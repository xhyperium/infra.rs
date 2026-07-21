# Residual — PLAN-GOALCTL-002-phase1.1-v1

> 本文件只登记 **非 DONE 终态** 或战役边界外项。Agent-safe 完成后应清空「OPEN」；剩余仅 HUMAN_ONLY / DEFERRED / POLICY。

| ID | 项 | Disposition | 原因 | Owner 建议 |
|----|-----|-------------|------|------------|
| R-001 | Schema↔Rust 自动一致性（GAP-009） | DEFERRED | Phase 1.2；非 MVA 阻断 | Platform |
| R-002 | 完整 JCS/跨语言 golden（GAP-010） | DEFERRED | AC-P1-DETERMINISM 跨 OS | Platform |
| R-003 | RepositoryIdentity FULL（GAP-011） | DEFERRED | 需可信 numeric id / CI input | Platform + Sec |
| R-004 | module filter 边界（GAP-012） | DEFERRED | 非 P0 假阳性源 | Platform |
| R-005 | 统一 PathSpec（GAP-013） | DEFERRED | Phase 1.2 | Platform |
| R-006 | Bootstrap Trust Root（GAP-015） | HUMAN_ONLY | 签名/双人审 | Security |
| R-007 | Harness/Evidence/Verifier/Shadow（GAP-016） | HUMAN_ONLY | Phase 2–4 + 独立 CR | Platform |
| R-008 | Failure Corpus / SLO / replay（GAP-017） | HUMAN_ONLY | Phase 3+ | Platform |
| R-009 | required CI Cutover | POLICY | D10；独立 Approved CR | Governance |
| R-010 | GOAL-GOALCTL-002 ACHIEVED | HUMAN_ONLY | §6.3 30 天指标与 Cutover | Owners |
| R-011 | 默认 compile 真 Goal→Task 完整编译（GAP-004 全量） | DEFERRED | 本战役仅禁虚构 PASS + 标注 template | Platform |
| R-012 | Evidence subject/freshness 全校验（GAP-006 全量） | DEFERRED | 依赖 Evidence 生产面 | Platform |

## 本战役完成后应满足

- 上表无 AGENT_SAFE OPEN
- todo.md 无 bare OPEN agent-safe
- 不把 residual 项标 DONE
