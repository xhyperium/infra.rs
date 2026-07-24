<!-- ssot:trace=domain_macro.task.001 -->
# domain_macro 任务

| ID | 任务 | 状态 | 依赖 |
|---|---|---|---|
| DM-T01 | 批准值对象/时间/精度 ADR | 未开始 | Goal 审批 |
| DM-T02 | 实现受控构造和反序列化 | 未开始 | DM-T01 |
| DM-T03 | 实现 Identity/Period/Unit | 未开始 | DM-T02 |
| DM-T04 | 实现修订链、快照和 diff | 未开始 | DM-T03 |
| DM-T05 | 实现 wire schema、N-1 和回滚 fixture | 未开始 | DM-T04 |
| DM-T06 | 接入第一个已核验 provider | 未开始 | DM-T05 + provider 审批 |
