# adapters/storage/redis — Goal 管线契约

> 实现 / 代码唯一位置：`crates/adapters/storage/redis`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-redisx-complete-spec.md](spec/xhyper-redisx-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — **未宣称闭合**（无证据不得标 Done）  
> **布局**：对齐 [`.agents/ssot/kernel/`](../../../kernel)（[AGENTS.md](../../../../../AGENTS.md) §2）  
> **状态**：`redisx **0.3.13**` Standalone P0+ + selfcheck §6.5 + live/E2E；全公开 API 已落地；package stable **未宣称**

## infra.rs 本仓落地

- 对齐文档（权威）：[docs/ssot/redisx-ssot-alignment.md](../../../../../docs/ssot/redisx-ssot-alignment.md)
- adapters 汇总：[docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md)
- draft SPEC_GOAL 快照：[plan/infra-rs-draft-spec-goal.md](plan/infra-rs-draft-spec-goal.md)
- 落地说明：[plan/infra-rs-landing.md](plan/infra-rs-landing.md)
- **状态**：生产默认客户端 **P0+ 已落地**（deadline / pipeline / 锁 / metrics / selfcheck / live 矩阵）
- **拓扑证据**：Cluster / Sentinel / TLS 代码路径存在，真实 live 均 **OPEN**
- **Pub/Sub**：Standalone only；同源 ACL/TLS/deadline；其他拓扑失败关闭；重连/必达 **NO-GO**
- **selfcheck**：`redisx::selfcheck` · LIB-SELFCHECK §6.5 · 11 检查项；非 `tools/verifyctl`
- **重试**：配置 budget 后，只读与固定输入幂等写安全重试；相对 TTL SET、DEL、PEXPIRE 多次尝试在 I/O 前拒绝；PUBLISH 不自动重试

> Code 阶段的当前状态以 `spec/`、`matrix/` 与本仓 alignment 为准。既有
> `review/`、`release/`、`retrospective/` 是早期阶段制品，不得用其旧快照
> 覆盖当前 OPEN/NO-GO 裁定。

## 11 层映射

| 管线层 | 路径 | 状态 |
|--------|------|------|
| Goal | [goal/goal.md](goal/goal.md) | 入口存在 · 未宣称 AC 闭合 |
| Spec | [spec/spec.md](spec/spec.md) | **SSOT 入口**（布局迁移） |
| Design | [design/design.md](design/design.md) | 当前拓扑 / retry safety 设计 |
| Plan | [plan/plan.md](plan/plan.md) | 入口；战役文件可并列于 plan/ |
| Tasks | [tasks/tasks.md](tasks/tasks.md) | 入口 / 占位 |
| Prompt | [prompt/prompt.md](prompt/prompt.md) | 入口 / 占位 |
| **Code** | **`crates/adapters/storage/redis`** | 实现不在 `.agents/ssot/` · **0.3.13** |
| Test | [test/test.md](test/test.md) | 离线 lib 绿；live/E2E `#[ignore]`；见 alignment |
| Review | [review/review.md](review/review.md) | 历史制品；当前以 PR #281–#306 为准 |
| Release | [release/release.md](release/release.md) | 历史候选；当前 crate version **0.3.13** |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 入口 / 占位 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/](evidence/) · 尤其 `evidence/2026-07-23/` |

## 硬限制

1. 无证据不得宣称 Done / 全闭合 / package stable / 行覆盖 100%。
2. 本树禁止 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
3. 布局迁移 **≠** 实现完成 **≠** package stable。
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-redisx-complete-spec.md` 须 `cmp` 同构。

## 验证

```bash
cmp .agents/ssot/adapters/storage/redis/spec/spec.md \
    .agents/ssot/adapters/storage/redis/spec/xhyper-redisx-complete-spec.md

cargo test -p redisx --lib --features pubsub
scripts/live/export-foundationx-env.sh --env dev -- \
  cargo test -p redisx --features pubsub \
    --test integration_all_api --test e2e_klines_crud -- --ignored --test-threads=1
```
