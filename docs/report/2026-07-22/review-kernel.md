# Review: kernel v0.3.0 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `kernel` |
| 路径/层级 | `crates/kernel` / L0 |
| SSOT | `.agents/ssot/kernel/` |
| 对齐文档 | `docs/ssot/kernel-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

kernel 是 std-only 语义信任根。`XError`、ClockDomain、checked 时间运算、ShutdownSignal 和 ComponentState 均有源码、矩阵测试、public API、coverage 与 loom 证据。当前可支持“库语义有条件 GO”，不能外推为应用或 L5 发布签核；组合根 drain 由 bootstrap 负责。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 5 | `cargo doc -D warnings`、public API baseline 通过；checked API 有文档 |
| D2 类型与不变量 | 5 | ClockDomain 阻止跨域 duration；状态转换矩阵通过 |
| D3 错误处理 | 5 | `XError` 分类、source chain、中文 Display 和 ClockError 映射 |
| D4 并发安全 | 5 | Mutex/Condvar poison 恢复、并发测试、loom 3/3 |
| D5 Trait | 4 | Clock trait 语义明确；不承担 async runtime |
| D6 依赖与版本 | 5 | workspace 依赖门禁通过，std-only 生产面 |
| D7 SSOT 对齐 | 5 | alignment 中 ClockDomain、deadline、loom 均有路径证据 |
| D8 测试覆盖 | 5 | LCOV 773/773；all-target、public API、loom 通过 |
| D9 可观测性 | 2 | L0 不直接注入 tracing，属 N/A 边界；由上层 observex 消费 |

## 3. 分层专项与发现

- `ClockDomain` 安全、`ShutdownSignal` 一次触发多观察者、`ComponentState` 非法转换均通过。
- `RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release` 通过 3/3。
- P1：无本 crate 缺陷；组合根 shutdown deadline 的实际装配证据应在 bootstrap 报告追踪。
- P2：archgate/`.architecture` 按仓库裁定 OOS，不应作为 kernel 缺陷；L5 签核仍未知。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| XError/ErrorKind | fully | PASS |
| ClockDomain/checked time | fully | PASS |
| ShutdownSignal/ComponentState | fully | PASS |
| loom/coverage/平台门禁 | fully | PASS；仍不等于 L5 |

## 5. 质量门禁

本 crate 的 build/test/fmt/clippy/doc、public-api、LCOV 和 loom 均通过；workspace 级完整结果见 [`review-workspace.md`](./review-workspace.md)。

## 6. 生产就绪判定

| 维度 | 判定 |
| --- | --- |
| L 层 | L1 + L4 有条件 |
| S 完整性 | 34/35（历史对齐基线） |
| QT | QT-6 Conditional；其余 N/A/由上层决定 |
| Go/No-Go | 有条件 GO（库语义）；不是 workspace release GO |

## 7. 建议

保持 kernel 的零 runtime 依赖和跨域类型边界；将应用关停 deadline、adapter wiring 和 L5 证据留在上层组合根与发布流程。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
