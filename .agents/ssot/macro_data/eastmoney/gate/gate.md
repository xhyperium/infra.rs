<!-- ssot:trace=eastmoney.gate.001 -->
# eastmoney — 门禁

当前状态为 `draft`/`not_started`，只执行 SSOT 结构、UTF-8、追溯、路径、fixture 和 Cargo 质量门禁。

provider 晋级还必须同时满足：

- manifest 中的 provider 根路径已获批且是 workspace member；
- 官方或合同来源尚未核验；端点、字段、错误、配额、许可和再分发均必须保持 `UNKNOWN`；
- 脱敏 fixture、坏响应、单位、分页、重复身份、错误脱敏测试真实执行；
- 访问权限未核验时，拒绝/挑战/配额响应必须停止并返回稳定错误；
- matrix、evidence、测试报告和 commit SHA 完全一致。

空扫描、跳过网络测试、warning-only 结果和未绑定提交的证据均不允许晋级。
