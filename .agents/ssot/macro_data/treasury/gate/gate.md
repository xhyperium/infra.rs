<!-- ssot:trace=treasury.gate.001 -->
# treasury — 离线门禁

- 只执行 workspace fmt、clippy、test、编码、SSOT 和离线 fixture 门禁；
- 拒绝、未知权限、坏数值、重复身份和缺失语义必须有稳定错误与测试 ID；
- 不执行真实服务、不读取凭据、不猜测请求间隔、端点、缓存 TTL 或再分发许可；上述语义均为 `UNKNOWN`；
- 只有获批 Cargo member、人工审查、回滚目标和 commit-matched evidence 才能晋级。
