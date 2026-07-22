# schedulex SSOT 本仓对齐

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21；**defer-close 复核 2026-07-22** |
| Active SSOT | `.agents/ssot/schedulex/spec/spec.md` ≡ `spec/xhyper-schedulex-complete-spec.md` |
| 本仓 crate | `crates/schedulex` · package `schedulex` · lib `schedulex` |
| 合同范围 | **内存任务 ID 登记表** + **进程内 tick 驱动 JobRunner**（**≠** 分布式调度器） |

## 合同矩阵

| 表面 | Active SSOT / 扩展 | 本仓 | 状态 |
|------|---------------------|------|------|
| Package / lib / path | `schedulex` / `schedulex` / `crates/schedulex` | 同左 | ✅ |
| 依赖 | 登记表 std-only | 登记表无生产 dep | ✅ |
| `new` / `Default` | 空登记表 | `Scheduler::{new, default}` | ✅ |
| `schedule(id)` | 插入；重复 ID 幂等覆盖 | `HashMap::insert` | ✅ |
| `cancel(id)` | 删除并返回此前是否存在 | `remove(...).is_some()` | ✅ |
| `list()` | 当前 ID；顺序未承诺 | `keys().cloned().collect()` | ✅ |
| Job / Schedule / JobRunner | 审查 OBJECTIVE 关闭项 | `job.rs` / `schedule.rs` / `runner.rs` | **PASS** |
| `JobRunner::tick(now_ms)` | 确定性 tick | 显式时间输入；无墙钟 daemon | **PASS** |
| Once / FixedDelay / cron 子集 | tick 驱动 | `Schedule::{once,fixed_delay,cron}` | **PASS**（**≠** 完整 cron 方言 / 分布式 lease） |
| async runtime / 持久化 / 分布式 | 非目标 | 无 | 诚实边界 OPEN |

## OBJECTIVE 处置（2026-07-22 defer-close）

| 项 | 前状态 | 现状态 | 证据 |
|----|--------|--------|------|
| timer / cron / Job 执行 | DEFER | **PASS（进程内 tick）** | `crates/schedulex/src/{job,schedule,runner}.rs` · `JobRunner::tick` |

## 明确非目标 / 诚实边界

- **分布式调度**、跨进程 lease、持久化恢复、misfire 产品矩阵
- 后台墙钟线程自动触发（本仓为 **显式 `tick(now_ms)`**）
- 完整 cron 方言 / 时区产品
- 把 `Scheduler`（登记表）与 `JobRunner`（执行器）混为一谈的误读

## 生产误用红线

| 允许 | 禁止 |
|------|------|
| 进程内任务 **ID 登记** / 幂等 cancel | 把 `Scheduler` 当分布式 cron |
| `JobRunner::tick` 由宿主驱动 | 依赖隐式墙钟 daemon 即生产调度平台 |
| 单测与组合根登记 | 宣称 package stable / Agent L5 |

见 crate README 与 [prod-consume-surface.md](../plans/artifacts/prod-consume-surface.md)。

## 验证

```bash
cmp .agents/ssot/schedulex/spec/spec.md \
    .agents/ssot/schedulex/spec/xhyper-schedulex-complete-spec.md
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo fmt --all --check
cargo llvm-cov -p schedulex --fail-under-lines 100 --summary-only
```

## 追溯

- Active SSOT：`.agents/ssot/schedulex/spec/spec.md`
- 实现：`crates/schedulex/`
- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)

## 双栏落地（2026-07-22 · STATUS 100% structure）

| 标尺 | 状态 |
|------|------|
| STATUS 结构完成度 | **100%**（layout+tests+content；非 Production Ready） |
| 声明面生产硬化 | 公共 API 集成测 + 热路径 bench + `docs/` 红线；**cov-gate-100 行覆盖** |
| 非宣称 | **禁止** workspace Production Ready / Agent L5 / 分布式调度产品 |

自验证：`cargo test -p schedulex --all-targets`；`node scripts/quality-gates/cov-gate-100.mjs -p schedulex`。

## 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-22 | **defer-close**：Job/Schedule/JobRunner tick 面 PASS；分布式仍 OPEN |
