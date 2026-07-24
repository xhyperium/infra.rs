<!-- ssot:trace=uk_cb.plan.001 -->
# uk_cb — 受限落地计划

当前 `implementation_status=not_started`，`planned_code_paths=[]`；英国央行来源、Series ID、协议、授权和许可均为 `UNKNOWN`。

- [ ] 整理脱敏 JSON/CSV fixture、缺字段、坏数值、重复身份和修订批次；
- [ ] 定义系列身份、期间、单位、精度和缺失原因的纯函数校验；
- [ ] 运行 workspace、离线测试、编码和 SSOT 门禁并记录证据。

不得创建 Cargo provider、HTTP 客户端、User-Agent、重试、限流、缓存、具体端点或真实 E2E。合同和授权获批后，另建实现任务并更新 manifest。
