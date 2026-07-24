# Approval Packet — SPEC-EVIDENCE-002

| 字段 | 值 |
|------|-----|
| Packet ID | `APPR-EVIDENCE-002-v1` |
| Spec | `SPEC-EVIDENCE-002` |
| Plan | `PLAN-EVIDENCE-002-v1-complete` |
| 日期 | 2026-07-14 |
| 基线 | `main@007ca7b5`（开战役时）· 实现分支 `feat/evidence-002-core-v1` / PR #253 |
| 人审 | **2026-07-14** · handle `ZoneCNH` · 会话明确「授权审批」 |

---

## 1. 请求裁定事项

请 Owner / 架构 / 安全 对下列事项给出 **Approve / Reject / Defer**：

| # | 事项 | 影响 | 建议 | **裁决（2026-07-14 ZoneCNH）** |
|---|------|------|------|-------------------------------|
| A1 | 将 `xhyper-evidence-complete-spec.md` 升为 **Approved**，supersede `evidence-spec.md` | 实现 SSOT | Approve after plan 10x | **Approve** |
| A2 | 修订 **ADR-010** evidence 部分：从六字段+mock feature → V1 模型；mock 迁出 core | 与 Article IX 历史约定冲突，需 ADR | Approve with ADR | **Approve 方向**；ADR 正文修订可随本 PR 或紧随 follow-up |
| A3 | runtime 路径 `tools/evidence` → `crates/infra/evidence`；适配器 `crates/adapters/evidence/*` | 改 R1 路径描述 | Approve | **Approve**（cutover 已落地） |
| A4 | core 依赖白名单：`kernel + sha2 + thiserror`；禁止 anyhow/serde | 破坏性相对现状 | Approve | **Approve** |
| A5 | 取消 core `mock` feature；测试替身仅 memory adapter / testkit | 宪法 Article IX 需解释 | Approve via A2 | **Approve** |
| A6 | 生产默认 Durability::Durable；memory 禁止生产 | 安全 | Approve | **Approve** |
| A7 | 旧链迁移策略：新 genesis + migration record，**不**静默 rehash | 历史审计语义 | Approve | **Approve** |
| A8 | 独立锚点最低要求（WORM OSS / 独立库） | 运维成本 | Defer 实现细节；Approve 合同 | **Approve 合同**；生产 WORM 实现 **Defer** |
| A9 | 版本 0.1.1 与 quality **stable** 时机 | 发布 | Defer until §33 | **Defer**（本 PR 不宣称 package stable / 0.1.1 发布） |
| A10 | golden vectors 目录最终路径 | 工具链 | 默认 `crates/infra/evidence/tests/vectors/evidence-v1/`（I-6） | **Approve** 默认路径 |
| A11 | **ADR-012 `auditx` vs 002 `crates/infra/evidence`** 路径唯一权威 | 迁移目标冲突 | **必须人审**；计划默认 002 | **Approve 002 路径为权威**；auditx 叙述后续修订 |
| A12 | 迁移期 package rename：`evidence_legacy` | 双包同名 | Approve 策略 | **Approve**；cutover 后 legacy 已删除 |
| A13 | T-ATOM-004 外部 Attempted+terminal 是否本战役交付或 DEFER | 范围 | 可 DEFER(accepted) 但须登记 | **DEFER(accepted)** · residual |

---

## 2. 不可豁免项（即使 Exception 流程）≡ I-26

```text
1. 假 §33 Done / registry stable / 无 Evidence 勾 PASS
2. 旧链静默 rehash 声称 V1 连续
3. core 引入 anyhow/serde/tokio/uuid/chrono
4. mock / evidence_memory 进生产
5. 通用 hash_bytes / Debug→digest
6. 链损坏映射 Invalid
7. SKIP 计 PASS / 手写 digest
8. 私钥进 core/仓库
9. AI 独断 Spec Approved
10. 两个 package 同名 evidence 并存无隔离
```

**关于 I-26.9**：本包 **不是** AI 独断。人审主体 `ZoneCNH`（repo admin / PR author）于 2026-07-14 在 agent 会话中明确指令「授权审批」；agent 仅代为落盘裁决与改状态。

---

## 3. AI 权限边界（本回合）

| AI 可做 | AI 不可做 |
|---------|-----------|
| 按人审裁决改 Spec Status → Approved | 将 registry quality 标 **stable**（A9 Defer） |
| 更新 plan/todo/alignment 诚实状态 | 宣称 §33.1–33.6 **全闭合**（仍有 residual） |
| CI 修绿 / PR 合入（人审授权后） | 宣称生产 WORM IndependentAnchor 已交付（A8 Defer 实现） |
| 登记 DEFER(accepted) | 静默 rehash / 双包同名 |

---

## 4. 人审签字区

| 角色 | 姓名/handle | 日期 | 裁决（A1–A13） | 签名 |
|------|-------------|------|----------------|------|
| Spec Owner | ZoneCNH | 2026-07-14 | A1–A8/A10–A12 Approve；A9 Defer；A13 DEFER(accepted)；A2 方向 Approve | 会话指令「授权审批」 |
| Architecture | ZoneCNH | 2026-07-14 | 同左（A3/A11 含路径权威） | 同上 |
| Security | ZoneCNH | 2026-07-14 | A4–A8 Approve 合同；WORM 实现 Defer | 同上 |
| Release | ZoneCNH | 2026-07-14 | A9 Defer（不在本 PR 做 stable/0.1.1 发布） | 同上 |

---

## 5. 实现与验收诚实摘要（人审时点）

| 项 | 状态 |
|----|------|
| plan 10x | pass3 fail_rounds=0（计划完备性，≠ 实现完成） |
| Core V1 `crates/infra/evidence` | 已实现 |
| adapters memory/file/postgres/signer | 已实现 |
| `tools/evidence-cli` | 已实现 |
| cutover 删除 `tools/evidence` | 已实现 |
| domain_macro / gate 迁 002 | 已实现 |
| Spec Approved | **本包人审后生效** |
| §33 全闭合 | **否** · residual 见 residual-open / evidence-todo |
| package quality stable | **否**（A9） |
| 生产 WORM anchor | **否**（A8 实现 Defer；DirectoryAnchor/MemoryAnchor 仅开发/测试） |

---

## 6. 附件

- [plan.md](./plan.md)
- [gap-matrix.md](./gap-matrix.md)
- [tasks.md](./tasks.md)
- [spec-inventory.md](./spec-inventory.md)
- [residual-open.md](./residual-open.md)
- [evidence-plan-10x-verdict.md](./evidence-plan-10x-verdict.md)
- [xhyper-evidence-complete-spec.md](../xhyper-evidence-complete-spec.md)
- [alignment-2026-07-14.md](../../../../../evidence/evidence-002/alignment-2026-07-14.md)
- [`.worktrees/evidence-todo.md`](../../../../.worktrees/evidence-todo.md)
- PR: https://github.com/xhyperium/infra.rs/pull/253
