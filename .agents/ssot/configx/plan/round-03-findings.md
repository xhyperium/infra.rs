# configx 第 3 轮候选准备记录

| 字段 | 值 |
|---|---|
| 日期 | 2026-07-23 |
| Beads | `infra-2d9.9` |
| 战役历史起点 | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 最终 Review base | `origin/main@5fe242cefc873117d024f0d09f8ad5cbf449d2ec` |
| 当前版本 | `0.1.2`（root 已完成 PATCH bump） |
| 候选状态 | 确定性加强完成；治理修正后候选已重冻，本地 reviewer 完成，verifier 技术/证据初验完成 |

## 已闭合实现事实

- 第 2 轮已实现原子 reload、`secret:` 诊断脱敏、错误不回显原始配置行、总 deadline 有界等待、
  显式 wait outcome 与 poison 可区分路径。
- 兼容折叠 API 仍保留；生产新调用方应使用 Result / 显式 outcome API。
- reload 仍由调用方手动触发；没有自动文件 watcher、远端配置中心、后台 runtime、类型化 schema
  或 secret 托管。

## 第 3 轮机器证据

| 检查 | 结果 | 证据边界 |
|---|---|---|
| root 串行行覆盖率门禁 | `1166 / 1166`（100.0000%），exit 0 | 共享工作树本地机器证据，不是固定提交 CI artifact |
| Round 3 定向 test / clippy / doc / fmt | 退出码 0 | 41 单元 + 2 并发集成 + 7 公开 API；bench/examples 通过 |
| phase-hook 竞态测试 | 连续 100 轮通过 | 无 sleep、轮询或调度概率 |
| active / complete spec | 本轮文档收敛后要求 `cmp` 一致 | 由 writer 交付检查复验 |
| 版本一致性 | Cargo 当前为 `0.1.2` | 版本由 root 更新，域内执行者未再次 bump |

Round 1/2 的“不新增版本号”是当时执行者范围；root 已在发布准备阶段将当前版本更新为 `0.1.2`。

## Reviewer 阻断修复

- reload 锁边界测试增加仅 `cfg(test)` 的 per-watch phase hook。hook 位于 state guard 释放后、store
  替换前；Barrier 到达即精确证明 state 可取且 mutation 仍由 reload 持有，不使用轮询或 sleep。
- 上述竞态测试在冻结前连续运行 100 轮，全部通过。
- timed wait 在每轮观察前及接受 generation 前裁定 deadline；确定性 elapsed/sleeper 测试证明
  deadline 后到达或接受前跨过 deadline 的 generation 均返回 `TimedOut`，且 `seen` 不前移。
- PR 前 `codex review --base main` 发现已关闭 watch 的零时限等待被 deadline 快路径误报为
  `TimedOut`；现已将可立即观察的 `Closed` 判定前置，并补精确回归测试。
- 所有用户可见 `XError` context 改为简体中文；关键失败路径精确断言 `ErrorKind::Invalid` 与完整文案。
- 新公开 Result / wait API 补齐 `# Errors` rustdoc。
- `FileSource::load` 在保留中文 context 的同时通过 `XError::with_source` 保留 `std::io::Error`；缺失文件
  测试精确断言外层 `ErrorKind::Invalid`、完整 context、非空 source、可 downcast 及 `NotFound`。
- Round 1/2 findings 的 trailing whitespace 已清除；Round 2 旧 scoped diff-check 自报已更正为不充分，
  Round 3 使用显式 base + scope 的 `git diff --check`。

最终覆盖率由 root 在 PR 前审查修复后串行确认为 `1166 / 1166`（100.0000%），并已统一写入
治理/规格/alignment 制品；未并行运行 llvm-cov。

## 审查状态与待完成项

Round 3 已用 per-watch phase hook + Barrier 替换竞态轮询，并补 deadline 后 generation 裁定与中文错误
合同；本地独立 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验。

- Done（本地）：并发测试确定性加强完成并连续运行 100 轮；
- Done（本地）：候选已重冻；独立 reviewer 完成实现/证据审查；独立 verifier 完成技术/证据初验；
  本次纯状态 delta 不改变受审源码/测试；
- Pending：GitHub 固定提交 CI artifact；
- Pending：PR、维护者审批与合并；合并后再判断外部发布动作。

release 继续 BLOCKED；本记录不宣称 Production Ready、发布批准或 package stable。
