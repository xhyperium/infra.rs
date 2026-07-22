# Review: clickhousex v0.3.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `clickhousex` |
| 路径/层级 | `crates/adapters/storage/clickhouse` / L2 adapter |
| SSOT | `.agents/ssot/adapters/storage/clickhouse/` |
| 对齐文档 | `docs/ssot/clickhousex-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

clickhousex 已有 HTTP client/pool/backpressure、query/insert、AnalyticsSink、标识符和错误映射，并有 ignored live create/insert/select/ping。当前工程面可用，但批量写、池强度、真实 ClickHouse 和分析数据语义未全证，判定为部分 GO。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | client/pool/AnalyticsSink 有文档；insert schema 由调用方提供 |
| D2 类型与不变量 | 3 | identifier/empty event 校验；分析 schema 不由类型约束 |
| D3 错误处理 | 4 | HTTP status/timeout/body 映射 XError |
| D4 并发安全 | 4 | semaphore/pool close 路径有测试；live 压力未运行 |
| D5 Trait | 4 | AnalyticsSink 与 client 分离 |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 3 | HTTP 部分对齐；批量/池/分析产品 partial |
| D8 测试覆盖 | 3 | unit/mock + ignored live，批量 live 未证明 |
| D9 可观测性 | 3 | pool health/error context 有；批量吞吐 metrics 未证 |

## 3. 专项与发现

- P1：`live_smoke.rs` ignored；批量 insert、pool saturation、schema/重试/幂等没有当前 live 证据。
- P2：对外分析事件需固定 schema/version，避免仅以 `Bytes`/JSON object 通过编译即宣称可分析。

## 4. SSOT 对齐与判定

S=27/35；L1 HTTP 部分有条件 GO，QT-7 Conditional/Gap，批量分析 NO-GO。

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
