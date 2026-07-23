# observex — Design

状态：`observex 0.1.2` 第 3 轮候选设计边界；不宣称 package stable 或 OpenTelemetry 兼容。

## 记录链

`ExportingInstrumentation` 先调用 inner，再同步调用 `TelemetryExporter`。记录路径内化 exporter
返回的错误，并以 `catch_unwind` 隔离可展开（unwind）的 Rust panic；`panic=abort` 不可捕获。
失败调用不重试，涉及事件计入 `unconfirmed_spans/metrics`；exporter 可能已产生部分副作用，
因此 wrapper 明确不判断其实际交付或丢弃状态。

## 非阻塞责任

- `TelemetryExporter` 实现必须快速返回，不得等待外部 I/O 或执行无界阻塞。
- 允许有界、短临界区的本地同步；`InMemoryExporter` 只在 mutex 内执行容量判断、有限 Vec 更新、
  计数与状态转换。
- `catch_unwind` 只隔离 unwind panic，不能捕获 abort 或中断慢调用。违反非阻塞合同的责任在实现方。
- 真实远端 I/O 必须放在本 trait 之外的有界 worker/队列；该 worker 当前未实现。

## 诊断

`ExportingInstrumentationStats` 公开失败调用、unwind panic 调用及交付状态未知的 span/metric 数。
wrapper 不重试这些事件；计数使用原子饱和更新，并以短临界区保证多字段快照一致；
`counters_saturated` 为真时数值只能解释为下界。

## 生命周期

记录方法不返回 exporter 错误。`flush` / `shutdown` 保留 `Result`：普通错误原样返回，unwind panic
转为 `ExportError::Panicked`。`InMemoryExporter::shutdown` 在同一 mutex 临界区执行 flush-and-close。

## 第 3 轮冻结边界

本轮新树由 root 串行确认覆盖率 `942 / 942`、zeros 0、100.0000%、exit 0。治理修正后候选已重冻；
本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。本次纯状态 delta
不改变受审源码/测试。GitHub 固定提交 CI artifact、PR、维护者审批、合并、tag/发布仍 pending。
