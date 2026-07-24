<!-- ssot:trace=fred.test.001 -->
# fred — 测试策略

当前只运行离线 fixture，不读取真实认证材料，不访问外部服务。

| 类别 | 必测场景 |
|---|---|
| 解析 | series、observation、vintage、缺失值、未知字段、坏数值 |
| 身份 | source/series/date/vintage 重复、修订追加和排序 |
| 安全 | sentinel secret 在 Debug、Display、Serialize/JSON、错误、tracing、URL 和原始响应中均不可见 |
| 合同失败 | 拒绝、挑战、配额和权限未知返回稳定错误 |

只有书面授权、manifest 状态批准和人工审查完成后，才可单独提案联网测试；未授权环境不得用跳过测试制造绿色证据。
