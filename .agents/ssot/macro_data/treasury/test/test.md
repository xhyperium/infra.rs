<!-- ssot:trace=treasury.test.001 -->
# treasury — 离线测试策略

当前只运行脱敏财政记录 fixture 和本地纯数据桩；不读取认证材料、不访问外部服务、不以跳过联网测试形成证据。

- 覆盖合法、缺字段、未知字段、坏数值、重复身份、乱序期间、修订追加和缺失原因；
- 覆盖单位、精度、排序、错误脱敏和 `domain_macro` 映射；
- 覆盖 secret sentinel 不进入 Debug、Display、Serialize、错误、tracing 或原始响应；
- 合同、授权、脱敏配置、人工审查和回滚证据齐全后，才能另提 provider 测试任务。

来源字段、访问方式、认证、错误映射、参数、缓存、限流和再分发均为 `UNKNOWN`，本层不定义客户端或请求合同。
