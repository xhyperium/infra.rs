<!-- ssot:trace=eastmoney.test.001 -->
# eastmoney — 离线测试策略

当前 `offline_fixture_only`/`unknown`，只运行脱敏 fixture 和本地纯数据桩；不启动网络监听、不读取 Cookie/token、不进行联网验收，也不以跳过联网测试形成绿色证据。

## 必测场景

- 正常、空响应、缺字段、未知字段、坏数值和重复身份；
- 单位缩放、分页字段、修订追加、来源身份和缺失原因；
- 拒绝、挑战、配额和未知权限返回稳定 `access_denied` 语义；
- sentinel secret 不进入 Debug、Display、Serialize、URL、错误、tracing 或原始响应。

只有书面授权、合同、脱敏配置、人工审查、显式 CI 开关和回滚证据齐全后，才能另提网络测试任务。
