<!-- ssot:trace=domain_macro.test.001 -->
# domain_macro 测试策略

- 值对象：大小写、Unicode、保留代码、超长输入和错误码。
- 时间：Month/Quarter/Year 边界、DST 不存在/重复时刻、跨日和晚于 as-of。
- 数值：百分点/比例、scale10、基期、Decimal 精度、NaN/∞/负零和区间边界。
- 身份与修订：跨源同点、重复插入、断号、逆序、前值不匹配、失败原子性。
- Wire：golden JSON、roundtrip、N-1、未知字段/枚举、损坏 envelope 和回滚。
- 安全：Debug/Serialize/Error/tracing 不出现秘密；任意外部输入不 panic。

真实 provider 网络测试不属于 L0 默认测试；必须显式授权、超时、脱敏并保留 fixture。
