<!-- ssot:trace=domain_macro.goal.001 -->
# domain_macro 目标

> 目标状态：草案；批准状态和责任人待补。以下目标不代表当前 crate 已完成。

- G1 类型安全：所有外部输入经验证值对象进入核心域，不产生非法状态或 panic。
- G2 观测身份：source/series/indicator/subject/period/vintage 可稳定区分，禁止静默覆盖。
- G3 时间与修订：期间、发布时间、as-of、vintage 和 revision chain 语义可测试、可回放。
- G4 Wire 兼容：JSON envelope、Decimal/缺失值、N-1 迁移和回滚可验证。

非目标：HTTP、认证、缓存、重试、代理、供应商 API 和发布日历抓取。
