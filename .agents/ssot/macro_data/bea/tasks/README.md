<!-- ssot:trace=bea.task.001 -->
# bea — 离线任务

当前 `not_started`，不创建 provider crate，不执行联网或凭据测试。

| 任务 | 范围 | 退出条件 |
|---|---|---|
| BEA-T-001 | 待来源合同确认格式后的脱敏 fixture 解析 | 合法、缺失、未知和坏输入可重放 |
| BEA-T-002 | 来源身份、单位、频率和期间校验 | 重复身份与冲突返回稳定错误 |
| BEA-T-003 | source-to-kernel 映射占位 | 仅在来源合同和字段字典获批后建立映射；当前外部事实为 `UNKNOWN` |
| BEA-T-004 | secret sentinel 安全回归 | 所有输出表面均不可见 |
| BEA-T-005 | workspace、离线测试和 SSOT 门禁 | 命令、退出码、fixture 摘要写入证据 |

未来只有在合同、授权和路径获批后，才可另建请求、认证、限流或联网任务；当前这些语义均为 `UNKNOWN`。
