# resiliencx round-02 findings

> 历史版本说明：本轮执行者未修改版本；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| 来源 | 两轴只读 code review + root 独立 all-features 门禁 |
| 范围 | 仅 round-01 resiliencx owned paths |

## 复审发现与修复

1. caller seed 最初只暴露纯计算 helper，未进入真实 retry。引入 `RetryContext`，
   `with_jitter_seed` 现驱动 safe sync/async/deadline 的实际退避，并有 `RecordingWait` 延迟断言。
2. async deadline 在退避期取消可能造成预算/观测语义冲突。新增预算 reservation：退避前原子 reserve，
   空预算立即标准错误；退避完成 commit；future 在退避期 drop 时 RAII refund，且 commit 前不记录 retry。
3. `retry_async_with_deadline` 的扁平参数过多。用 `RetryContext` 聚合 config、safety、instrumentation、op、
   可选 budget 与 caller seed；未使用 lint allow。
4. 新公共 API / 行为变化缺少 crate releases 记录与同文件单元测试。新增未发布记录，并为
   `RetryContext`、`RetrySafety`、seed helper、safe sync/async、async budget/deadline 补单元测试。
5. `docs/README.md` 包名与 Cargo 不一致、CHANGELOG 存在英文自然语言标题。已统一为 `resiliencx` 与中文标题。
6. Evidence 入口仍宣称“无战役归档”。已改为可复验命令与 findings 索引，并明确没有原始日志/CI artifact。
7. 首版 reservation 使用 eager `then_some`，失败分支仍构造临时 reservation 并在 drop 时误 refund。
   改为显式 `if/else`，并锁定 capacity=1 已耗尽后 `reserve == None && remaining == 0`。
8. 串行 LCOV 门禁发现 14 行未命中。将 `call_with_retry_budget` 改为已验证前置条件下的有界 loop，
   消除循环后不可达 fallback；测试 instrumentation 对 circuit hooks 记录并断言真实事件；补
   `retry_async_safe` 成功执行路径。未使用 coverage 排除或空断言。

## 新增回归

- 非零 capacity 已耗尽且 wait 永久 pending：失败 reserve 不复活令牌，仍在进入 wait 前立即返回标准错误；
- deadline 在 pending backoff 期间取消：reservation 自动 refund，budget 不减少、无 `record_retry`；
- 正常 async retry：恰好 commit 一个 budget token 并记录一次失败 attempt；
- caller seed 真实改变 safe retry 的 `RecordingWait` 延迟。

## 当前边界

- reservation 只保护本 crate 持有的 budget token；不撤销 operation 已发生的外部副作用。
- `RetrySafety::Idempotent` 与 seed 唯一性仍由调用方负责。
- 本地验证不是固定提交或 CI 发布证据；最终合并状态需维护者复验。

## 验证

| 命令 | 退出码 | 结果 |
|------|--------|------|
| `cargo fmt -p resiliencx -- --check` | 0 | scoped 格式通过 |
| `cargo test -p resiliencx --all-features --all-targets` | 0 | 80 个 Rust 测试通过；bench/examples targets 通过 |
| `cargo clippy -p resiliencx --all-features --all-targets -- -D warnings` | 0 | 无 warning |
| `cargo doc -p resiliencx --all-features --no-deps` | 0 | rustdoc 生成成功 |
| `node scripts/quality-gates/cov-gate-100.mjs -p resiliencx --filter crates/infra/resiliencx/src` | 0 | 994 / 994 行命中，100% |
| active/complete spec `cmp` | 0 | 双镜像一致 |
| scoped `git diff --check` | 0 | 无 whitespace 错误 |
| `crates/infra/resiliencx/Cargo.toml` base diff | 0 | 本任务未改 manifest 或版本 |

本表记录共享 worktree 的本地执行；没有原始日志归档、CI artifact、固定提交、commit 或发布动作。
共享 worktree 的 `Cargo.lock` 当前存在其他 writer 的并行 diff；本任务未编辑、格式化或回退该越界文件。
