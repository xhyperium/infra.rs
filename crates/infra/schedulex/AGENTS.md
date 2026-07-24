# AGENTS.md — schedulex

> 仓库级规则见 [`../../AGENTS.md`](../../AGENTS.md)。
> 权威规范：[SPEC-SCHEDULEX-003](../../.agents/ssot/infra/schedulex/spec/spec.md)

## 身份

- package/lib：`schedulex`；L1；`publish = false`
- `Scheduler`：任务 ID registry seam
- `JobRunner`：独立、进程内、宿主显式 `tick(now_ms)` 的 deterministic seam
- 两者不自动联动；本 crate 不是后台或分布式 scheduler

## 强制约束

- 生产依赖 std-only；`[dependencies]` 为空，`default = []`
- 禁止 Clock、真实墙钟、sleep、后台线程、tokio/async runtime
- 禁止持久化、misfire 补跑产品、lease/fencing/leader election、完整 cron/时区
- `add` 必须在插入前校验 JobId 与 Schedule；失败不得改变 runner
- 同 tick 执行和 `list_meta` 按 Rust `str::cmp` 的 Job ID 字典序
- 时间回退不执行、不推进；大跨度 tick 每 Job 最多执行一次
- `every:<ms>` 首次 tick 立即执行，随后按上次执行时刻计算 interval；Err 也推进
- Job Err 推进状态并继续；panic 传播
- 用户可见错误为简体中文

## 验证

```bash
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/cov-gate-100.mjs -p schedulex --filter crates/infra/schedulex/src
```

不得把本地测试通过外推为 package stable、生产平台 readiness 或分布式能力。
