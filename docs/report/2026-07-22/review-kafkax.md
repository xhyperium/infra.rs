# Review: kafkax v0.3.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `kafkax` |
| 路径/层级 | `crates/adapters/storage/kafka` / L2 adapter |
| SSOT | `.agents/ssot/adapters/storage/kafka/` |
| 对齐文档 | `docs/ssot/kafkax-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

kafkax 默认使用 rskafka，提供 producer/partition consumer、EventBus AMO facade、持久化 offset、应用层 AtLeastOnceConsumer/KafkaAtLeastOnceBus 与 EosSession。代码和注释明确 rskafka 无 group coordinator/transactional producer；真实 broker 测试存在但 ignored。本轮未运行，EOS 不能视为原生 Kafka 事务保证。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | pool/producer/consumer/ALO APIs 有文档；分区限制需显式理解 |
| D2 类型与不变量 | 4 | Delivery/offset/ack pending/closed state 有约束 |
| D3 错误处理 | 4 | broker/config/error mapper 映射 XError |
| D4 并发安全 | 4 | mpsc/consumer ownership；broker 并发未运行 |
| D5 Trait | 4 | EventBus facade 与 ALO 扩展分离；无原生 group trait |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 4 | AMO/ALO/EOS 代码面存在；live/原生 EOS 边界已声明 |
| D8 测试覆盖 | 4 | unit/mock/offset + ignored broker live |
| D9 可观测性 | 3 | health/stats 有；consumer lag/commit metrics 未证 |

## 3. 专项与发现

- `src/bus.rs` 明确 EventBus 无 ack/redelivery，默认是 at-most-once；这不是缺陷，但必须防止上层误读。
- P0：QT-Ship-3 不能由类型名自动满足；部署必须选择 AMO、应用层 ALO 或明确 EOS 语义，并运行 broker-backed conformance。
- P1：live_event_bus ignored，且当前消费设计不依赖 coordinator；多实例消费/重平衡/崩溃恢复不能宣称支持。

## 4. SSOT 对齐与判定

S=31/35；L1 AMO/应用层 ALO 有条件 GO，原生 EOS/完整 consumer group NO-GO，QT-4 Conditional/Gap（取部署语义）。

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
