# 可生产消费面清单（冻结）

| 字段 | 值 |
|------|-----|
| Bead | `infra-s9t.1` |
| 日期 | 2026-07-21 |
| 状态 | **Frozen**（变更须 PR + 更新本表） |
| 关联 | [prod-trait-inventory.md](./prod-trait-inventory.md) · [status-modules-prod-followup.md](../2026-07-21-status-modules-prod-followup.md) |

> **STATUS 完成度 ≠ 可生产消费。** 本表以 **声明 SSOT 面 + 分层签字** 为准。

## 1. Allow（应用可直接依赖）

| Crate（`-p` 短名） | 最大诚实层 | 允许用途 | 禁止用途 |
|--------------------|------------|----------|----------|
| `kernel` | L1+L4 有条件 | 错误/时钟/关停原语 | 业务编排 |
| `testkit` | L1 test-support | 单测 ManualClock | 生产 runtime |
| `decimalx` | L1 有条件 | 金额 `checked_*` | panicking 运算符资金路径 |
| `canonical` | L2 committed 子集 | 已 committed DTO | 未 committed 当跨服务契约 |
| `bootstrap` | L1 装配 | 组合根 / 关停句柄 | 完整 app 生命周期平台 |
| `configx` | L1 内存 KV | 进程内字符串配置 | 唯一生产配置中心 / schema 中心 |
| `evidence` | L1 | `FileEvidenceAppender` 或自实现 trait | **仅** `InMemory*` 当合规审计 |
| `observex` | L1 tracing 面 | Instrumentation 打点 | OTEL 完成宣称 |
| `resiliencx` | L1 | 同步 `retry_fn`；async 用 `retry_async` | async 任务默认 `ThreadSleepWait` |
| `schedulex` | L1 registry | 任务 ID 集合 | timer/cron/调度执行 |
| `transportx` | L1 I/O 有条件 | HTTP/WS 客户端（自管策略） | TLS 全矩阵完成宣称 |
| `contracts` | L3 **有条件** | trait + Fake + 真入口（见下） | 无真后端即宣称 L3 |

## 2. Conditional（须真入口 / feature）

| 入口 | Feature / 条件 | 说明 |
|------|----------------|------|
| `redisx` live KV | feature `live` + 可达 Redis | 非 scaffold 验证入口（`infra-s9t.2`） |
| `postgresx` live | 未交付 | 仍 scaffold |
| exchange live | 未交付 | 仅 mock/HttpDriver |

## 3. Deny（禁止当生产后端/平台）

| 路径 | 原因 |
|------|------|
| `*Adapter` 默认 scaffold（无 `live`） | 进程内 HashMap/Vec |
| `InMemoryEvidenceAppender` 唯一审计 | 进程退出即失 |
| `schedulex::Scheduler` 当调度器 | SSOT 禁止 timer |
| `configx` 唯一配置源且无校验 | 无 schema fail-fast |
| `observex` 当 OTEL 栈 | 仅 tracing info |
| Agent 代签 L5 | Maintainer only |

## 4. STATUS 成熟度对照（七包 + 核心）

| Package | STATUS 成熟度（参考） | 消费裁定 |
|---------|----------------------|----------|
| resiliencx / transportx / contracts | active / 高分 | 按上表 Allow 条件使用 |
| configx / evidence / observex / schedulex | partial | **合同内** Allow；禁止平台幻想 |
| adapters ×9 默认 | scaffold(+mock) | Deny 除非 live 入口文档明示 |

## 5. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | 初冻（infra-s9t.1）；对齐 STATUS 全模块审计与 follow-up epic |
