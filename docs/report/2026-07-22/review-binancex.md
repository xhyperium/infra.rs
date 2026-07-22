# Review: binancex v0.3.2 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `binancex` |
| 路径/层级 | `crates/adapters/exchange/binance` / L2 adapter |
| SSOT | `.agents/ssot/adapters/exchange/binance/` |
| 对齐文档 | `docs/ssot/adapters-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

binancex 当前有生产默认 REST+WS 代码、HMAC、structured cancel/query、账户/余额解析和 mock HTTP/WS fixture；ignored live 仅覆盖公共 server time。无凭证路径返回明确 mock/占位状态，仍可能被错误调用方当作成功。当前交易判定 NO-GO。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 3 | REST/WS surface 有文档；mock fallback 和空流提高调用风险 |
| D2 类型与不变量 | 2 | 金额使用 Decimal，但 symbol filters 先转 f64；stepSize 未落入 meta |
| D3 错误处理 | 3 | Binance error body 与 HTTP 状态有映射；部分降级固定状态 |
| D4 并发安全 | 3 | atomics/driver trait 可共享；WS 重连与长期并发未证 |
| D5 Trait | 4 | VenueAdapter、ExecutionVenue、MarketData、Account、Time 能力实现 |
| D6 依赖与版本 | 5 | dependency/version gates 通过 |
| D7 SSOT 对齐 | 2 | 代码面已厚于 scaffold；交易 live/协议完成度仍 OPEN |
| D8 测试覆盖 | 4 | 38 unit + HTTP/WS fixtures；live 只有 ignored server time |
| D9 可观测性 | 2 | 错误上下文有限，无交易指标/订单审计面 |

## 3. 专项与发现

- P0：`tests/live_server_time.rs` 只验证公共时间，不验证签名下单、撤单、查单、账户或私有 WS，因此 QT-1/2 与 QT-Ship-6 未满足。
- P1：`src/adapter.rs:393-401` 用 `f64` 解析 tick/min/step，`417` 丢弃 step；`706-775` 无 WS connector 返回成功空流，读错误直接结束且无重连。
- P2：无凭证 `place_order`/legacy cancel/query 的 mock 结果应与生产类型/能力标记更强隔离。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| HMAC/REST 请求映射 | partial/fixture | PASS（需 live） |
| public market parser | fully tested | PASS |
| private order/account | code path only | OPEN |
| WS reconnect/private stream | missing evidence | OPEN |

## 5. 质量门禁与判定

workspace build/test/fmt/clippy/doc 通过；live ignored 未运行。S=24/35，L2 adapter 有代码证据但交易 NO-GO，QT-1/2 Gap。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
