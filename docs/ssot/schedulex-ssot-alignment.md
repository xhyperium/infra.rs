# schedulex SSOT 本仓对齐

| 字段 | 值 |
|---|---|
| Baseline | `55433a2ec3567624c5cd98601b9f4581a7e69cb6` |
| Active SSOT | `.agents/ssot/schedulex/spec/spec.md` ≡ dual mirror |
| Crate | `crates/infra/schedulex` · package/lib `schedulex` · 当前 v0.1.2 |
| 范围 | Scheduler registry + 独立 explicit tick JobRunner |

## 合同矩阵

| 表面 | 本仓事实 | 状态 |
|---|---|---|
| std-only / default=[] | Cargo 无生产依赖 | PASS |
| Scheduler registry | 登记、取消、查询、集合运算 | PASS |
| Job/Schedule/JobRunner | `JobRunner::tick` 为进程内宿主驱动入口 | PASS |
| add 输入校验 | ID + Schedule 插入前校验 | PASS |
| add 失败原子性 | 无效替换保留原 callback/schedule/运行与取消状态 | PASS |
| deterministic | due 与 metadata 按 Job ID 排序 | PASS |
| 时间回退 | 忽略且不推进 | PASS |
| 大跨度 tick | 每 Job 最多一次，不补跑 | PASS |
| `every:<ms>` | stateful interval；首次 tick 执行，Err 推进 | PASS |
| Job Err | 有序记录、推进并继续 | PASS |
| panic | 传播，中止当前 tick | PASS |
| 中文错误 | cron/调度详情中文 | PASS |
| 后台/持久化/分布式调度 | 不实现 | NO-GO |
| 根 `AGENTS.md` 身份 | 已声明 registry + 独立显式 tick JobRunner | PASS |

## 误用红线

- `Scheduler::schedule` 不执行 Job。
- `JobRunner` 需要宿主主动 tick，不是 daemon。
- 测试与 coverage 不能证明业务 live、长稳、分布式能力或 package stable。
- 完整 cron、时区、misfire、恢复、lease/fencing 保持 OPEN/NO-GO。

## 验证

```bash
cmp .agents/ssot/schedulex/spec/spec.md \
  .agents/ssot/schedulex/spec/xhyper-schedulex-complete-spec.md
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p schedulex --filter crates/infra/schedulex/src
```

最终发布仍需前序 PR 合并后的版本/lock/STATUS 同步、全仓门禁、独立 review、PR CI 与人工审批。
