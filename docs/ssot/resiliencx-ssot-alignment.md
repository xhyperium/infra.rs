# resiliencx SSOT 对齐（infra.rs）

| 字段 | 值 |
|------|-----|
| 日期 | 2026-07-23 · round-03 候选准备 |
| Active SSOT | `.agents/ssot/infra/resiliencx/spec/spec.md` |
| Complete mirror | `.agents/ssot/infra/resiliencx/spec/xhyper-resiliencx-complete-spec.md`（须 `cmp`） |
| 实现 | `crates/infra/resiliencx` · package/lib `resiliencx` |
| 当前版本 | 0.1.2；同一 PR 已 bump 一次，按 R-C2 不重复 bump |
| 状态 | L1 进程内弹性原语；Internal Ready，非 package stable |

## round-01 需求对齐

| 需求 | 状态 | 实现与证据 |
|------|------|------------|
| 显式 RetrySafety | **PASS** | `RetryContext` + `RetrySafety` + safe sync/async 入口；多次尝试拒绝 `UnsafeSideEffect` |
| 保留低层兼容 API | **PASS** | 旧 sync/async retry API 保留；rustdoc/README 明确其不做 safety 校验 |
| 整次 async deadline | **PASS** | feature `tokio` 的 `retry_async_with_deadline`；覆盖 operation + retry + wait |
| deadline 错误映射 | **PASS** | timeout → `XError::deadline_exceeded`；测试成功与超时路径 |
| cancellation 边界 | **PASS** | SSOT/README/API 明确 cooperative cancellation 不撤销已发生副作用 |
| async budget parity | **PASS** | `retry_async_with_budget` / `retry_async_safe` 可消费预算；耗尽统一标准错误 |
| attempt 观测一致性 | **PASS** | `call_with_retry_budget` 与其他 retry 均记录刚失败 attempt，从 1 起 |
| Bulkhead poison 容量恢复 | **PASS** | poisoned inner 恢复并 clear poison；permit drop 后槽位可复用测试 |
| 调用方 seed jitter | **PASS** | `RetryContext::with_jitter_seed` 接入实际 safe sync/async/deadline；另有纯计算 helper |
| attempt-only jitter 边界 | **PASS** | 明确不具备抗群聚保证 |
| active/complete spec | **PASS** | 两份 current-state spec 同构；以 `cmp` 为门禁 |
| findings | **PASS** | `.agents/ssot/infra/resiliencx/plan/round-01-findings.md` |

## 当前能力

| 能力 | 状态 | 诚实边界 |
|------|------|----------|
| 有限同步/异步重试 | PASS | 仅 retryable 错误；生产新接线使用显式 safety 入口 |
| 重试预算 | PASS | 本地共享令牌；不是动态或分布式预算 |
| 退避/jitter | PASS | seed 由调用方管理；非加密 RNG |
| 熔断 | PASS | 本地无墙钟；按拒绝次数推进 HalfOpen |
| 限流 | PASS | 本地令牌桶；显式 refill；不足立即拒绝 |
| 舱壁 | PASS | 本地并发计数；满载立即拒绝；无队列/等待 deadline |
| async deadline | PASS（feature `tokio`） | cooperative cancellation，不撤销外部副作用 |
| Package stability | **未宣称** | 无 crates.io / stable API 承诺 |

## Round 3 Adapter safety 候选

| 项 | 当前事实 | 裁决 |
|---|---|---|
| Generic Adapter budget | 新增 sync/async `_safe` 入口，首次 operation 前校验 `RetrySafety` | 本地 reviewer 审查完成；verifier 技术/证据初验完成 |
| Legacy API | `call_with_retry_budget`、未带 safe 的 retry 与 adapter wrapper 均为 unchecked compatibility | 本地 reviewer 审查完成 |
| Redis client | GET/EXISTS/PTTL/MGET 为 ReadOnly；无 TTL SET/MSET 为 Idempotent；相对 TTL SET/DEL/PEXPIRE 为 UnsafeSideEffect；PUBLISH 不自动重试 | 实现完成；live I/O 未在本地执行 |
| Postgres pool | 当前没有 budget 字段或自动接线；只提供显式 safety wrapper | 诚实更正；不虚构生产接线 |

治理修正后候选已重冻；本地 reviewer 已完成实现/证据审查，独立 verifier 已完成技术/证据初验；
本次纯状态 delta 不改变受审源码/测试。GitHub CI/交付与发布动作仍 pending，release 继续 BLOCKED。

## 生产误用红线

- `RetrySafety` 是调用方声明，不是闭包幂等性的静态证明。
- 低层兼容 retry API 不执行 safety 校验，不能作为新生产接线的默认入口。
- async deadline 不能替代幂等键、事务或补偿；已经发生的外部副作用不会自动撤销。
- attempt-only jitter 会在相同配置实例间同相，不具备抗群聚保证。
- 本地 `try_acquire` / `try_enter` 是立即拒绝原语，不提供公平排队、跨进程配额或自动墙钟补充。
- STATUS/结构完成度不等于 Production Ready 或 package stable。

## 验收

```bash
cargo fmt --all --check
cargo test -p resiliencx --all-features --all-targets
cargo clippy -p resiliencx --all-features --all-targets -- -D warnings
cmp .agents/ssot/infra/resiliencx/spec/spec.md \
    .agents/ssot/infra/resiliencx/spec/xhyper-resiliencx-complete-spec.md
```

详细发现、选择与残余风险：`.agents/ssot/infra/resiliencx/plan/round-01-findings.md`、
`.agents/ssot/infra/resiliencx/plan/round-02-findings.md` 与 `.agents/ssot/infra/resiliencx/plan/round-03-findings.md`。

第 2 轮独立代码/规格复审通过；此前 root 串行覆盖率 994/994 是本轮 Adapter safety 补丁前基线。
首次当前树 coverage 为 1106/1116（99.1039%）；缺失行为测试补齐后，root 串行复验为
1156/1156、zeros 0、100.0000%、退出码 0。治理修正后候选已重冻，本地 reviewer 完成、verifier
技术/证据初验完成；GitHub 固定提交 CI artifact、PR/审批/合并仍 pending，因此本文件不构成发布批准。

最新固定 review 又修改了 unchecked generic async budget core 与 Redis 零 attempts 路由，因此
1156/1156 是本次修复前基线；root 最终串行重跑为 1208/1208、zeros 0、100.0000%、退出码 0。
三包最终测试为 resiliencx 84 passed、postgresx 52 passed + 6 ignored、redisx 51 passed + 8 ignored；
postgresx/redisx 当前版本均为 0.3.4，0.3.3 保留为 main 历史。
