<!-- ssot:trace=fred.plan.001 -->
# fred — 分阶段计划

| 阶段 | 交付物 | 退出条件 |
|---|---|---|
| P0 | 核验官方来源、许可、访问权限和修订语义 | 证据含来源、日期、授权依据和人工审查 |
| P1 | series、observation、vintage、缺失值的脱敏 fixture | fixture SHA-256 固定，坏输入和重复身份可重放 |
| P2 | 离线 parser、错误模型和 `domain_macro` 映射 | 认证材料不进入 SSOT/Debug/Serialize/日志/错误 |
| P3 | 获批 provider 根路径提案 | manifest、Cargo member、测试与 evidence 同一 commit |

当前不创建具体 provider 路径，不实现网络、认证、缓存或未核验配额。权限未知时拒绝真实服务调用。
