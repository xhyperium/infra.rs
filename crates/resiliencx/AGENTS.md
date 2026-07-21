# AGENTS — crates/resiliencx

- Package：`xhyper-resiliencx` · lib：`resiliencx`
- 定位：L1 **重试（退避/jitter/可注入 wait）+ 熔断 + 限流 + 舱壁**
- 依赖：`xhyper-kernel` + `xhyper-contracts`；**禁止** observex；**禁止** 反向依赖 transport/domain/app
- 可观测：注入 [`contracts::Instrumentation`]（ADR-005）
- 熔断/限流/舱壁：**无墙钟**；wait 可注入（测试 `NoWait`/`RecordingWait`）
- Active SSOT：`.agents/ssot/resiliencx/spec/spec.md`（镜像可能滞后；以本仓 alignment 为准）
- 验收：`cargo test -p xhyper-resiliencx`；`cov-gate-100` Lines 100%
