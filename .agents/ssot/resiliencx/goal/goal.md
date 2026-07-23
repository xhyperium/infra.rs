# resiliencx — Goal

> 状态：`infra-2d9.9` 第 3 轮候选准备中；目标尚未闭合。

## 目标

把 `resiliencx 0.1.2` 收敛为可固定复验的单进程弹性候选：生产重试入口要求显式 safety，整次
deadline 覆盖 operation 与退避，seed 进入真实 jitter 路径，预算 reservation 在取消时 refund，
bulkhead poison 不泄漏容量，且观测只记录真正准备执行的 retry。

| 完成条件 | 当前状态 |
|---|---|
| safety / deadline / seeded jitter / budget / poison 实现 | 已完成；第 2 轮独立代码/规格复审通过 |
| 行覆盖率 100% | root 串行复验 `1208 / 1208`、zeros 0、退出码 0 |
| active / complete spec 同构 | 本轮 writer `cmp` 退出码 0 |
| 本地独立 reviewer | 已完成实现/证据审查；纯状态 delta 不改变受审源码/测试 |
| 独立 verifier | 已完成技术/证据初验 |
| 固定 commit CI、PR、维护者审批与合并 | Pending |

完成条件不包含分布式预算/熔断/限流/舱壁、撤销外部副作用、排队舱壁或 package stable。
