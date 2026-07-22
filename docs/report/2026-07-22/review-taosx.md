# Review: taosx v0.3.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `taosx` |
| 路径/层级 | `crates/adapters/storage/taos` / L2 adapter |
| SSOT | `.agents/ssot/adapters/storage/taos/` |
| 对齐文档 | `docs/ssot/taosx-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

taosx 提供 REST client/pool、TimeSeriesStore、SQL/table sanitization、批次切分辅助和 native feature 面；live REST ping/write/query 测试存在但 ignored。本轮未运行 TDengine，native/批量/池强度仍不能宣称完成。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | config/client/TimeSeriesStore 有文档；SQL 仍需调用方 schema 纪律 |
| D2 类型与不变量 | 3 | identifier/table sanitization/max in flight；点/列 schema 类型化不足 |
| D3 错误处理 | 4 | REST/pool/timeout/closed 映射 XError |
| D4 并发安全 | 4 | semaphore/pool ownership；live 压力未运行 |
| D5 Trait | 4 | TimeSeriesStore 与 pool/adapter 分层 |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 3 | REST 部分对齐；native/批量/池 partial |
| D8 测试覆盖 | 3 | unit/mock + ignored live |
| D9 可观测性 | 3 | pool/error context；写入吞吐/lag metrics 未证 |

## 3. 专项与发现

- P1：批量 write、native path、pool saturation 和真实 TDengine live 仍 OPEN；`TimeSeriesStore` trait 通过不等于时序产品完成。
- P2：固定 table/column schema 与版本策略，避免调用方拼接 SQL 的隐式协议。

## 4. SSOT 对齐与判定

S=27/35；L1 REST 部分有条件 GO，QT-7 Conditional，批量/native 分析 NO-GO。

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
