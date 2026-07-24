# DESIGN-SCHEDULEX-003

状态：APPROVED FOR IMPLEMENTATION

## 深模块与 seam

- `Scheduler` 保持 registry interface；无自动执行。
- `JobRunner` 是唯一运行 seam：调用方只需学习 add/cancel/remove/list_meta/tick，校验、排序、到期判断与状态推进均藏在实现内部。
- 不新增 Clock、executor adapter 或 runtime seam；当前只有进程内实现，引入抽象会成为假 seam。

## 决策

1. `add` 先校验 JobId，再校验 Schedule，最后原子覆盖条目。
2. Cron 公开 `expr` 必须重新解析并与 `parsed` 相等，拒绝伪造组合。
3. due ID 和 metadata 按 Rust `str::cmp` 排序，不依赖 HashMap。
4. runner 记录上次 tick；时间回退返回空结果且不推进。
5. Job Err 推进状态并继续；panic 不捕获。
6. FixedDelay 大跨度只执行一次；不实现补跑。
7. cancel 返回“条目存在”，重复取消仍 true，remove 才删除。
8. `every:<ms>` 是 stateful interval：首次 tick 立即执行，后续按上次执行时刻计时；不使用无状态 `cron_matches()` epoch predicate。

回滚条件：breaking API、越界依赖/路径、非目标能力或门禁三次最小修复后仍失败。
