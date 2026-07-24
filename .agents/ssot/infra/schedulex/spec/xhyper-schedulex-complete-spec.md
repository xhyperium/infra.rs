# SPEC-SCHEDULEX-003

状态：APPROVED FOR MAINTENANCE IMPLEMENTATION；本轮候选尚未发布。

| 字段 | 值 |
|---|---|
| Baseline | `55433a2ec3567624c5cd98601b9f4581a7e69cb6` |
| Package / lib | `schedulex` / `schedulex` |
| Path / layer | `crates/infra/schedulex` / L1 |
| 依赖 | std-only，生产依赖必须为空 |
| Authority | 本文件与双镜像共同定义当前声明面 |

## 1. 模块定位

本 crate 包含两个互不自动联动的深模块：

- `Scheduler`：任务 ID registry seam；登记、查询、集合运算，不执行回调。
- `JobRunner`：宿主显式调用 `tick(now_ms)` 的进程内 deterministic seam。

本 crate 不是后台 timer、分布式 scheduler 或持久化作业平台。登记 ID 不等于执行 Job。
`JobRunner::tick` 是宿主驱动的显式入口；本 crate 不提供分布式调度。

## 2. Scheduler interface

- `schedule` 重复 ID 幂等覆盖；`cancel` 返回此前是否存在。
- `schedule_checked` / `schedule_normalized` 使用统一 ID 校验。
- `list` 与集合运算顺序不承诺；调用方不得依赖 HashMap 迭代顺序。
- registry 与 `JobRunner` 不同步、不共享生命周期。

## 3. JobRunner interface

| 行为 | 合同 |
|---|---|
| `add` | 插入前校验 `JobId` 与 `Schedule`；失败不改变 runner |
| 重复 ID | 新 Job、Schedule 与全部运行状态完整替换旧条目 |
| `cancel` | 标记取消；条目存在即返回 `true`，重复取消仍为 `true`，直至 `remove` |
| `remove` | 删除条目并释放 Job；返回此前是否存在 |
| `list_meta` | 包含已取消未移除条目，按 Rust `str::cmp` 的 Job ID 字典序返回 |
| `tick` | 同一 tick 的到期 Job 按 Rust `str::cmp` 的 Job ID 字典序执行 |
| 错误 | Job `Err` 按执行顺序进入 `TickResult.errors`，推进触发状态并继续后续 Job |
| panic | 不捕获；panic 传播并中止当前 tick，部分执行状态不保证 |

`JobId::new` / `From` 可构造未校验值，但任何值进入 runner 前必须由 `add` 统一校验，不能绕过空值、长度和控制字符规则。

## 4. Schedule 与时间语义

- `Once { at_ms }`：首次 `now_ms >= at_ms` 时执行一次。
- `FixedDelay`：`every_ms > 0`；首次到期后按上次实际执行时间计算；大跨度 tick 只执行一次，不补跑。
- `Cron every:<ms>`：stateful interval；首次非回退 tick 立即执行，之后距上次执行达到 `every_ms` 时执行；大跨度只执行一次。Job `Err` 也推进 interval 基准。
- 五段 Cron：仅分钟子集，按逻辑分钟 epoch 对齐；公开 `expr` 必须可重新解析且结果与 `parsed` 相等。
- `cron_matches()` 是公开的无状态 epoch predicate；`JobRunner` 的 `every:<ms>` interval 不使用它，五段 MinuteMatch 行为不变。
- `now_ms` 是逻辑毫秒且应非递减；小于上次 tick 的输入 fail-closed：不执行、不推进。
- 同一逻辑分钟内 Cron MinuteMatch 最多执行一次。
- 每个 Job 在每次 `tick` 调用中最多执行一次。

调度解析失败详情必须为简体中文；协议字面量（如 `every:<ms>`）和标识符可保留英文。

## 5. 明确非目标 / NO-GO

- 真实墙钟、后台线程、daemon、`Clock`、tokio 或 async runtime；
- 持久化恢复、misfire 补跑矩阵、timeout/cancellation 产品；
- 跨进程 lease、fencing、leader election、分布式调度；
- 完整 cron 方言、时区和日历产品；
- package stable、Production Ready/L5 或业务 live 声明。

## 6. 验收与证据

外部测试只通过公开 interface 验证：非法 ID/调度 fail-closed、稳定排序、错误继续、时间回退、无补跑、替换、取消、Cron 与 panic 策略。最终必须通过：

```bash
cmp .agents/ssot/infra/schedulex/spec/spec.md \
  .agents/ssot/infra/schedulex/spec/xhyper-schedulex-complete-spec.md
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo fmt --all --check
node scripts/quality-gates/cov-gate-100.mjs -p schedulex --filter crates/infra/schedulex/src
node scripts/quality-gates/check-workspace-deps.mjs
```

本地通过不能替代 PR CI、独立 reviewer、版本同步或人工审批。
