# schedulex maintenance 管线

> 实现：`crates/schedulex`
> Active Spec：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-schedulex-complete-spec.md](spec/xhyper-schedulex-complete-spec.md)
> 战役：`SPEC-SCHEDULEX-003` · Beads `infra-2d9.9.1`

## 三轮状态

| 轮次 | 目标 | 状态 |
|---|---|---|
| Round 1 | 源码、规格、测试、历史与依赖事实审计 | PASS |
| Round 2 | 冻结 registry/tick interface、失败语义与 NO-GO | PASS |
| Round 3 | public seam TDD、门禁、独立复审与发布收口 | IN PROGRESS |

## 管线入口

| 层 | 路径 |
|---|---|
| Goal / Design / Plan | [goal](goal/goal.md) · [design](design/design.md) · [plan](plan/plan.md) |
| Spec / Matrix | [spec](spec/spec.md) · [matrix](matrix/matrix.md) |
| Tasks / Prompt | [tasks](tasks/tasks.md) · [prompt](prompt/prompt.md) |
| Test / Gate / Evidence | [test](test/test.md) · [gate](gate/gate.md) · [evidence](evidence/README.md) |
| Review / Release / Retro | [review](review/review.md) · [release](release/release.md) · [retrospective](retrospective/retrospective.md) |

## 硬边界

- `Scheduler` 仅登记 ID；`JobRunner` 仅由宿主显式 `tick(now_ms)` 驱动。
- std-only；禁止后台线程、真实墙钟、async runtime、持久化和分布式 lease。
- 测试/覆盖率不能外推为 package stable 或生产调度平台 readiness。
- 双镜像必须字节一致；实现代码不得复制到 SSOT 树。
