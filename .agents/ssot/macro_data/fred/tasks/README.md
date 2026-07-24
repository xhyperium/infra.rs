<!-- ssot:trace=fred.task.001 -->
# fred — 任务分解

| ID | 任务 | 状态 | 依赖 |
|---|---|---|---|
| FRED-001 | 核验来源、许可、权限和修订语义 | 待办 | — |
| FRED-002 | 建立 series/observation/vintage 脱敏 fixture | 待办 | FRED-001 |
| FRED-003 | 设计离线 parser、错误和 `domain_macro` 映射 | 待办 | FRED-002 |
| FRED-004 | 完成 sentinel secret、坏输入和重复身份测试 | 待办 | FRED-003 |
| FRED-005 | 审批 provider 根路径与访问模式 | 待办 | FRED-001–FRED-004 |

当前任务不产生网络调用、不保存认证材料；未批准路径不是 evidence。
