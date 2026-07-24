# bea — 美国经济分析局（Bureau of Economic Analysis）

<!-- ssot:domain=bea -->
<!-- ssot:provenance=status=unknown; source=UNKNOWN; as_of=UNKNOWN; fixture=UNKNOWN -->
<!-- ssot:spec_status=draft -->
<!-- ssot:implementation_status=not_started -->

**路径**：`.agents/ssot/bea/`

**对应实现：尚未批准；禁止将 provider I/O 放入 macrox L0**

## 域概述

bea 域当前只维护脱敏宏观数据 fixture 的离线解析边界。来源、数据集、字段、访问方式、认证、配额、许可和再分发语义均为 `UNKNOWN`；没有 provider 实现，也没有进入 `domain_macro` 的代码路径。

## 当前可交付范围

- 保留来源身份、期间、单位、频率、修订和缺失原因；
- 对合法、缺失、未知和坏输入执行纯数据解析；
- 通过人工审查、脱敏 fixture、门禁和 commit-matched evidence 后，才可另提来源核验任务。

任何外部服务能力、请求模型、响应格式、错误码、配额数字或重试策略在证据核验前都不得写入本域。

## 文件索引

| 文件 | 说明 |
|------|------|
| [goal/goal.md](goal/goal.md) | G1–G4 域目标 |
| [spec/spec.md](spec/spec.md) | 离线解析边界草案；不提供完整 Rust 类型、API 合同或分片实现 |
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

## 依赖

- 计划依赖 `domain_macro` 规格；当前 workspace 仅有部分既有类型，不能作为 provider API 依赖
- `serde` / `serde_json` / `serde_xml_rs`：序列化

## 变更管理

本 SSOT 目录变更须走 **worktree + PR** 流程，禁止直接在 `main` 上修改。
