# Review: redisx v0.3.2 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `redisx` |
| 路径/层级 | `crates/adapters/storage/redis` / L2 adapter |
| SSOT | `.agents/ssot/adapters/storage/redis/` |
| 对齐文档 | `docs/ssot/redisx-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

redisx 生产默认客户端、pool、KV trait、TTL、pubsub、TLS policy 和 resilience helper 已存在；live KV/conformance 测试提供真实入口但默认 ignored。本轮未运行 Redis，因此判定为 KV 有条件 GO，不是 Cluster/Sentinel 或 L5 ready。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | config/client/pool/trait API 有文档；环境配置错误可返回 |
| D2 类型与不变量 | 4 | TTL、pool capacity、closed 状态、Debug 脱敏 |
| D3 错误处理 | 4 | Redis error mapper 映射 XError kind |
| D4 并发安全 | 4 | pool/async client 与 lock 错误路径；live 压力未运行 |
| D5 Trait | 4 | KeyValueStore/PubSub object-safe，mock 入口存在 |
| D6 依赖与版本 | 5 | workspace gates 通过 |
| D7 SSOT 对齐 | 4 | L1 + KV L3 subset 路径存在；Cluster/Sentinel/TLS policy partial |
| D8 测试覆盖 | 4 | unit/public/mock + ignored live KV/conformance |
| D9 可观测性 | 3 | pool stats/health 存在，完整 metrics/tracing 未证 |

## 3. 专项与发现

- P1：live tests `live_kv.rs`/`live_kv_conformance.rs` 存在但本轮 ignored；不能将 Fake 或编译通过视为 Redis live PASS。
- P1：Cluster/Sentinel、强制 TLS 和 resilience 与业务操作的完整部署语义仍 OPEN。
- P2：在 bootstrap 中注入时应使用明确能力 profile，不能把单节点 KV 客户端当成生产高可用集群。

## 4. SSOT 对齐与判定

KV code surface fully/mostly 对齐；真实服务、Cluster/Sentinel partial。S=33/35，L1 + L3-KV 有条件 GO，QT-4 Conditional。

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
