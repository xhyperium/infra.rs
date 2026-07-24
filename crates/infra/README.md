# crates/infra/ — L1 平台能力平面

本目录归组 **L1 平台能力** crate（方案 A）。与 `types/`、`adapters/` 对称；**不是**把整个仓库都塞进 “infra”。

## 成员

| Package（`cargo -p`） | 路径 | 角色 |
|----------------------|------|------|
| `configx` | `configx/` | 本地多源配置 + reload/secret |
| `schedulex` | `schedulex/` | 任务 ID 登记 + 宿主 `JobRunner::tick` |
| `resiliencx` | `resiliencx/` | 重试 / 熔断 / 限流 / 舱壁 |
| `observex` | `observex/` | instrumentation + 有界进程内 sink |
| `transportx` | `transport/` | HTTP/WS 传输（目录名 `transport`） |
| `evidence` | `evidence/` | 审计证据追加面 |
| `bootstrap` | `bootstrap/` | 唯一组合根（ADR-016） |

## 不在本目录

| 平面 | 位置 |
|------|------|
| L0 | `crates/kernel/` |
| types | `crates/types/*` |
| ports | `crates/contracts/` |
| adapters | `crates/adapters/**` |
| test-support | `crates/testkit/` · `crates/test-support/` |
| CLI tools | `tools/*` |

## 文档

- 归组分析：[`docs/report/2026-07-24/crates-infra-grouping-analysis.md`](../../docs/report/2026-07-24/crates-infra-grouping-analysis.md)
- 迁移计划：[`docs/plans/2026-07-24-crates-infra-directory-migration.md`](../../docs/plans/2026-07-24-crates-infra-directory-migration.md)
- Agent 规则：[`crates/AGENTS.md`](../AGENTS.md)
