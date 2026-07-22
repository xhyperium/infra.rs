# resiliencx — Design

> 当前设计对应 `resiliencx 0.1.2`；只描述单进程弹性原语。

## 重试上下文

- `RetryContext` 聚合 config、`RetrySafety`、instrumentation、op、可选 budget 与 caller seed。
- 多次尝试在首次 operation 前拒绝 `UnsafeSideEffect`；`Idempotent` 仍由调用方保证领域语义。
- budget 在 async 退避前 reserve，退避完成后 commit 并记录 retry；future 被取消时 RAII refund。
- feature `tokio` 的 deadline 包裹整次安全重试，但 cooperative cancellation 不撤销已发生副作用。

## 本地原语边界

熔断、限流与舱壁只维护进程内状态；资源不足立即拒绝，不提供公平队列、自动墙钟策略或跨进程协调。
bulkhead 从 poisoned mutex 恢复 inner，并保证 permit drop 归还容量。caller seed 用于实例去相关，
不是加密 RNG。

第三轮不扩展能力。治理修正后候选已重冻；本地独立 reviewer 已完成实现/证据审查，独立 verifier
已完成技术/证据初验。本次纯状态 delta 不改变受审源码/测试。GitHub 固定提交 CI artifact、PR、
维护者审批、合并、tag/发布仍 pending。
