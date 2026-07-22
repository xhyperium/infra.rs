# Review: contracts v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `contracts` |
| 路径/层级 | `crates/contracts` / Contracts/L3 subset |
| SSOT | `.agents/ssot/contracts/` |
| 对齐文档 | `docs/ssot/contracts-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

contracts 提供 KV、EventBus、Repository、Tx、ObjectStore、TimeSeries、Analytics、Instrumentation 以及 Venue capability traits。语义文档、对象安全和 additive default 门禁已加强；`VenueAdapter` 仍保留 legacy facade，EventBus 明确是 AMO 最小面。当前为子集有条件 GO，不是全 trait L3。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | missing_docs/unreachable_pub、trait surface tests 通过 |
| D2 类型与不变量 | 4 | BusMessage/Ack、CancelOrderRequest、能力 trait 类型化 |
| D3 错误处理 | 4 | trait 约定 XError；默认 Venue 方法显式 Invalid |
| D4 并发安全 | 4 | Send/Sync bounds；具体 adapter 负责资源并发 |
| D5 Trait | 4 | docs、object safety、ExecutionVenue 无 default；legacy facade 有边界 |
| D6 依赖与版本 | 5 | 依赖集中、禁止 unsafe |
| D7 SSOT 对齐 | 4 | live helpers/profile 存在；真实各后端仍是外部证据 |
| D8 测试覆盖 | 4 | unit/integration/public API/contract-testkit 通过 |
| D9 可观测性 | 3 | Instrumentation trait 存在，具体 exporter 不在 contracts |

## 3. 专项与发现

- `ExecutionVenue` 的 structured cancel/query 没有 additive default；`VenueAdapter` 旧方法带 deprecated 文档和默认 structured 错误。
- `EventBus` 文档明确无 ack/redelivery，调用方可用 `BusMessage.id` 做幂等但不能得到 broker 语义。
- P1：`LiveContractProfile` 和 `contracts/src/live.rs` 是编排/声明辅助，不构成 real backend live；first-batch 之外的 Tx/Bus/Repo/Venue 仍需独立证据。

## 4. SSOT 对齐与判定

trait 与语义文档 mostly fully；full first-batch L3/live partial。workspace gates 通过，S=33/35，L3 子集有条件 GO，QT-2/4 依赖 adapter live。

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
