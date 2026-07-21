> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# Round 8 Pass2

> Verifier: 计划完备性重检（非实现验收）  
> 对照: pass1 `round-08-findings.md` F8-1…2 · v1.1 I-SPEC-PATH / I-RFC-DEL / T-DOC-*  
> 日期: 2026-07-14

## 原 FAIL 关闭状态（逐条 CLOSED/OPEN + 证据文件引用）

### F8-1 — §19.6 终态权威路径未强制为 `spec/spec.md`

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-SPEC-PATH** = `.agents/ssot/testkit/spec/spec.md` |
| 证据 | **T-DOC-005**「终态 path I-SPEC-PATH」AC=`active=1` |
| 证据 | **T-ARCH-007** AC 强制 `I-SPEC-PATH`（不再「complete-spec 或 …」） |
| 说明 | 路径字符串已钉死；迁移/archive 由 T-DOC-002/003/005 + ARCH-007 链负责 |

### F8-2 — §18.2 宏/placeholder 删除须 Approved RFC 未任务化

| 状态 | **CLOSED** |
|------|------------|
| 证据 | **I-RFC-DEL**：Approved RFC + 消费者清单 + 同批迁移 + CHANGELOG + API diff + compatibility |
| 证据 | **T-DOC-RFC-DEL** AC「六步」=`I-RFC-DEL` |
| 证据 | **T-24-006** 依赖含 `T-DOC-RFC-DEL` |
| 说明 | 未新增 A11 条目，但 Task 六步已覆盖 §18.2 删除流程；不依赖 approval A11 才能闭合 |

## 新发现 FAIL（若有）

无（F8-3/4 次要观察项保持建议级）。

## 本轮结论 PASS

## fail_count: 0
