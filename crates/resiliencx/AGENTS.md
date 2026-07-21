# AGENTS — crates/resiliencx

- Package：`xhyper-resiliencx` · lib：`resiliencx`
- 定位：L1 **重试**；熔断/限流 **未实现**，禁止当已交付
- 依赖：仅 `xhyper-kernel`；**禁止** observex；**禁止** 反向依赖 transport/domain/app
- 可观测：注入 [`contracts::Instrumentation`]（ADR-005）；**禁止**依赖 observex
- Active SSOT：`.agents/ssot/infra/resiliencx/spec/spec.md`
- 验收：`cargo test -p xhyper-resiliencx`；覆盖率目标 Lines 100%
