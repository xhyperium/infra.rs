# Review: canonical v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `canonical` |
| 路径/层级 | `crates/types/canonical` / Types/L2 committed subset |
| SSOT | `.agents/ssot/types/canonical/` |
| 对齐文档 | `docs/ssot/types-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

canonical 已具备 Envelope schema_version、DTO shape 校验、`deny_unknown_fields`、双向 golden、N-1 fixture 和拒绝样例。当前可判定 committed wire subset 有条件 GO；这不代表所有 venue/业务协议或未来 schema 都稳定。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 5 | public surface 与 doc 通过 |
| D2 类型与不变量 | 5 | VenueId/InstrumentId/OrderRef 等类型化，shape 函数存在 |
| D3 错误处理 | 4 | serde/shape 错误可观察；业务错误由上层负责 |
| D4 并发安全 | 4 | 不含共享可变状态，主要 N/A |
| D5 Trait | 4 | DTO 无业务方法；wire 边界清晰 |
| D6 依赖与版本 | 5 | workspace dependency gate 通过 |
| D7 SSOT 对齐 | 5 | envelope 与 alignment/fixtures 一致 |
| D8 测试覆盖 | 5 | LCOV 679/679、44 unit + public/golden 通过 |
| D9 可观测性 | 1 | wire 类型不适用 tracing |

## 3. 专项与发现

- committed DTO 逐项覆盖 legacy fixture、N-1、unknown-field/非法 scale 拒绝。
- P2：继续维护 wire inventory、schema version 和 migration 文档；不要把“全 DTO 可 serde”当作业务协议 live。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| Envelope/schema_version | fully | PASS |
| committed DTO v1–v1.3 | fully | PASS |
| 交易所全量协议 | missing/out of scope | OPEN，不属于 canonical 当前交付 |

## 5. 质量门禁与判定

canonical align、build/test/fmt/clippy/doc、public API 和 LCOV 均通过；L2 committed subset 有条件 GO，S=33/35，QT-1/2/4/7 仅类型面 Conditional。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
