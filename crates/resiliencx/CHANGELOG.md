# Changelog — resiliencx

本文件记录 resiliencx 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### 新增

- 初始落地：`RetryConfig`、`retry_fn`、`Instrumentation` / `NoopInstrumentation`、`retry_ok` / `retry_downcast`（`RetryValue`）。
- 行为对齐 active SSOT §2：attempts 含首次、zero→Invalid、仅 Transient 重试、耗尽返回最后错误、`record_retry`、可选阻塞 sleep。
- 契约 + 库外 integration 测试；`base_delay_ms>0` 短路径覆盖 sleep 分支；llvm-cov **Lines 100%**。

### 诚实边界

- 熔断 / 限流 / bulkhead / async wait / backoff / budget：**未实现**（OPEN）。
- `thread::sleep` 为已知生产差距，非批准 wait 合同。
- **≠** package stable。
