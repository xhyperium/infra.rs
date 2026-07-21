> **历史 monorepo 记录**（infra.rs）：文中 archgate / `.architecture` 不构成本仓验收条件；本仓不移植 archgate。

# EVID-KERNEL-002 — CI babysit @ `98af7c9c`

| 字段 | 值 |
|------|-----|
| Date | 2026-07-14 |
| Commit | `98af7c9c` — fix(kernel): drop from_std and harden Timestamp bounds tests |
| PR | [#235](https://github.com/xhyperium/infra.rs/pull/235) |
| §18 | **仍 OPEN**（Spec Proposed；禁止 stable） |

## 范围

HEAD 含 G2 全机器轨 + RES-CLK-009 / RES-TEST-004 收口：

- `MonotonicInstant::from_std` 删除
- FixedClock → `from_clock_elapsed`
- archgate KERNEL-* 13 规则 + CI `kernel-loom`
- Timestamp MIN/MAX proptest

## CI 终态（`gh pr checks 235`，无 pending / fail）

| Check | 结果 |
|-------|------|
| fmt | **pass** |
| lint-deps | **pass** |
| gate | **pass** |
| architecture-drift | **pass** |
| kernel-loom | **pass** |
| clippy | **pass** |
| machete | **pass** |
| codespell | **pass** |
| coverage | **pass** |
| kafkax-real | **pass** |
| test-stable | **pass** |
| docs-check | **pass** |
| deny / harness check / ssot-optional | **pass** |
| test-matrix / audit / docs-deploy | **skip**（按设计） |

**结论：代码路径 CI 绿 · 不得宣称 §18 闭合。**

## 本地复验（提交前）

```text
cargo test -p kernel --test clock_contract  # 9 passed
cargo test -p gate -p binance -p okx --lib
cargo run -p archgate -- --json             # 13 KERNEL-* ok
```

## 仍 OPEN residual

RES-API-007 · RES-TEST-005/014/015/016 · RES-DOWN-006 · Spec Approved / §18
