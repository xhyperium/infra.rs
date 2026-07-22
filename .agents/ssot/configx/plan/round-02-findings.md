# configx Round 02 — Reviewer 阻断修复

> 历史版本说明：本轮执行者未修改版本；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

状态：代码与定向测试已通过，等待冻结验证与独立复审
Base commit：`3cd29a942710c0fb42f3f6bc05e3c31570acad47`

## Reviewer 阻断

1. timed wait 首次 `Mutex::lock` 与 Condvar 重获锁可能越过总 deadline。
2. watch reload 等待 store 写锁时持有 state mutex。
3. `MemorySource` 派生 Debug 泄露 `secret:` 值，parse 错误回显原始行。
4. reload 并发测试没有证明 writer 活跃期间真实采样。
5. 伪通知测试没有就绪握手或实际通知计数。
6. secret/subset 辅助仍折叠 poison，Option wait 无法区分超时和关闭。

## 修复裁定

- 增加独立 mutation mutex 串行化 `notify / reload / close`。
- reload 在 mutation 临界区短暂检查 state 后释放 state，再等待 store；store 整图替换是配置线性化点，
  generation 发布在 mutation mutex 释放前完成。
- timed wait 改用 state `try_lock` + 最多 1ms 的剩余 deadline 轮询，不再执行无界 state lock。
- 增加 `ConfigWaitOutcome::{Changed, TimedOut, Closed}` 与显式 wait API，保留兼容 Option API。
- 增加 `try_get_secret / try_subset_snapshot`，保留原辅助函数的 poison 折叠语义。
- `MemorySource::Debug` 复用 `secret:` 脱敏视图；parse 错误只报告行号和类别。

## 确定性测试证据

| Reviewer 要求 | 测试 |
| --- | --- |
| reload 不持 state 等 store，notify 顺序明确 | `reload_releases_state_lock_while_waiting_for_store_and_serializes_notify` |
| state mutex 竞争受 deadline 限界 | `wait_timeout_is_bounded_when_state_mutex_is_held` |
| 实际伪通知有握手和计数 | `wait_timeout_deadline_survives_actual_spurious_notifications` |
| writer 临界区内真实读尝试 | `reload_readers_only_observe_complete_snapshots` 的 per-store hook + Barrier + `try_read` |
| Debug / parse 不泄密 | `memory_debug_redacts_secret_and_parse_error_omits_raw_line` |
| 显式 outcome | `explicit_wait_outcome_reports_changed_and_closed` 及 timed tests |
| Result secret/subset | 单元 poison 路径 + 公开 API 消费测试 |

首版 active count 仍允许 writer 真正取得写锁前出现采样空窗，已被复审否决。最终测试使用仅 `cfg(test)`
且绑定单个 store 的 replace hook：hook 只在 `replace_entries` 已取得写锁、尚未替换 map 时进入 Barrier；
测试线程随后断言同一 store 的 `try_read` 精确返回 `WouldBlock`，证明读尝试与 writer 写锁临界区重叠。
释放 writer 后再验证完整 new 快照；开始前已验证完整 old 快照。全程不使用 sleep 或调度概率。

## 覆盖率说明

首次 cov gate 在多个 writer 共用 llvm-cov target 时运行，测试均显示执行，但 LCOV 将已执行测试行记为
零，结果不可采信。root 已明确要求停止并行 cov；所有 writer 冻结后由 root 串行复验。

Root 早期串行复验发现唯一零覆盖是锁边界测试里 fresh mutation lock 的防御性 poison panic arm。
该 lock 在测试中只由无 panic 的 reload 路径持有，poison arm 不可达。本轮将其改为
每轮执行的“非 poisoned”断言，再以 `Ok` / 竞争结果控制循环，不排除 coverage、不使用空断言。

中间轮次曾达到 100%；Round 3 新增实现后的最终精确数字待 root 在全部 writer 冻结后串行复验。

## 冻结前定向验证

```text
cargo fmt -p configx -- --check
结果：exit 0

cargo test -p configx --all-targets
结果：exit 0；40 单元 + 2 并发集成 + 7 公开 API，bench / examples targets 通过

cargo clippy -p configx --all-targets -- -D warnings
结果：exit 0

cargo doc -p configx --no-deps
结果：exit 0

node scripts/quality-gates/cov-gate-100.mjs -p configx --filter crates/configx/src
结果：Round 3 root 串行复验 `1164 / 1164`（100.0000%），exit 0

active / complete spec cmp
结果：exit 0

scoped git diff --check
历史自报 exit 0 未包含当时 untracked findings，不能作为 base...diff 证据；Round 3 已改用显式 base + scope 命令复验
```

## 保留边界

- timed wait 使用同步轮询，不引入后台 runtime；调度器仍可造成通常的 OS 调度延迟。
- store 替换与 generation 发布有明确顺序，但不是跨两个锁的一条联合原子读 API。
- 兼容 Option / 折叠 API 仍有歧义，新生产调用方应使用 Result / 显式 outcome。
- 不包含远端配置、自动 watcher、类型化 schema 或 secret 托管。
