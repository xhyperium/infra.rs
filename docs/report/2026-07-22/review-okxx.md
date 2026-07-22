# Review: okxx v0.3.3 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `okxx` |
| 路径/层级 | `crates/adapters/exchange/okx` / L2 adapter |
| SSOT | `.agents/ssot/adapters/exchange/okx/` |
| 对齐文档 | `docs/ssot/adapters-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

okxx 有 REST+WS、OKX prehash/HMAC header、structured order/cancel/query、market parser 和 mock fixture；ignored live 只覆盖公共 server time。`sCode`、503 空体和订单级错误已有测试，但私有交易、私有 WS、重连和真实签名证据仍缺，因此交易 NO-GO。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 3 | API 有文档；无 WS 时成功空流/无凭证 mock 需显式区分 |
| D2 类型与不变量 | 3 | Decimal 字符串解析较好；instrument 失败时仍有默认 tick/qty |
| D3 错误处理 | 3 | code/sCode/status 映射覆盖 fixture；未知业务码和 live 未全证 |
| D4 并发安全 | 3 | atomics/driver abstraction；长连接重连未实现 |
| D5 Trait | 4 | Venue 与 capability trait 实现完整 |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 2 | 代码路径已存在，但 private protocol/live 仍 OPEN |
| D8 测试覆盖 | 4 | 38 unit/fixture 路径；live 仅 server time |
| D9 可观测性 | 2 | 无订单/WS lifecycle telemetry |

## 3. 专项与发现

- P0：没有 testnet/demo 私有下单、撤单、查单、账户和 private WS live 证据；不能宣称可交易。
- P1：`src/adapter.rs:550-630` 无 connector 返回成功空流，断线直接结束，无重连/退避/序列恢复。
- P2：`src/adapter.rs:683-689` 无 HTTP 时返回默认 SymbolMeta；应让 mock/placeholder 能力显式化，防止误用。

## 4. SSOT 对齐与判定

REST/WS code+fixture partial；private live missing。S=24/35，L2 adapter 交易 NO-GO，QT-1/2 Gap。

## 5. 质量门禁

workspace build/test/fmt/clippy/doc 通过；`tests/live_server_time.rs` ignored，未运行。详细门禁见 [`review-workspace.md`](./review-workspace.md)。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
