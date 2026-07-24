<!-- ssot:trace=fred.goal.001 -->
# fred — 目标

| 目标 | 可验证结果 |
|---|---|
| G1 数据合同 | series、observation、vintage、单位和缺失语义明确 |
| G2 离线解析 | 脱敏 fixture、坏输入、重复身份和修订链可重放 |
| G3 安全 | 运行时 secret 只由专用引用承载，不进入 SSOT、日志、错误或序列化 |
| G4 准入 | 来源、许可、授权、Cargo member、测试和 commit 证据齐全 |

G4 完成前保持 `implementation_status=not_started`，不提供外部服务可用性承诺。
