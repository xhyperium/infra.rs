# domain_macro — 宏观经济领域共享模型（L0 kernel）

<!-- ssot:domain=domain_macro -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=partial -->

生产声明：本域当前为 draft；manifest 中的 `partial` 仅表示 `macrox` 有少量既有类型，不表示下面列出的契约已经全部实现、测试或发布。

**路径**：`.agents/ssot/domain_macro/`

**对应 crate**：`crates/macrox`（L0 宏观经济数据模型核心 crate）

## 域概述

domain_macro 定义宏观经济数据的核心共享类型系统，作为整个 infra.rs macro_data 域的数据契约 SSOT（Single Source of Truth）。所有上层 crate
（指标存储、发布日历、数据管道等）均依赖本域定义的类型。

## 核心类型

| 类型 | 说明 |
|------|------|
| `CountryCode` | ISO 3166-1 alpha-2，两位大写 ASCII 字母 |
| `CurrencyCode` | ISO 4217，三位大写 ASCII 字母 |
| `EconomicRegion` | 经济体分类枚举 |
| `Country` | 国家/地区聚合 |
| `IndicatorId` | 指标标识符，格式 `<CAT>.<NAME>` |
| `IndicatorCategory` | 指标大类枚举 |
| `Frequency` | 发布频率 |
| `Unit` | 指标单位（含货币计价） |
| `Indicator` | 指标完整定义 |
| `PointStatus` | 数据点状态 |
| `Revision` | 修订记录 |
| `Observation` | 规格中的原子观测值（当前 crate 尚未实现） |
| `PointKey` | 数据点唯一键 |
| `ReleaseCalendarEntry` | 发布日历条目 |
| `ReleaseCalendar` | 发布日历聚合 |
| `MacroState` | 一致性快照 |
| `MacroDiff` | 快照差异 |
| `CalendarStatus` | 日历条目状态 |
| `MacroError` | 错误类型 |

## 文件索引

| 文件 | 说明 |
|------|------|
| [goal/goal.md](goal/goal.md) | G1–G4 域目标 |
| [spec/spec.md](spec/spec.md) | 核心规格（完整 Rust 类型定义 + 验证规则 + 错误 + 序列化 + 版本兼容） |
| [design/design.md](design/design.md) | ADR 设计决策 |
| [plan/plan.md](plan/plan.md) | 落地计划 |
| [matrix/matrix.md](matrix/matrix.md) | 对齐矩阵 |
| [gate/gate.md](gate/gate.md) | 门禁定义 |
| [evidence/evidence.md](evidence/evidence.md) | 验证证据 |
| [tasks/README.md](tasks/README.md) | 任务分解 |
| [prompt/prompt.md](prompt/prompt.md) | Agent 提示词模板 |
| [test/test.md](test/test.md) | 测试策略 |
| [review/review.md](review/review.md) | 审查记录 |
| [release/release.md](release/release.md) | 发布记录 |
| [retrospective/README.md](retrospective/README.md) | 回顾占位 |

## 验证规则速查

| 编号 | 规则 |
|------|------|
| V-1 | CountryCode 必须两位大写 ASCII 字母 |
| V-2 | CurrencyCode 必须三位大写 ASCII 字母 |
| V-3 | IndicatorId 格式 `<CAT>.<NAME>` |
| V-4 | Observation.value 与 unit 语义一致 |
| V-5 | Revision.revision_number 递增 |
| V-6 | PointKey 不可重复 |
| V-7 | scheduled_date 不可为空 |

## 变更管理

本 SSOT 目录变更须走 **worktree + PR** 流程，禁止直接在 `main` 上修改。详见
[AGENTS.md](../../../AGENTS.md) 与 [worktree-policy.md](../../../docs/governance/worktree-policy.md)。
