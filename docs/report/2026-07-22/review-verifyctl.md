# Review: verifyctl v0.1.0 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `verifyctl` |
| 路径/层级 | `tools/verifyctl` / Tools |
| SSOT | `.agents/ssot/tools/verifyctl/` |
| 对齐文档 | `docs/ssot/tools-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

verifyctl 提供 plan/execute/report、dry-run、changed-paths 推导和可选 evidence feature。unit/integration dry-run 通过，CLI 子命令可运行；当前只是最小 verification planner，不等于完整证据编排或生产审计系统。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 3 | plan/execute/report 有 surface；main.rs 序列化使用 expect |
| D2 类型与不变量 | 3 | Plan/RunResult schema 类型化；输入 contract 兼容策略有限 |
| D3 错误处理 | 3 | 文件/parse/execute 分支有退出码；错误文案未全中文 |
| D4 并发安全 | 3 | 每次 CLI 执行独立；无共享状态，主要 N/A |
| D5 Trait | 3 | 以命令与数据模型为主 |
| D6 依赖与版本 | 5 | workspace dependency/version gates 通过 |
| D7 SSOT 对齐 | 3 | 最小 plan/execute/report 存在；完整 evidence/release 仍 OPEN |
| D8 测试覆盖 | 4 | plan dry、execute dry、report roundtrip/public tests 通过 |
| D9 可观测性 | 2 | with-evidence 可选；默认无 telemetry |

## 3. 专项与发现

- `src/main.rs:72,117` 的 `serde_json::to_string_pretty(...).expect(...)` 是公共 CLI 未声明 panic；当前具体可序列化类型使其难以触发，但不应依赖这个假设。
- `src/plan.rs:135` 的 `unwrap_or_default()` 会把 digest 输入序列化失败压成空字符串，属于静默降级设计。
- P2：将序列化失败映射为明确 exit code；证据 append 失败不能被无声忽略；plan 检查项应与 workspace 门禁版本化。

## 4. SSOT 对齐与判定

最小 CLI 面 partial/mostly 对齐；S 以 tools alignment 为准。Tools 有条件 GO，不能宣称完整审计证据、release signoff 或所有检查已执行。

## 5. 质量门禁

workspace build/test/fmt/clippy/doc 通过；详细结果见 [`review-workspace.md`](./review-workspace.md)。

> 本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计。

## 6. 生产就绪判定

本 crate 的层级、S1–S7 与 QT 判定以本报告上文和 workspace 综合报告为准；不能外推为 L5。

## 7. 综合建议

按本报告 P0/P1/P2 顺序补齐能力边界，并在对应真实后端或交易所环境中留下可复现实证。

## 8. 变更记录

2026-07-22：按 `review-prompt.md` v1.0 补充逐 package 审查报告。

## 9. 限制声明

本审查为 AI 辅助代码审查，不替代 Maintainer 人类签核与安全审计；历史、mock、fixture 和 ignored live 入口不等同于 live PASS。
