# TimeSeriesStore 语义合同

- ownership：实现管理连接；调用方提供表名、点与查询闭区间。
- success：写入成功后，后端支持的可见性边界内可按时间范围查询。
- failure：编码、网络、鉴权与后端错误必须返回可分类 `XError`。
- idempotency：重复点、去重键与 upsert 语义当前未统一。
- cancel / timeout：由实现配置；trait 不承诺 deadline。
- ordering：查询结果顺序未统一，调用方不得依赖返回顺序。
- time precision：调用方必须提供已按具体后端精度对齐的时间戳。
- resource release：连接与响应体由实现回收；测试表清理由 adapter profile 负责。
- object-safety：`dyn TimeSeriesStore` 可用。
- fake entry：`contract_testkit::FakeTimeSeriesStore`。
- test entry：`contract_testkit::assert_time_series_store`；只验证单点 write/query 可读。

本合同不统一 retention、分区、乱序写、聚合、schema migration 或 exactly-once。
