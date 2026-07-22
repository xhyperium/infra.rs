# ObjectStore 语义合同

- ownership：实现拥有后端连接；调用方拥有 key 与 payload。
- success：`put_object` 成功只表示后端接受写入；`get_object` 成功返回原始字节。
- failure：鉴权、网络与后端错误必须映射为可分类 `XError`，不得 panic。
- idempotency：相同 key 的覆盖、条件写与版本语义当前未统一。
- cancel / timeout：由实现配置；trait 本身不承诺截止时间或取消完成。
- ordering：不同 key 或并发写之间无通用顺序保证。
- resource release：连接与响应体由实现回收；测试对象由调用方清理。
- not-found：具体 `ErrorKind` 尚未冻结，portable suite 不作断言。
- object-safety：`dyn ObjectStore` 可用。
- fake entry：`contract_testkit::FakeObjectStore`。
- test entry：`contract_testkit::assert_object_store`；只验证唯一 key 的精确字节往返。

本合同不统一 metadata、分片上传、checksum、版本化、生命周期策略或跨对象事务。
