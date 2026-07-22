# Review: goalctl v0.2.0 — 2026-07-22

| 字段 | 值 |
| --- | --- |
| 目标 crate | `goalctl` |
| 路径/层级 | `tools/goalctl` / Tools |
| SSOT | `.agents/ssot/tools/goalctl/` |
| 对齐文档 | `docs/ssot/tools-ssot-alignment.md` |
| 审查者 | AI Agent |

## 1. 概览

goalctl 提供 doctor、validate、compile，将 YAML/JSON Goal 转为带 canonical digest 的 Contract。单测覆盖 digest、校验和 public exports，workspace build/test/doc 通过；它是最小 CLI，不包含完整 goal registry、执行器或审计签核。

## 2. 通用维度评估

| 维度 | 评分 | 证据 |
| --- | ---: | --- |
| D1 公开 API | 4 | clap 子命令和 compile API 有文档；CLI 文案与错误部分英文 |
| D2 类型与不变量 | 3 | Goal model/validate 有校验；schema/未知字段策略需继续抽查 |
| D3 错误处理 | 3 | CompileError 分层，但 Display `io:/parse:/validate:` 非中文 |
| D4 并发安全 | 2 | CLI 单进程、无共享并发状态，D4 主要 N/A |
| D5 Trait | 3 | 以数据模型/API 为主，无复杂 trait |
| D6 依赖与版本 | 5 | workspace dependency/version gates 通过 |
| D7 SSOT 对齐 | 3 | 最小 Goal→Contract 面存在；完整 tools/xtask 非本仓目标 |
| D8 测试覆盖 | 4 | unit/public tests 通过；CLI subprocess UX 未全覆盖 |
| D9 可观测性 | 1 | doctor/status 仅 CLI 输出，不是 tracing |

## 3. 专项与发现

- CLI UX 的 doctor/validate/compile 子命令基本正交；当前没有 Prompt 要求的 `lint` 子命令，README/SSOT 必须明确这是最小面。
- P2：`CompileError` Display 使用英文前缀，应中文化；错误退出码可继续保持稳定。

## 4. SSOT 对齐与判定

最小 compiler 面 fully/mostly 对齐；S 由 tools alignment 约束。Tools 有条件 GO，不宣称完整目标治理平台。

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
