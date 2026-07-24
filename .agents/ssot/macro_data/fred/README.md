# fred — FRED 数据域规格 SSOT（外部 API 事实为 `UNKNOWN`）

<!-- ssot:domain=fred -->
<!-- ssot:provenance=status=unknown; source=UNKNOWN; as_of=UNKNOWN; fixture=UNKNOWN -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=not_started -->

FRED 是待核验的宏观经济数据来源候选；机构身份、覆盖指标、序列数量和数据范围均保持 UNKNOWN，不得将候选范围写成已核验事实。

## 定位

- **类型**：provider 规格草案（不是 L0 实现）
- **角色**：计划中的原始数据适配层；当前没有客户端
- **计划依赖**：domain_macro 规格；当前 workspace 未实现 provider 类型或重试组件，外部访问语义为 `UNKNOWN`
- **当前边界**：不创建 provider trait 实现；外部事实保持 `UNKNOWN`

## 域结构

| 层 | 文件 | 说明 |
|----|------|------|
| 目标 | `goal/goal.md` | G1–G4 域目标 |
| 规格 | `spec/spec.md` | 核心规格（200~400 行） |
| 设计 | `design/design.md` | ADR 设计决策 |
| 计划 | `plan/plan.md` | 落地规划 |
| 矩阵 | `matrix/matrix.md` | 待实现条款 |
| 门禁 | `gate/gate.md` | 门禁定义 |
| 证据 | `evidence/evidence.md` | 验证证据 |
| 任务 | `tasks/README.md` | 任务分解 |
| 提示词 | `prompt/prompt.md` | Agent 提示词模板 |
| 测试 | `test/test.md` | 测试策略 |
| 审查 | `review/review.md` | 审查记录 |
| 发布 | `release/release.md` | 发布记录 |
| 回顾 | `retrospective/README.md` | 回顾 |

## 关键约束

- API 端点/认证/限流/格式：均为 `UNKNOWN`，待官方文档、访问日期和 fixture 核验；不形成访问合同
- 语言：Rust（tokio async）
