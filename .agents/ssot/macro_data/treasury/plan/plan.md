<!-- ssot:trace=treasury.plan.001 -->
# treasury — 受限落地计划

当前 `implementation_status=not_started`，`planned_code_paths=[]`。

- [ ] 整理脱敏财政记录 fixture、缺失值、坏数值、重复身份和修订样本；
- [ ] 定义来源表、期间、单位、金额精度和缺失原因的纯函数校验；
- [ ] 记录未知权限、拒绝和输入冲突的稳定错误；
- [ ] 运行 workspace、离线测试、编码和 SSOT 门禁并记录证据。

不得创建 HTTP 客户端、认证、缓存、限流、重试、具体端点或真实 E2E。合同和授权获批后，另建 provider 任务并原子更新 manifest。
