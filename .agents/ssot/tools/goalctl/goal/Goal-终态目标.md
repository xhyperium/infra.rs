# AI-Native Goal：终态目标

```text
Status:     PROPOSED 目标叙述（非 Active / 非 Achieved）
Foundation: DECISION-PACK-001 + CR-20260716 已 Approved（2026-07-16）
Read first: ../README.md · ../decisions/DECISION-PACK-001.md
```

## 北极星目标

构建生产级 AI-Native 软件交付控制系统（Goal Delivery OS），让 AI 在严格约束下执行工程工作，所有结果均可验证、可审计、可追溯。

## 终态目标

- 单一事实源：`docs/goal` + `.agents/ssot`
- 禁止第二控制面（`.config/goal`）
- Goal→Spec→Design→Plan→Tasks→Matrix→Gate→Evidence 全链路闭环
- 实现 goalctl：Doctor、Index、Authority、Artifact、Reconciliation、Task Compiler、Harness、Verifier、Gate Adapter
- Worktree 隔离、单 Writer、Scope Guard、Capability Policy、Protected Asset Policy
- Review Bundle + Audit Chain 双轨 Evidence
- Shadow→Mirror→Cutover 渐进迁移
- Bootstrap Trust、Approval、Repository Identity、Break-glass、安全沙箱
- Failure→Eval→Self-improving 持续改进

## 最终系统能力

系统能够自动且可验证地回答：

1. 当前权威是什么；
2. 当前真实状态是什么；
3. 哪些声明冲突；
4. 哪些 Evidence 有效；
5. Task 是否可执行；
6. 允许修改哪些范围；
7. 如何证明完成；
8. 是否允许发布。

## 当前门闩

- [DECISION-PACK-001](../decisions/DECISION-PACK-001.md) 与 foundation CR：**已批准**。
- PR-0A 形状（policy/schemas/contracts）：**已落盘**。
- 实现 CR：[CR-20260716-goalctl-impl-phase1](../../../../../docs/goal/change-requests/CR-20260716-goalctl-impl-phase1.md) — **Proposed**（批准后 PR-1）。
- 本 Goal 叙述本身仍为 PROPOSED，**非** Active/Achieved。
