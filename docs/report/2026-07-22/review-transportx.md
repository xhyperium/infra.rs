# Review: transportx v0.1.2 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `transportx` |
| 路径/层级 | `crates/transport` / L1 |
| SSOT | `.agents/ssot/infra/transport/` |
| 对齐文档 | `docs/ssot/transport-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

transportx 提供 Reqwest HTTP、WebSocket connector/connection、body/frame limits、TLS system/custom/insecure、proxy 和 bounded pool。mock/fixture、错误映射、敏感 header Debug 脱敏和 pool poison 测试通过；实际代理/TLS 后端矩阵仍需环境证据。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | limits/timeouts/error mapping 有文档；调用方仍可选择 insecure |
| D2 类型与不变量 | 4 | body/frame fail-closed、TLS/proxy builder、pool limits |
| D3 错误处理 | 4 | HTTP/WS status/timeout/closed 映射 XError |
| D4 并发安全 | 4 | pool checkout/release/poison tests；实际连接器压力未运行 |
| D5 Trait | 4 | HttpDriver/WsConnector/WsConnection 支持 mock 与 dyn |
| D6 依赖与版本 | 5 | workspace dependency gate 通过 |
| D7 SSOT 对齐 | 4 | TLS/pool/proxy 已实现；部署默认策略需按应用确认 |
| D8 测试覆盖 | 4 |  limits、mock HTTP、reqwest、websocket、public API 通过 |
| D9 可观测性 | 3 | 错误有上下文；无完整请求指标/追踪策略 |

## 3. 专项与发现

- TLS builder 支持 system/custom CA/insecure；代理 Debug 脱敏。
- P1：insecure/custom TLS 选项必须受部署策略或组合根约束，不能仅由 builder API 约束。
- P2：补充真实 TLS 握手、代理 CONNECT、pool 饱和和长连接断线测试；exchange 还需要重连策略。

## 4. SSOT 对齐与判定

transport I/O 面 fully/mostly 对齐，真实部署矩阵 partial；L1 有条件 GO，S=33/35，QT-1/2 Conditional。

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
