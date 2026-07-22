# AnalyticsSink 语义合同

- ownership：实现拥有后端连接；调用方提供事件名与字节 payload。
- success：`sink` 成功表示后端接受该写入请求，不单独证明查询可见或持久介质已落盘。
- failure：编码、网络、鉴权与后端拒绝必须返回可分类 `XError`。
- idempotency：幂等键、去重与重放语义当前未统一。
- cancel / timeout：由实现配置；trait 不承诺 deadline。
- ordering / delivery：不同事件之间无通用排序、批处理或投递次数保证。
- resource release：连接与响应体由实现回收。
- object-safety：`dyn AnalyticsSink` 可用。
- fake entry：`contract_testkit::FakeAnalyticsSink`。
- test entry：`contract_testkit::assert_analytics_sink` 只验证入口接受；真实 adapter 必须追加后端查询证据。

本合同不统一 schema、批量原子性、落盘确认、重试或 exactly-once。
