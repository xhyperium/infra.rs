<!-- ssot:trace=yahoo.goal.001 -->
# yahoo — 目标

当前目标是形成可审计的离线数据合同，而不是承诺外部服务可用性。

| 目标 | 可验证结果 |
|---|---|
| G1 数据模型 | quote、chart、fx、search 的字段、单位、时间和缺失语义明确 |
| G2 稳定解析 | 脱敏 fixture 与坏输入测试可重放 |
| G3 安全边界 | 凭据、原始响应和访问方式不进入 SSOT 或日志 |
| G4 生产准入 | 官方/合同证据、人工批准、Cargo member 和 commit 绑定齐全 |

在 G4 前，`implementation_status` 保持 `not_started`。
