# Review: evidence v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `evidence` |
| 路径/层级 | `crates/evidence` / L1 |
| SSOT | `.agents/ssot/tools/evidence/` |
| 对齐文档 | `docs/ssot/evidence-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

evidence 提供内存/文件追加、查询、远端传输和 HMAC-SHA256 签名，依赖面小且无 unsafe。它适合作为开发默认的追加面；合规审计所需的不可变、密钥生命周期、远端幂等、错误分类和 durable failure 证据仍 OPEN。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | 追加/查询/签名 API 有文档；文件读取与锁边界仍需强化 |
| D2 类型与不变量 | 5 | seq 为 u64、签名固定 32 字节、forbid unsafe |
| D3 错误处理 | 2 | `EvidenceError` 仅少量变体，未统一 XError；Display 有英文 |
| D4 并发安全 | 3 | lock 错误路径有的返回 Err；历史路径仍需统一审查 |
| D5 Trait | 4 | EvidenceAppender/Transport/Query 可组合；文件查询能力需保持一致 |
| D6 依赖与版本 | 5 | sha2 workspace 依赖，门禁通过 |
| D7 SSOT 对齐 | 3 | append/query/sign/remote 存在；SSOT 规格与合规 gate 较薄 |
| D8 测试覆盖 | 3 | unit/integration/surface/bench 通过；无本轮 live durable/fuzz 证据 |
| D9 可观测性 | 2 | 本 crate 无 tracing/metrics；调用方可注入但无默认信号 |

## 3. 专项与发现

- HMAC key 由调用方注入；签名计算和 constant-time compare 路径有测试。
- `remote.rs:92-100` 先在本地追加并占用 seq，再发送传输；远端失败留下本地记录，重试语义未协议化。
- P2：`EvidenceError` 用户可见 Display 与 XError/错误分类需中文化/统一；合规产品不能只凭内存/文件实现宣称不可变审计。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| append/query/sign/remote | fully | PASS（声明面） |
| durable immutable audit | partial | OPEN |
| key management/rotation | missing | OPEN/非本 crate 默认能力 |

## 5. 质量门禁与判定

workspace 门禁通过；历史专项 evidence 测试与 coverage 通过。L1 append 有条件 GO，S=30/35（采用 defer-close 后基线），QT-4 Conditional，合规审计 NO-GO。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
