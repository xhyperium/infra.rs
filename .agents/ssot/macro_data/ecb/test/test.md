<!-- ssot:trace=ecb.test.001 -->
# ecb 测试策略

- 解析：合法 metadata、未知维度、字段缺失、错误媒体类型、截断 JSON。
- 数值：缺失标记、单位缩放、非有限值、精度保留。
- 身份：不同 dataflow/DSD/维度/vintage 不得碰撞。
- 兼容：当前 schema roundtrip、N-1 fixture、未知字段和未知枚举的明确策略。
- 网络：默认离线；真实端点测试必须显式 `ignore`、具备授权、超时和脱敏输出。
