# PubSub 语义合同

- ownership：实现管理订阅连接与 stream；调用方消费 `BusMessage`。
- success：`sub_channel` 成功只表示流已创建；`pub_message` 成功只表示 publish 请求被接受。
- failure：连接、鉴权与后端错误必须返回可分类 `XError`；stream item 当前没有运行期错误通道。
- delivery：当前最小面最多表达 at-most-once 能力，**不**等于消息必达。
- replay / ordering：不承诺历史回放、全局顺序、重复抑制或稳定 ID。
- cancel / timeout：drop stream 结束本地消费；远端取消时点由实现定义。
- backpressure：trait 未暴露容量或 lag 信号。
- object-safety：`dyn PubSub` 可用。
- fake entry：`contract_testkit::FakePubSub`。
- test entry：`contract_testkit::assert_pub_sub_surface` 先订阅再发布，但不 poll、不宣称投递成功。

需要 durable、ack、redelivery 或 consumer group 时必须使用后端专用扩展合同。
