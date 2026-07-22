# `schedulex` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.1`：任务 ID 登记表（active SSOT registry）+ 确定性 tick 驱动 JobRunner（additive 面）；**非** 完整调度器/执行器 |
| Package / lib | `schedulex` / `schedulex`（别名 `xhyper-schedulex` 仅作废弃兼容标签 / dual-mirror 文件名） |
| Path | `crates/schedulex` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Candidate | [SPEC-INFRA-SCHEDULEX-002](../../../draft/schedulex-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Implementation snapshot | `b0934baa`（2026-07-15） |
| Document commit | `e0b98df4` |
| Verified at | `e0b98df4`（相关实现路径未变化） |

## 1. 定位与依赖

本 crate 是 L1 任务 ID 登记表（**active SSOT registry**：单一任务 ID 真源），并附带确定性、进程内 `tick(now_ms)` 驱动的 `JobRunner` additive 面（最小 cron 子集）。**登记 ≠ 自动执行**：`Scheduler`（登记表）与 `JobRunner`（执行器）相互独立，无自动联动。明确边界：**非** 完整作业调度器 / 执行器——无 async runtime、无分布式 lease、无墙钟 daemon、无生产调度平台。当前 crate 为 std-only、无任何生产依赖；workspace 无 owner 外的生产消费者。

## 2. 当前公开 API

`Scheduler` 内部为 `HashMap<String, ()>`（任务 ID 登记表）：

| API | 当前行为 |
|---|---|
| `new/default` | 创建空登记表 |
| `schedule(id)` / `schedule_checked` / `schedule_normalized` / `try_schedule` / `schedule_many` | 登记任务 ID；重复 ID 幂等覆盖 |
| `cancel(id)` / `cancel_many` | 删除并返回此前是否存在 |
| `list()` / `contains` / `len` / `is_empty` | 查询登记表 |
| `intersection_ids` / `difference_ids` / `union_ids` / `retain` / `clear` | 集合运算 |

`additive 面`（与登记表独立，无自动联动）：

| 类型 | 当前行为 |
|---|---|
| `JobRunner` | `add(Job, Schedule)` + `tick(now_ms) -> TickResult`；确定性、进程内、无墙钟 |
| `Schedule` | `once` / `fixed_delay` / `cron`（最小子集；非完整 cron 方言） |
| `Job` / `JobFn` / `JobId` / `JobMeta` | 闭包式一次性 job 描述 |

“登记一个任务 ID”**不等于**定时触发或执行任务；`JobRunner::tick` 由宿主显式驱动。

## 3. 未实现能力

- 分布式调度 / 跨进程 lease / fencing；
- 墙钟后台 daemon（仅显式 `tick(now_ms)`，无自动触发）；
- async runtime / tokio 集成；
- 持久化恢复 / misfire 产品矩阵；
- 完整 cron 方言 / 时区产品；
- 把 `Scheduler`（登记表）与 `JobRunner`（执行器）混为一谈，或宣称 package stable / Agent L5。

候选单进程/分布式分级方案见 Candidate Draft；未批准前不属于 active API。

## 4. 当前测试

5 个测试覆盖：登记/list、取消、取消 missing、Default 空、重复登记幂等。

反例条件：owner 外出现 consumer，或源码/Cargo 出现 timer、Clock、Job/Run、runtime/persistence/shutdown 时，“仅登记”结论失效。

## 5. 验收

```bash
cargo test -p schedulex
cargo check -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

通过条件：API/依赖与源码一致；不把 registry 冒充 production scheduler。

## 6. 追溯

- `docs/architecture/spec.md` §4.4
- `crates/schedulex/{Cargo.toml,src/lib.rs}`
