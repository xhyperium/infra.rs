<!-- ssot:trace=yield_curve.test.001 -->
# yield_curve 测试策略

- 值对象：期限单位、12M/1Y convention、币种、估值/结算日、day-count、复利、排序、零值、超大值、非法单位。
- 曲线类型：par/zero/discount/derived 的互斥性、官方/派生身份分离和算法版本。
- 身份：来源、series、币种、日期、期限、曲线类型、convention 和 vintage 任一不同都不得碰撞。
- 数值：非有限值、舍入、原始精度、百分点/小数口径、插值/外推边界和误差界。
- 故障：重复观测、缺失 required point、跨 convention 混合、部分曲线失败、fixture 哈希错误、未知列和回滚。
- 恢复：停止摄取、隔离批次、重放脱敏输入、schema/算法版本回退和恢复结果比较。
- 联网测试默认不执行；只有有书面授权且脱敏输出时才可加入显式 ignored 场景。
