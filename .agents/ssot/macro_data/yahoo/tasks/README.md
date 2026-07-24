<!-- ssot:trace=yahoo.task.001 -->
# yahoo — 任务分解

| ID | 任务 | 状态 | 依赖 |
|---|---|---|---|
| YAH-001 | 核验来源、许可、权限和数据语义 | 待办 | — |
| YAH-002 | 建立 quote/chart/fx/search 脱敏 fixture | 待办 | YAH-001 |
| YAH-003 | 实现离线解析、schema 检查和统一映射设计 | 待办 | YAH-002 |
| YAH-004 | 编写缺失值、未知字段、拒绝和日志脱敏测试 | 待办 | YAH-003 |
| YAH-005 | 审批 provider 根路径后再实现 | 待办 | YAH-001–YAH-004 |
