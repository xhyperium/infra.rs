# types/canonical — Goal 管线契约

> 实现：`crates/types/canonical`  
> Spec：[spec/spec.md](spec/spec.md) ≡ dual-mirror · **S1 Approved**（≠ package stable）  
> Alignment：[plan/alignment-matrix-infra-2026-07-21.md](plan/alignment-matrix-infra-2026-07-21.md)  
> 状态：agent-safe DTO 表面 **PASS** · package stable **未宣称**

## 11 层

| 层 | 路径 | 状态 |
|----|------|------|
| Goal | [goal/goal.md](goal/goal.md) | agent-safe **PASS**；stable/全 wire OPEN |
| Spec | [spec/spec.md](spec/spec.md) | S1 Approved |
| Design | [design/design.md](design/design.md) | 要点有效 |
| Plan | [plan/plan.md](plan/plan.md) | 战役 + alignment |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | agent-safe DONE |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 |
| Code | `crates/types/canonical` | 实现 |
| Test | [test/test.md](test/test.md) | 22 tests PASS |
| Review | [review/review.md](review/review.md) | **PASS with residual** |
| Release | [release/release.md](release/release.md) | stable **BLOCKED** |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 部分填写 |

硬限制：`OrderId` 类型已删；`ts`=Unix ns；禁止假 Done；dual-mirror 须 cmp。
