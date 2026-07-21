# schedulex SSOT 本仓对齐

| 字段 | 值 |
|------|-----|
| 审计日期 | 2026-07-21 |
| Active SSOT | `.agents/ssot/schedulex/spec/spec.md` ≡ `spec/schedulex-complete-spec.md` |
| 本仓 crate | `crates/schedulex` · package `xhyper-schedulex` · lib `schedulex` · `0.1.0` |
| 合同范围 | **内存任务 ID 登记表**（非 production timer scheduler） |

## 合同矩阵

| 表面 | Active SSOT | 本仓 | 状态 |
|------|-------------|------|------|
| Package / lib / path | `xhyper-schedulex` / `schedulex` / `crates/schedulex` | 同左 | ✅ |
| 依赖 | std-only，无生产依赖 | `[dependencies]` 空 | ✅ |
| `new` / `Default` | 空登记表 | `Scheduler::{new, default}` | ✅ |
| `schedule(id)` | 插入；重复 ID 幂等覆盖 | `HashMap::insert` | ✅ |
| `cancel(id)` | 删除并返回此前是否存在 | `remove(...).is_some()` | ✅ |
| `list()` | 当前 ID；顺序未承诺 | `keys().cloned().collect()` | ✅ |
| Clock / timer / Job·Run | **禁止**（§3） | 生产源码无此类类型/依赖 | ✅ |
| async runtime / 持久化 / shutdown | **禁止**（§3） | 无 | ✅ |
| 测试五条 | schedule+list / cancel / cancel missing / Default 空 / 重复幂等 | `src/lib.rs` unit + `tests/public_api.rs` | ✅ |
| 覆盖率 | 目标 100% lines（本仓 goal） | `cargo llvm-cov -p schedulex --fail-under-lines 100` | ✅（见 evidence） |

## 明确非目标

Once / FixedDelay / FixedRate / cron / misfire / 并发 lease / timeout token /
graceful shutdown / 持久化恢复 / 分布式调度 —— active §3；Candidate Draft 非权威。

## 生产误用红线（infra-s9t.8）

| 允许 | 禁止 |
|------|------|
| 进程内任务 **ID 登记** / 幂等 cancel | 把 `Scheduler` 当 timer / cron / 调度执行器 |
| 单测与组合根登记表 | 依赖墙钟触发、持久化恢复、分布式 lease |

见 crate README 与 [prod-consume-surface.md](../plans/artifacts/prod-consume-surface.md)。

## 验证

```bash
cmp .agents/ssot/schedulex/spec/spec.md \
    .agents/ssot/schedulex/spec/schedulex-complete-spec.md
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
cargo fmt --all --check
cargo llvm-cov -p schedulex --fail-under-lines 100 --summary-only
```

## 追溯

- Active SSOT：`.agents/ssot/schedulex/spec/spec.md`
- 实现：`crates/schedulex/`
- 总览：[workspace-ssot-alignment.md](./workspace-ssot-alignment.md)
