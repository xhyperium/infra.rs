# Alignment — TYPES-CANONICAL-002（2026-07-17）

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
| [20260717/xhyper-canonical-complete-spec.md](../20260717/xhyper-canonical-complete-spec.md) | Candidate Draft，**不覆盖** active |
| [20260717/xhyper-canonical-complete-goal.md](../20260717/xhyper-canonical-complete-goal.md) | Goal Draft |
| [plan/](./) | 本战役执行包 |
| [todo.md](../todo.md) | 工作台账（≠ Spec Approved） |

## 本轮对齐动作（计划）

1. Candidate 路径：`.agent/draft/*` → `20260717/*`  
2. Goal/Spec 相对链接修正  
3. crate README/CHANGELOG 与公开 API 对齐  
4. 测试覆盖对齐 Spec §7 agent-safe 子集  
5. residual OPEN 显式，不粉饰  

## 不等于

- Spec Approved  
- package stable  
- Goal ACHIEVED / Production Ready  
- 10x PASS 单独成实现完成  

## 生产晋级（2026-07-17）

| 文档 | 作用 |
|------|------|
| [production-upgrade.md](./production-upgrade.md) | 生产门槛 + Phase A/B/C |
| [approval-packet-prod-m1.md](./approval-packet-prod-m1.md) | TIME/ID/WIRE/VALID **人审提案**（未签字） |
| [wire-commitment-matrix.md](./wire-commitment-matrix.md) | wire 等级 |
| [validation-owners.md](./validation-owners.md) | 校验 owner 表 v1 |

## Evidence 目录

`evidence/types-canonical-002/`（门禁日志、10x、approval readback）
