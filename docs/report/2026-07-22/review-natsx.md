# Review: natsx v0.3.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `natsx` |
| 路径/层级 | `crates/adapters/storage/nats` / L2 adapter |
| SSOT | `.agents/ssot/adapters/storage/nats/` |
| 对齐文档 | `docs/ssot/natsx-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

natsx 提供 Core NATS pool/EventBus、TLS policy、health/close 与 JetStream 薄封装（stream/pull consumer）。Core EventBus 明确无历史/ack/redelivery；live pubsub/EventBus 测试存在但 ignored，本轮未运行。因此 Core 面有条件 GO，JetStream 需独立签核。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | pool/bus/JetStream/config 文档和校验 |
| D2 类型与不变量 | 4 | stream/consumer name 校验、TLS policy、closed 状态 |
| D3 错误处理 | 4 | connect/publish/subscribe/health 错误映射 |
| D4 并发安全 | 4 | async-nats ownership/pool；live 压力未运行 |
| D5 Trait | 4 | EventBus facade 与 JetStream 能力拆分 |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 4 | Core/JetStream/TLS 路径存在；durable delivery partial |
| D8 测试覆盖 | 4 | unit/mock + ignored live Core/EventBus |
| D9 可观测性 | 3 | health/stats；无完整 delivery metrics |

## 3. 专项与发现

- 非 loopback 默认 Require TLS、loopback Prefer 的策略代码存在；实际部署是否禁止明文仍由 config/组合根决定。
- P0：若产品选择 durable NATS，必须运行 JetStream stream/consumer/ack/redelivery 证据；Core EventBus 不能替代。
- P1：live tests ignored，本轮无真实 server/权限/TLS 证据。

## 4. SSOT 对齐与判定

S=31/35；Core NATS L1 有条件 GO，JetStream/可靠消息 OPEN，QT-4 Conditional/Gap。

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
