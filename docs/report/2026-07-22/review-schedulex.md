# Review: schedulex v0.1.1 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `schedulex` |
| 路径/层级 | `crates/schedulex` / L1 |
| SSOT | `.agents/ssot/schedulex/` |
| 对齐文档 | `docs/ssot/schedulex-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

schedulex 是由调用方注入 `now_ms` 的确定性 JobRunner，支持 once、fixed delay、every-ms/分钟 cron、取消、统计和错误继续执行。它已超出单纯 registry，但仍是进程内 tick runner；没有线程 timer、持久化、分布式 leader 或跨实例去重。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | checked id/schedule、tick API 有文档；job closure 错误由结果返回 |
| D2 类型与不变量 | 4 | JobId 校验、zero fixed delay 拒绝、once/fired 状态 |
| D3 错误处理 | 4 | ScheduleError 与 TickResult.errors；用户文案局部英文技术文本 |
| D4 并发安全 | 3 | runner 本身单线程可变借用，避免数据竞争；不提供并发执行模型 |
| D5 Trait | 4 | JobFn 语义清晰，但 scheduler 不提供异步 job trait |
| D6 依赖与版本 | 5 | workspace dependency gate 通过 |
| D7 SSOT 对齐 | 4 | tick/cron/job 代码已实现；分布式调度明确为边界 |
| D8 测试覆盖 | 4 | schedule、runner、bulk、public API 通过；无跨进程测试 |
| D9 可观测性 | 2 | stats/status line 有本地面；无 tracing integration |

## 3. 专项与发现

- `JobRunner::tick` 收集到期 ID 后运行，单个 job 错误会记录并继续；同一 runner 的借用使 map 结构安全。
- `runner.rs:107` 的 `expect("due id must exist")` 依赖内部 due snapshot 不被并发修改；当前 `&mut self` API 保证该前提，属于内部 invariant，不是外部输入 panic。
- P2：对外命名须明确是 deterministic tick runner，不能宣称分布式调度或 cron service。

## 4. SSOT 对齐

| 条目 | 状态 | 结论 |
| --- | --- | --- |
| JobId/Schedule/JobRunner::tick | fully | PASS |
| timer/cron 执行线程 | partial/out of scope | OPEN |
| distributed scheduling | missing | N/A/NO-GO |

## 5. 质量门禁与判定

workspace 门禁通过；进程内 L1 有条件 GO，S=33/35，QT-5 Conditional，分布式调度 NO-GO。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
