# Alignment — TYPES-CANONICAL-002（2026-07-17）

> **SUPERSEDED for current-state authority（2026-07-21）**  
> 本文件保留为 2026-07-17 战役对齐记录。  
> **当前 1:1 权威矩阵**：[alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md)  
> **Active spec**：`spec/spec.md`（S1 Approved；OrderId 类型已删；`ts`=Unix ns）  
> 下文中「Candidate Draft / 不等于 Spec Approved」为 **历史快照**，不得覆盖 active/residual 已批准事实。

| 字段 | 值 |
|------|-----|
| Campaign | `GOAL-TYPES-CANONICAL-002` / `SPEC-TYPES-CANONICAL-002` |
| Plan | `PLAN-TYPES-CANONICAL-002-v1` |
| Branch | `docs/types-canonical-002-closure` |
| Baseline tip | `4fe8e98873f43dfa49f206752654fd9d246540a1` |
| Alignment tip | `26b4238befc70ffaae5c7828729c84e29551bc4f` |

## 权威关系

| 文档 | 角色 |
|------|------|
| [canonical-spec.md](../canonical-spec.md) | **active** 当前实现合同 SSOT |
| [20260717/xhyper-canonical-complete-spec.md](../20260717/xhyper-canonical-complete-spec.md) | **Approved S1** complete-spec（≠ package stable）；与 active 对齐 |
| [20260717/xhyper-canonical-complete-goal.md](../20260717/xhyper-canonical-complete-goal.md) | Goal（当前事实已重写；历史 M* 叙事） |
| [alignment-matrix-infra-2026-07-21.md](./alignment-matrix-infra-2026-07-21.md) | **当前 1:1 权威矩阵** |
| [plan/](./) | 本战役执行包 |
| [todo.md](../todo.md) | 工作台账（≠ package stable） |

## 本轮对齐动作（计划）

1. Candidate 路径：`.agent/draft/*` → `20260717/*`  
2. Goal/Spec 相对链接修正  
3. crate README/CHANGELOG 与公开 API 对齐  
4. 测试覆盖对齐 Spec §7 agent-safe 子集  
5. residual OPEN 显式，不粉饰  

## 不等于（仍成立）

- package stable / crates.io  
- Goal ACHIEVED / 全 wire Production Ready  
- 10x PASS 单独成实现完成（无 fresh 证据时 SAFE-15=DEFERRED）  
- ~~Spec S1 Approved~~ ← **已完成**（2026-07-17）；勿再写 Draft  

## 生产晋级（2026-07-17）

| 文档 | 作用 |
|------|------|
| [production-upgrade.md](./production-upgrade.md) | 生产门槛 + Phase A/B/C |
| [approval-packet-prod-m1.md](./approval-packet-prod-m1.md) | TIME/ID/WIRE/VALID **人审提案**（未签字） |
| [wire-commitment-matrix.md](./wire-commitment-matrix.md) | wire 等级 |
| [validation-owners.md](./validation-owners.md) | 校验 owner 表 v1 |

## Evidence 目录

`evidence/types-canonical-002/`（门禁日志、10x、approval readback）
