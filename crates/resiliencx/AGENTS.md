# AGENTS — crates/resiliencx

- Package：`xhyper-resiliencx` · lib：`resiliencx`
- 定位：L1 **重试 + 熔断 + 令牌桶限流**
- 依赖：`xhyper-kernel` + `xhyper-contracts`；**禁止** observex；**禁止** 反向依赖 transport/domain/app
- 可观测：注入 [`contracts::Instrumentation`]（ADR-005）
- 熔断/限流：**无墙钟**（拒绝计数 / 显式 refill）
- Active SSOT：`.agents/ssot/infra/resiliencx/spec/spec.md`（镜像可能滞后；以本仓 alignment 为准）
- 验收：`cargo test -p xhyper-resiliencx`；`cov-gate-100` Lines 100%
