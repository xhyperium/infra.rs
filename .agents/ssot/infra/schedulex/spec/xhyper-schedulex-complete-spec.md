# `schedulex` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 最小登记合同；无真实定时器 |
| Package / lib | `xhyper-schedulex` / `schedulex` |
| Path | `crates/infra/schedulex` |
| Layer | L1 Infra |
| Authority | 本文件是 active current-state spec |
| Candidate | [SPEC-INFRA-SCHEDULEX-002](../../../../draft/xhyper-schedulex-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Implementation snapshot | `b0934baa`（2026-07-15） |
| Document commit | `e0b98df4` |
| Verified at | `e0b98df4`（相关实现路径未变化） |

## 1. 定位与依赖

长期职责是定时/异步任务调度；当前 crate 为 std-only、无任何依赖，workspace 没有 owner 外的生产消费者。

## 2. 当前公开 API

`Scheduler` 内部为 `HashMap<String, ()>`：

| API | 当前行为 |
|---|---|
| `new/default` | 创建空登记表 |
| `schedule(id)` | 插入任务 ID；重复 ID 幂等覆盖 |
| `cancel(id)` | 删除并返回此前是否存在 |
| `list()` | 返回当前所有 ID；顺序未承诺 |

“登记一个任务 ID”不等于定时触发或执行任务。

## 3. 未实现能力

- Clock、timer、async runtime、Job/Run；
- Once/FixedDelay/FixedRate/cron；
- misfire、并发、timeout/cancellation、shutdown；
- 持久化恢复、lease/fencing、分布式调度。

候选单进程/分布式分级方案见 Candidate Draft；未批准前不属于 active API。

## 4. 当前测试

5 个测试覆盖：登记/list、取消、取消 missing、Default 空、重复登记幂等。

反例条件：owner 外出现 consumer，或源码/Cargo 出现 timer、Clock、Job/Run、runtime/persistence/shutdown 时，“仅登记”结论失效。

## 5. 验收

```bash
cargo test -p xhyper-schedulex
cargo check -p xhyper-schedulex --all-targets
cargo clippy -p xhyper-schedulex --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

通过条件：API/依赖与源码一致；不把 registry 冒充 production scheduler。

## 6. 追溯

- `docs/architecture/spec.md` §4.4
- `crates/infra/schedulex/{Cargo.toml,src/lib.rs}`
