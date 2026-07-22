# Review: contract-testkit v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `contract-testkit` |
| 路径/层级 | `crates/test-support/contracts` / T0，仅 dev |
| SSOT | `.agents/ssot/testkit/` §3.2 + `.agents/ssot/contracts/` |
| 对齐文档 | `docs/ssot/testkit-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

contract-testkit 提供 Fake、Recording、Batch-2 实现和 per-trait conformance suite，且不进入 production graph。它能证明 trait 表面、错误路径和最小编排行为；Fake/Mock 不能证明 Redis、Kafka、NATS、DB、OSS 或 exchange 的实际协议。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | fake/suite 有文档和 public surface |
| D2 类型与不变量 | 4 | fake 状态可观测；不模拟真实持久化约束 |
| D3 错误处理 | 4 | ContractFailure 与 XError failure 注入 |
| D4 并发安全 | 4 | fake 锁错误显式返回；测试资源为内存 |
| D5 Trait | 4 | 每个 trait suite 与 batch2 fake 覆盖 |
| D6 依赖与版本 | 5 | workspace dependency gate 通过，dev-only |
| D7 SSOT 对齐 | 4 | 挂靠 testkit/contracts 的设计明确 |
| D8 测试覆盖 | 4 | suite/fake tests 通过；不替代 live |
| D9 可观测性 | 2 | 测试工具只记录 failure，不是生产 telemetry |

## 3. 专项与发现

- Batch-2 Fake 覆盖 ObjectStore、TimeSeries、Analytics、PubSub 等新增能力。
- P1：conformance suite 若只运行 Fake 会形成“契约绿但 backend 不绿”的错觉；真实 backend profile 必须单独显示。

## 4. SSOT 对齐与判定

Fake/suite fully 对齐，真实 backend profile 仍依赖 adapters 的 ignored live tests。S=32/35（继承最新声明面），仅 dev-support 有条件 GO。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 5. 质量门禁结果

workspace build/test/fmt/clippy/doc、依赖与版本门禁的当前结果见 [`review-workspace.md`](./review-workspace.md)；本 crate 不重复宣称 ignored live 测试已运行。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
