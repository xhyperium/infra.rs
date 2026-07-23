# adapters/storage/redis — Goal 管线契约

> 实现 / 代码唯一位置：`crates/adapters/storage/redis`  
> **当前 SSOT Spec**：[spec/spec.md](spec/spec.md) ≡ [spec/xhyper-redisx-complete-spec.md](spec/xhyper-redisx-complete-spec.md)  
> **Source Goal**：见 [goal/goal.md](goal/goal.md) — **未宣称闭合**（无证据不得标 Done）  
> **布局**：对齐 [`.agents/ssot/kernel/`](../../../kernel)（[AGENTS.md](../../../../../AGENTS.md) §2）  
> **状态**：`redisx 0.3.4` 当前合同已对齐；候选曾冻结，治理修正后最终 SHA / reviewer / verifier / CI pending

## infra.rs 本仓落地（#188–#191）

- 落地说明：[plan/infra-rs-landing.md](plan/infra-rs-landing.md)
- draft SPEC_GOAL 快照：[plan/infra-rs-draft-spec-goal.md](plan/infra-rs-draft-spec-goal.md)
- 对齐：[docs/ssot/adapters-ssot-alignment.md](../../../../../docs/ssot/adapters-ssot-alignment.md)
- **状态**：生产默认客户端 **P0 已落地**；package stable **未宣称**
- **拓扑证据**：Cluster / Sentinel / TLS 代码路径存在，真实 live 均 **OPEN**
- **Pub/Sub**：Standalone only；同源 ACL/TLS/deadline，其他拓扑失败关闭
- **重试**：配置 budget 后，只读与固定输入幂等写安全重试；相对 TTL SET、DEL、PEXPIRE 多次尝试
  在 I/O 前拒绝；PUBLISH 不自动重试

> Code 阶段的当前状态以 `spec/`、`matrix/` 与本仓 alignment 为准。既有
> `review/`、`release/`、`retrospective/` 是早期阶段制品，本 Task 不改写，也不得用其旧快照
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
| **Code** | **`crates/adapters/storage/redis`** | 实现不在 `.agents/ssot/` |
| Test | [test/test.md](test/test.md) | 51 passed + 8 ignored；最终 SHA CI pending |
| Review | [review/review.md](review/review.md) | 最终重冻树复审 pending |
| Release | [release/release.md](release/release.md) | `0.3.4` 候选；BLOCKED |
| Retrospective | [retrospective/retrospective.md](retrospective/retrospective.md) | 入口 / 占位 |

## 横切

| 制品 | 路径 |
|------|------|
| Matrix | [matrix/matrix.md](matrix/matrix.md) |
| Gate | [gate/gate.md](gate/gate.md) |
| Evidence | [evidence/](evidence/) |

## 硬限制

1. 无证据不得宣称 Done / 全闭合 / 5/5 / Spec Approved（除非既有战役文件已证明）。
2. 本树禁止 `src/`、`Cargo.toml`、`*.rs` 实现副本（C-LINT-007）。
3. 布局迁移 **≠** 实现完成 **≠** package stable。
4. 双镜像：`spec/spec.md` 与 `spec/xhyper-redisx-complete-spec.md` 须 `cmp` 同构。

## 验证

```bash
cmp .agents/ssot/adapters/storage/redis/spec/spec.md \
    .agents/ssot/adapters/storage/redis/spec/xhyper-redisx-complete-spec.md
# 结构：README + 11 层目录 + evidence/ 横切
test -f .agents/ssot/adapters/storage/redis/README.md
test -f .agents/ssot/adapters/storage/redis/spec/spec.md
```

**布局对齐：是 · 战役全闭合：未宣称 · 禁止假 Done。**
