# Review: bootstrap v0.3.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `bootstrap` |
| 路径/层级 | `crates/bootstrap` / L1 组合根 |
| SSOT | `.agents/ssot/infra/bootstrap/` |
| 对齐文档 | `docs/ssot/bootstrap-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

bootstrap 提供 PlatformContext、typed StoreSet、ShutdownController、AsyncDrain 和 instrumentation/evidence 注入。组合根 API 与本地测试通过，但当前证据只证明“可接线的组合面”，没有证明真实 exchange 与 storage 已从 bootstrap 闭合，因此交易装配仍 NO-GO。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | try_build/register_drain 有错误；`build` 是显式 panic 语义需谨慎使用 |
| D2 类型与不变量 | 4 | Bounded* context 和 StoreSet 限制能力；Option 表达未注入 |
| D3 错误处理 | 5 | BootstrapError、source chain、XError 映射测试通过 |
| D4 并发安全 | 4 | kernel ShutdownSignal 注入，drain 语义有序；真实运行负载未证 |
| D5 Trait | 4 | Bounded traits 缩小接口；真实 adapter capability 仍需注入 |
| D6 依赖与版本 | 5 | workspace dependency gate 通过 |
| D7 SSOT 对齐 | 4 | StoreSet/drain 已有路径；产品 wiring/应用示例仍 partial |
| D8 测试覆盖 | 4 | 组合根、drain、StoreSet tests 通过；缺完整真实链路 |
| D9 可观测性 | 4 | instrumentation/evidence 注入面存在；外部 exporter 不是本 crate 责任 |

## 3. 专项与发现

- `StoreSet` 能按 capability 提供 typed/Arc accessor；`AsyncDrain` 支持顺序与失败策略。
- P0：未发现 bootstrap 单元构建缺陷，但 QT-Ship-1 缺真实 bootstrap→adapter→backend 证据。
- P1：若应用只使用 Bounded placeholder，接口成功不表示生产客户端已连接；应在组合根验证能力 profile。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| typed StoreSet | fully | PASS |
| graceful drain | fully | PASS |
| real adapter wiring | partial | OPEN |

## 5. 质量门禁与判定

workspace build/test/fmt/clippy/doc 通过；L1 声明面有条件 GO，S=33/35，QT-1/2/4/6 为横切 Conditional，整体交易装配 NO-GO。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
