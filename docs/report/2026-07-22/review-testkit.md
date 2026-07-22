# Review: testkit v0.1.2 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `testkit` |
| 路径/层级 | `crates/testkit` / T0，仅测试支持 |
| SSOT | `.agents/ssot/testkit/` |
| 对齐文档 | `docs/ssot/testkit-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

testkit 提供 ManualClock 与 IntegrationHarness，生产依赖仅 kernel，且未进入 production graph。锁内快照、fault 注入、单调时间与 harness 步骤具有确定性测试；其 GO 只能解释为 dev/test-support GO。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 5 | public surface、doc 和 constructor 测试通过 |
| D2 类型与不变量 | 5 | wall/monotonic 分离，fault 与 snapshot 同锁 |
| D3 错误处理 | 4 | `ManualClockError` 有分类；示例/测试中的 expect 不属于生产路径 |
| D4 并发安全 | 5 | 并发 readers/control、poison recovery、确定性测试通过 |
| D5 Trait | 4 | Clock 投影清晰；harness API 明确测试语义 |
| D6 依赖与版本 | 5 | production 仅 kernel，依赖门禁通过 |
| D7 SSOT 对齐 | 5 | ManualClock、harness 与 alignment 路径一致 |
| D8 测试覆盖 | 5 | unit、integration、property、public surface 全通过 |
| D9 可观测性 | 1 | 测试支持 crate 不适用 tracing；由业务 crate 验证 |

## 3. 专项与发现

- 跨 ManualClock domain 比较返回不可用结果而非静默比较；monotonic regression/overflow 不修改状态。
- `IntegrationHarness::run` 停在首个失败并保留记录，适合确定性契约测试。
- P2：不要将 testkit 的 `expect` 统计误报为生产 panic；示例文档应继续明确“测试用途”。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| ManualClock | fully | PASS |
| IntegrationHarness | fully | PASS（仅测试） |
| production graph 隔离 | fully | PASS |

## 5. 质量门禁与判定

workspace 门禁、testkit all-target 与 doc 通过；专项结果见 [`review-workspace.md`](./review-workspace.md)。L1 test-support 有条件 GO，S=33/35，QT 全部 N/A。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
