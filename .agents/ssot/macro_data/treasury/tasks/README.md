<!-- ssot:trace=treasury.task.001 -->
# treasury — 离线任务

当前 `not_started`，不创建网络客户端、认证、缓存、限流、重试或 provider crate。

| 任务 | 范围 | 退出条件 |
|---|---|---|
| TRY-T01 | 财政记录 fixture 解析 | 合法、缺失、未知和坏输入可重放 |
| TRY-T02 | 来源表、期间、单位和精度校验 | 重复身份与冲突返回稳定错误 |
| TRY-T03 | 修订、缺失和错误脱敏 | 不丢失身份且无敏感输出 |
| TRY-T04 | workspace、离线测试和 SSOT 门禁 | 命令、退出码和 fixture 摘要入证据 |
