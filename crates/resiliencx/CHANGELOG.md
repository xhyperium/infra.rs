# Changelog — resiliencx

本文件记录 resiliencx 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### 新增

- **熔断** `CircuitBreaker` / `CircuitConfig` / `CircuitState`（三态；无墙钟）
- **限流** `RateLimiter` / `RateLimitConfig`（令牌桶；显式 `refill`）
- 模块拆分：`retry` / `circuit` / `rate_limit`

### 诚实边界

- bulkhead / async wait / backoff / budget：**仍未实现**
- 熔断 Open→HalfOpen 用拒绝计数推进（非墙钟冷却）
- 限流不自动按时间 refill

## [0.1.0] - 2026-07-21

### 新增

- 初始落地：`RetryConfig`、`retry_fn`、`Instrumentation` / `NoopInstrumentation`、`retry_ok` / `retry_downcast`（`RetryValue`）
- 行为对齐 active SSOT §2：attempts 含首次、zero→Invalid、仅 Transient 重试、耗尽返回最后错误、`record_retry`、可选阻塞 sleep
- 契约 + 库外 integration 测试；llvm-cov **Lines 100%**

### 诚实边界

- `thread::sleep` 为已知生产差距，非批准 wait 合同
- **≠** package stable
