<!-- ssot:trace=bea.plan.001 -->
# bea — 受限落地计划

当前 `implementation_status=not_started`，`planned_code_paths=[]`。本计划只允许离线 fixture、纯函数解析、错误脱敏和领域映射讨论。

- [ ] 整理脱敏观测 fixture、缺字段、坏数值、重复身份和修订批次；
- [ ] 定义来源身份、期间、单位、频率和缺失原因的纯函数校验；
- [ ] 运行 workspace、离线测试、编码和 SSOT 门禁并记录证据；
- [ ] 来源、访问方式、认证、字段、配额和许可当前为 `UNKNOWN`；核验完成后另建 provider 任务并更新 manifest。

不得创建网络客户端、凭据类型、请求模型、重试、限流、缓存、具体端点或真实 E2E。
