# AGENTS — crates/resiliencx

- Package / lib：`resiliencx` · 当前版本：`0.1.2`
- 定位：L1 **安全重试 + budget + 退避/jitter/可注入 wait + 熔断 + 限流 + 舱壁**
- 依赖：`kernel` + `contracts` + `async-trait`；feature `tokio` 可选；**禁止**直接依赖 observex
- 依赖边界：禁止反向依赖 transport/domain/app；可观测性注入 `contracts::Instrumentation`
- 生产重试：必须显式 `RetrySafety`；generic Adapter 使用 `call_with_retry_budget_*_safe`
- 兼容入口：`call_with_retry_budget` / `call_with_retry_budget_async`、`retry_fn*`、`retry_async*`
  的未带 `safe` 变体均为 unchecked compatibility
- 熔断/限流/舱壁：无墙钟；本地立即拒绝，不宣称分布式协调或公平队列
- Active SSOT：`.agents/ssot/resiliencx/spec/spec.md`，须与
  `.agents/ssot/resiliencx/spec/xhyper-resiliencx-complete-spec.md` 保持 `cmp` 一致

## 验收命令

```bash
cargo fmt -p resiliencx -- --check
cargo test -p resiliencx --all-features --all-targets
cargo clippy -p resiliencx --all-features --all-targets -- -D warnings
cargo doc -p resiliencx --all-features --no-deps
node scripts/quality-gates/cov-gate-100.mjs -p resiliencx --filter crates/resiliencx/src
cmp .agents/ssot/resiliencx/spec/spec.md \
    .agents/ssot/resiliencx/spec/xhyper-resiliencx-complete-spec.md
```

覆盖率门禁使用共享 target；多 writer 并行时由 root 停止其他 writer 后串行执行。
