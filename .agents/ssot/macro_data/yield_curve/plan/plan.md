<!-- ssot:trace=yield_curve.plan.001 -->
# yield_curve 落地计划

| 阶段 | 交付 | 退出条件 |
|---|---|---|
| P0 | 期限/曲线类型/单位值对象 | 边界和序列化测试 |
| P1 | 来源无关的脱敏曲线 fixture | schema、单位、日期和哈希；不固化外部 URL |
| P2 | domain_macro 映射 | identity、vintage、精度不丢失；provider 适配另行核验 |
| P3 | 派生曲线算法 | 算法版本、输入审计、误差和回滚证据 |

实现路径在批准设计前保持为空；不得创建虚构 crate 作为 evidence。
