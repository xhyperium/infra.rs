<!-- ssot:trace=japan_cb.task.001 -->
# japan_cb — 离线任务

当前 `not_started`，不创建 provider crate，不执行联网、缓存或凭据测试。

| 任务 | 范围 | 退出条件 |
|---|---|---|
| JCB-T01 | 脱敏 SDMX/CSV fixture 解析 | 合法、缺失、未知和坏输入可重放 |
| JCB-T02 | 系列身份、期间、单位和修订校验 | 重复身份与乱序返回稳定错误 |
| JCB-T03 | 缺失原因与来源字段映射 | 不丢失来源身份 |
| JCB-T04 | workspace、离线测试和 SSOT 门禁 | 命令、退出码和 fixture 摘要入证据 |

合同和授权获批后，另建网络实现任务并原子更新 manifest。
