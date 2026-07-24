<!-- ssot:trace=jin10.plan.001 -->
# jin10 — 受限落地计划

当前 `implementation_status=not_started`，`planned_code_paths=[]`。计划只允许离线 parser、脱敏 fixture 和错误语义；不存在获批的 provider crate 路径。

## 当前阶段

- [ ] 整理合法、缺字段、未知字段、坏数值、重复身份和乱序时间 fixture
- [ ] 定义来源事件 ID、发布时间、业务期间、语言和缺失原因的纯函数映射
- [ ] 定义拒绝、未知权限和输入冲突的稳定错误语义
- [ ] 运行 workspace、离线测试、编码和 SSOT 门禁并记录退出码

## 明确禁止

- 不创建网络客户端、WebSocket 客户端、认证注入、缓存、自动重试或限流实现
- 不添加真实服务依赖、真实凭据、外部 URL 或周期性联网任务
- 不以 ignored、跳过或本地 Mock 结果证明供应商合同和生产可用性

## 晋级条件

只有书面授权、官方合同、获批代码路径、脱敏 fixture、真实测试、合规审查、回滚目标和 commit-matched evidence 齐全后，才能另建 provider 任务并原子更新 manifest。
