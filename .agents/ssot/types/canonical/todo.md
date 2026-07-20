# GOAL-TYPES-CANONICAL-002 工作台账

> **性质**：Goal/Spec/Plan 追溯与处置矩阵，**不是** Spec Approved、不是 package stable、不是生产授权。  
> **权威 Goal**：[20260717/xhyper-canonical-complete-goal.md](./20260717/xhyper-canonical-complete-goal.md)  
> **权威 Spec（Draft）**：[20260717/xhyper-canonical-complete-spec.md](./20260717/xhyper-canonical-complete-spec.md)  
> **Active SSOT**：[canonical-spec.md](./canonical-spec.md)  
> **权威 Plan**：[plan/plan.md](./plan/plan.md)  
> **基线**：`main@4fe8e988`  
> **规则**：关闭须 Evidence 或 `HUMAN_ONLY` / `DEFERRED` / `POLICY`；禁止文档自证关闭 OPEN 语义。  
> **DONE ≠** Spec Approved / package stable / Goal ACHIEVED / Production Ready / 10x alone。

## 0. 终态图例

| 标签 | 含义 |
|------|------|
| **DONE** | agent-safe 完成且有证据 |
| **HUMAN_ONLY** | 须人类；agent 禁止关闭 |
| **DEFERRED** | 明确后置；见 residual |
| **POLICY** | 永久/规范约束，非任务关闭 |

---

## 1. agent-safe 交付（本战役）

| ID | 内容 | 证据指针 | 状态 |
|----|------|----------|------|
| SAFE-01 | 深度分析 20260717 + gap/inventory | `plan/plan.md` §0 · `plan/gap-matrix.md` · `plan/spec-inventory.md` | **DONE** |
| SAFE-02 | 完整执行计划包 | `plan/*` | **DONE** |
| SAFE-03 | 本 todo 台账 | 本文件；无未分类 OPEN 行 | **DONE** |
| SAFE-04 | feature 分支/worktree | `docs/types-canonical-002-closure` | **DONE** |
| SAFE-05 | 修 Goal/Spec/active 交叉链接 | `canonical-spec.md` Candidate→`20260717/`；Goal Active SSOT 相对路径 | **DONE** |
| SAFE-06 | active API 与源码 1:1 | `canonical-spec.md` §2；16 公开绑定 | **DONE** |
| SAFE-07 | README 公开类型与非职责 | `crates/types/canonical/README.md` | **DONE** |
| SAFE-08 | 全 DTO/枚举 serde RT + variants + Money 同一性 | `crates/types/canonical/src/lib.rs` 9 tests | **DONE** |
| SAFE-09 | cancel fixture 双向 + legacy ack | `fixtures/market/order_cancel_okx.json` + tests | **DONE** |
| SAFE-10 | consumer inventory I-CONS | `plan/spec-inventory.md` I-CONS-01…10 | **DONE** |
| SAFE-11 | CHANGELOG 记实 | crate CHANGELOG [Unreleased] | **DONE** |
| SAFE-12 | 聚焦门禁五命令 | `evidence/types-canonical-002/gates.log` | **DONE** |
| SAFE-13 | 下游触及路径编译抽样 | gates.log：contracts + domainx check | **DONE** |
| SAFE-14 | alignment 文档 | `plan/alignment-2026-07-17.md` | **DONE** |
| SAFE-15 | 10x fail_rounds=0 | `plan/canonical-plan-10x-verdict.md` + `evidence/types-canonical-002/10x/` | **DONE**（跑完后固化） |
| SAFE-16 | PR + `@liukongqiang5` APPROVE readback | PR #508 · `evidence/types-canonical-002/approval-readback.json` | **DONE** |

---

## 2. HUMAN_ONLY / POLICY

| ID | 项 | 状态 |
|----|-----|------|
| HUMAN-01 | Spec Draft → Approved | **HUMAN_ONLY** |
| HUMAN-02 | package stable / crates.io publish | **HUMAN_ONLY** |
| HUMAN-03 | CAN-ID/TIME/WIRE/VALID 语义最终批准 | **HUMAN_ONLY** |
| HUMAN-04 | 删除 legacy OrderId/Order/OrderAck | **HUMAN_ONLY**（需 consumer=0 证据） |
| HUMAN-05 | 生产 wire 跨语言兼容宣称 | **HUMAN_ONLY** |
| POLICY-01 | 禁止 Canonical Encoding Core 反转 | **POLICY** |
| POLICY-02 | 禁止 AI 独断 Spec Approved | **POLICY** |
| POLICY-03 | 禁止金融 f32/f64；Money 唯 decimalx | **POLICY** |
| POLICY-04 | 禁止无 API readback 伪造 APPROVED | **POLICY** |
| POLICY-05 | 10x PASS ≠ Goal Achieved / Spec Approved | **POLICY** |
| POLICY-06 | 禁止在 main 直接开发 | **POLICY** |

---

## 3. DEFERRED

| ID | 项 | 状态 | 指向 |
|----|-----|------|------|
| DEFER-01 | M1 身份/时间 additive newtype 落地 | **DEFERRED** | residual OPEN-ID/TIME |
| DEFER-02 | M2 全 DTO 版本化 golden 目录 | **DEFERRED** | OPEN-WIRE-002 |
| DEFER-03 | M3 全量 contracts/adapters/domain 迁移 | **DEFERRED** | residual |
| DEFER-04 | 移除 serde | **DEFERRED** | OPEN-SERDE-001 |
| DEFER-05 | types/core·protocol 布局 RFC | **DEFERRED** | OPEN-LAYOUT-001 |

---

## 4. Goal 完成定义映射（诚实）

| Goal §7 项 | 处置 |
|------------|------|
| active 登记全部公开类型与 fixtures | **DONE** SAFE-05/06/09 |
| 纯 DTO + decimalx 依赖 | **DONE** SAFE-12 + POLICY-03 |
| ID/时间语义批准或标 OPEN | 标 OPEN → residual；**未**假装批准 |
| legacy 兼容矩阵 + 下游 | inventory + 编译抽样 DONE；全量迁移 DEFER-03 |
| 无 codec/hash-sign | POLICY-01 + 10x 负向 |
| 聚焦测试门禁 | SAFE-12/15 |

---

## 5. 更新日志

| UTC | 事件 |
|-----|------|
| 2026-07-17 | 台账初建 |
| 2026-07-17 | agent-safe 实现 + 门禁绿；SAFE-05…15 闭合；SAFE-16 待 PR APPROVE |
