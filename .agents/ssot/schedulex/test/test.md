# TEST-SCHEDULEX-003

状态：SCOPED PUBLIC-SEAM TESTS PASS；FINAL GATES PENDING

## 策略

- 外部集成测试仅通过 crate 根公开 interface。
- Red 先证明旧实现违反合同；Green 只加入最小实现。
- 排序测试重复构造多组 HashMap，避免偶然顺序形成假绿。
- 失败替换测试从已有同 ID 条目出发，固定 callback、schedule、运行状态与取消状态均不变。
- `every:<ms>` 覆盖 off-grid 首次 tick、跨度跳过、Err 推进与时间回退，不把它误当 epoch predicate。
- 不使用真实时间、sleep、网络、线程或持久化。

## 必跑

```bash
cargo test -p schedulex --test job_runner_public
cargo test -p schedulex --all-targets
cargo clippy -p schedulex --all-targets -- -D warnings
node scripts/quality-gates/cov-gate-100.mjs -p schedulex --filter crates/schedulex/src
```

panic 测试只证明 panic 传播，不把 unwind 当作恢复保证。

2026-07-23 本地 scoped：30 unit + 16 `job_runner_public` + 4 既有 public API = 50 tests，全部通过；LCOV 768/768。最终结果仍需在前序 rebase、版本与 contract-testkit 变更后重跑。
