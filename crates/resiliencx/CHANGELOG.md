# Changelog — resiliencx

## [Unreleased]

当前无新增条目。

## [0.1.2] - 2026-07-23

> 本节记录工作树中的 Cargo 版本与候选变更，不代表已创建 tag 或完成外部发布。

### 安全加固

- 新增 generic safety-aware Adapter budget 同步/异步入口；显式 `RetrySafety` 在首次 operation 前校验。
- 新增明确标注 unchecked compatibility 的 generic async budget core；safe async 入口校验后委托该核心，
  Postgres/Redis legacy async wrapper 共享标准 budget 错误与失败 attempt 观测语义。
- 明确完整 unchecked compatibility API 清单；不再把所有 budget/retry 入口描述为生产安全。
- 新增 `RetryContext`、`RetrySafety::{ReadOnly, Idempotent, UnsafeSideEffect}` 与生产安全同步/异步入口；
  `max_attempts > 1` 时在首次调用前拒绝不安全副作用。
- feature `tokio` 新增整次 retry deadline；超时映射 `XError::deadline_exceeded`，并明确
  cooperative cancellation 不撤销已发生副作用。
- async retry budget 与同步语义对齐；预算耗尽统一返回标准 budget 错误。
- async 退避使用预算 reservation；deadline 在退避期取消时 RAII refund，且不记录虚假 retry。
- 修正 `call_with_retry_budget`：`record_retry` 观测刚失败的 attempt，且不记录未实际发起的 retry。
- 修复 `BulkheadPermit` 在状态锁 poisoned 时无法归还槽位导致的永久容量泄漏。
- 新增调用方 seed jitter 入口；保留 attempt-only 兼容入口并明确其不具备抗群聚保证。

### 新增

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法

### 新增

- **退避**：`Backoff::{Constant, Exponential}`、`retry_delay_ms`
- **确定性 jitter**：`jitter_bps` + `apply_deterministic_jitter`
- **可注入 wait**：`Wait` / `ThreadSleepWait` / `NoWait` / `RecordingWait` / `retry_fn_with_wait`
- `RetryConfig::fixed` 便捷构造

### 诚实边界

- 默认 `retry_fn` 仍可能 `thread::sleep`（非 async runtime wait）
- jitter 为确定性伪随机，非加密 RNG；调用方 seed 仅用于去相关
- 低层兼容 retry API 不执行 `RetrySafety` 校验
- deadline 不撤销已发生的外部副作用
- 本地限流/舱壁仅立即拒绝，不提供排队或分布式协调
- PostgresPool 当前没有 budget 自动接线；Redis client 已按操作语义迁移到安全 wrapper
- package stable：**仍未声明**

## [0.1.0] - 2026-07-21

- 重试 / 熔断 / 限流 / 舱壁初始落地；LCOV 100%
