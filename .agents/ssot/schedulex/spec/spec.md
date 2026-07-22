# schedulex 当前实现规范

状态：`schedulex` `0.1.1` active current-state 合同；任务 ID 登记 + 宿主驱动的确定性 `JobRunner::tick` 已实现，**非 runtime 或分布式调度产品**。

## 1. 权威与定位

- Package / lib / path：`schedulex` / `schedulex` / `crates/schedulex`。
- std-only，无生产依赖；`default = []`；`publish = false`。
- `Scheduler` 和 `JobRunner` 是两个独立表面；登记任务 ID 不会自动创建或运行 Job。

## 2. 可观察实现

| 表面 | 当前行为 |
|------|----------|
| `Scheduler` | 任务 ID 登记、校验/规范化、批量操作、cancel、查询与集合运算 |
| `Job` / `JobId` / `JobMeta` | 进程内闭包 Job 描述与元数据 |
| `Schedule::Once` | 在显式时间达到阈值后执行一次 |
| `Schedule::FixedDelay` | 按显式毫秒输入推进下一次执行 |
| `Schedule::Cron` | 受限 cron 子集；不是完整 cron 方言 |
| `JobRunner` | add/cancel/remove/query；`tick(now_ms)` 同步执行到期 Job 并返回 `TickResult` |

`JobRunner::tick` 的时间由宿主驱动。crate 不读取隐式墙钟，不创建线程、timer、daemon 或 async task。

## 3. 行为与错误边界

1. `Scheduler::schedule` 对重复 ID 幂等覆盖；checked/normalized API 拒绝非法 ID。
2. `JobRunner::add` 拒绝重复 Job ID；cancel 停用，remove 删除。
3. `tick(now_ms)` 只处理调用时已到期的 active Job；执行错误记录在结果中，不升级为调度平台级重试保证。
4. Once/FixedDelay/cron 子集的语义只对当前进程、当前 runner 实例和宿主提供的时间成立。

## 4. OPEN 与禁止声明

以下不在当前实现/证据范围：

- async runtime/tokio 集成、后台墙钟 daemon 与自动触发；
- 分布式调度、跨进程 lease/fencing、leader election；
- 持久化恢复、misfire 产品矩阵、重试编排、可观测控制面；
- 完整 cron 方言、时区/DST 产品合同；
- package stable、Production Ready 或 Agent L5。

不得把 `Scheduler` 写成执行器，也不得把宿主驱动的同步 `JobRunner::tick` 写成生产分布式 scheduler。

## 5. 验证与验收

```bash
cmp .agents/ssot/schedulex/spec/spec.md \
  .agents/ssot/schedulex/spec/xhyper-schedulex-complete-spec.md
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo fmt --all --check
```

通过条件：登记面与 runner additive 面均不被写窄；runtime/分布式能力仍明确 OPEN。
