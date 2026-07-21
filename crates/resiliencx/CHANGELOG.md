# Changelog — resiliencx

本文件记录 resiliencx 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### 新增

- **舱壁** `Bulkhead` / `BulkheadConfig` / `BulkheadPermit`（并发上限；RAII 归还）

### 诚实边界

- async wait / backoff / budget：**仍未实现**
- 舱壁无排队超时（满载立即拒绝）

## [0.1.0] - 2026-07-21

### 新增

- 重试：`RetryConfig`、`retry_fn`、`retry_ok` / `retry_downcast`
- 熔断：`CircuitBreaker` / `CircuitConfig` / `CircuitState`（三态；无墙钟）
- 限流：`RateLimiter` / `RateLimitConfig`（令牌桶；显式 `refill`）
- `Instrumentation` re-export + `NoopInstrumentation`
- 模块拆分：`retry` / `circuit` / `rate_limit`
- llvm-cov **Lines 100%**

### 诚实边界

- `thread::sleep` 为已知生产差距，非批准 wait 合同
- 熔断 Open→HalfOpen 用拒绝计数推进（非墙钟冷却）
- 限流不自动按时间 refill
- **≠** package stable
