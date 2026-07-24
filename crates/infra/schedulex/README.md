# schedulex

std-only 的 L1 任务 ID registry 与显式 tick Job runner。

| 项 | 值 |
|---|---|
| package / lib | `schedulex` / `schedulex` |
| path | `crates/infra/schedulex` |
| current version | `0.1.1` |
| publish | `false` |
| production deps | 无 |

## 两个独立 interface

`Scheduler` 只登记 ID：

```rust
use schedulex::Scheduler;

let mut registry = Scheduler::new();
registry.schedule("job-1");
assert!(registry.cancel("job-1"));
```

`JobRunner` 只在宿主显式 tick 时执行：

```rust
use schedulex::{Job, JobRunner, Schedule};

let mut runner = JobRunner::new();
runner.add(Job::new("job-1", || Ok(())), Schedule::once(10))?;
assert_eq!(runner.tick(9).fired, 0);
assert_eq!(runner.tick(10).fired, 1);
# Ok::<(), schedulex::ScheduleError>(())
```

`add` fail-closed 校验 ID/调度；同 tick 与 metadata 按 Rust `str::cmp` 的 Job ID 字典序；时间回退被忽略；FixedDelay 与 `every:<ms>` 大跨度均不补跑。`every:<ms>` 首次 tick 立即执行，之后按上次执行时刻计算 interval。Job Err 被记录、推进状态并继续，panic 传播。

## 非目标

无真实墙钟、后台线程、async runtime、持久化恢复、misfire 产品、分布式 lease、完整 cron/时区。不得把本 crate 当作生产调度平台或 package stable。

规范：[active SSOT](../../.agents/ssot/infra/schedulex/spec/spec.md) · [alignment](../../docs/ssot/schedulex-ssot-alignment.md)

## 验证

```bash
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p schedulex --filter crates/infra/schedulex/src
```
