# resiliencx round-01 findings

> 历史版本说明：本轮执行者按范围保留 `0.1.1`；root 已在第 3 轮发布准备阶段完成 PATCH bump 至 `0.1.2`。

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 |
| Base | `3cd29a942710c0fb42f3f6bc05e3c31570acad47` |
| 范围 | `crates/resiliencx/**`、`.agents/ssot/resiliencx/**`、`docs/ssot/resiliencx-ssot-alignment.md` |
| 版本 | 保持 `0.1.1`；任务明确禁止版本变更 |

## 发现与处置

1. 既有 retry API 无法表达操作副作用语义，生产调用方可对不幂等写操作配置多次尝试。
   新增 `RetryContext`、`RetrySafety` 与 sync/async 安全入口；多次尝试在首次调用前拒绝 `UnsafeSideEffect`。
2. 既有 async retry 缺少预算 parity 与整次 deadline。
   新增 async budget 入口及 feature `tokio` 的整次 deadline 包装；预算耗尽与同步统一标准错误。
3. `call_with_retry_budget` 在第 N 次调用前记录 N，而其他 retry 入口记录刚失败的 N-1，且预算耗尽时返回前一瞬态错误。
   调整为预算成功消费后记录刚失败 attempt；耗尽不记录虚假 retry，并返回标准 budget 错误。
4. `BulkheadPermit::drop` 遇到 poisoned mutex 会静默跳过归还，造成永久容量泄漏。
   对内部状态锁统一恢复 poisoned inner 并 clear poison；故障注入测试证明槽位可复用。
5. attempt-only jitter 在相同配置实例间产生相同序列，容易同相。
   保留兼容结果，以 `RetryContext::with_jitter_seed` 接入实际安全重试，并新增纯计算入口；文档明确
   两者都不是加密 RNG，attempt-only 不抗群聚。
6. active spec 与实现严重漂移，仍把 async、budget、熔断、限流与舱壁列为未实现。
   重写 current-state spec、complete mirror、crate 文档和 alignment，恢复可核验的诚实边界。

## 关键合同

- `record_retry(op, attempt)` 中 attempt 是刚失败、即将触发下一次 retry 的序号，从 1 起。
- 预算令牌仅在真正准备 retry 时消费；消费失败不调用 operation、不记录 retry。
- async 退避前原子 reserve 预算、退避后 commit 并记录 retry；空预算立即标准错误，deadline 在退避期
  取消通过 RAII refund，不泄漏预算或虚假事件。
- deadline 覆盖所有尝试与 wait；超时只停止 future 后续轮询，不保证撤销已发生副作用。
- 熔断、限流与舱壁均为本地原语；限流/舱壁资源不足时立即拒绝，不排队。
- 低层兼容 retry API 继续可用，但不校验 `RetrySafety`。

## 测试新增

- 多次不安全副作用在 sync/async 安全入口首次调用前被拒绝；只读与单次不安全操作可执行。
- async budget 耗尽返回标准错误，且仅观测实际 retry。
- 整次 deadline 的成功与超时映射。
- bulkhead poisoned mutex 恢复后 permit 容量可复用。
- 调用方不同 seed 产生去相关 jitter。

## 残余风险

- `RetrySafety::Idempotent` 依赖调用方诚实声明，crate 无法验证领域幂等性。
- cooperative cancellation 不监管 operation 自行派生的后台任务。
- seeded jitter 的 seed 质量与唯一性由调用方负责。
- 本轮未增加 Retry-After、execution report、分布式预算或排队舱壁。
- package stable 仍未宣称。

## 验证记录

| 命令 | 退出码 | 结果 |
|------|--------|------|
| `cargo fmt -p resiliencx -- --check` | 0 | scoped Rust 格式通过 |
| `cargo test -p resiliencx --all-features --all-targets` | 0 | round-01 复审前：41 unit + 36 其他测试/target 通过 |
| `cargo clippy -p resiliencx --all-features --all-targets -- -D warnings` | 0 | 无 warning |
| `cmp .agents/ssot/resiliencx/spec/spec.md .agents/ssot/resiliencx/spec/xhyper-resiliencx-complete-spec.md` | 0 | 双镜像一致 |
| `git diff --check -- crates/resiliencx .agents/ssot/resiliencx docs/ssot/resiliencx-ssot-alignment.md` | 0 | scoped diff 无 whitespace 错误 |
| `cargo doc -p resiliencx --all-features --no-deps` | 0 | 公共 rustdoc 可生成 |

首次 `cargo fmt --all --check` 因同一共享 worktree 中其他 writer 的 bootstrap 未格式化 diff 返回 1；
本任务未修改、格式化或回退其路径。resiliencx scoped fmt 与最终定向门禁均通过。
