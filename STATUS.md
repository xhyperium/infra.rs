# crates 子模块进度看板（自动生成）

> **生成方式**：`node scripts/docs/gen-crate-status.mjs`
> **生成时间**：2026-07-21T17:48:49Z
> **源权威**：根 `Cargo.toml` `[workspace.members]` + 各 crate 目录树
> **勿手改**：本文件由脚本覆盖。标准布局定义见 [crates/AGENTS.md](crates/AGENTS.md)；对齐叙事见 [docs/ssot/](docs/ssot/)。
> **口径声明**：完成度是**结构/可观测进度**（布局·测试·源码实质），**不是** Production Ready 签字，也不是 SSOT 镜像 COMPLETE。

## 总览

| 指标 | 值 |
|------|-----|
| workspace members | **24** |
| 布局七项齐全 | **22** / 24（92%） |
| 含测试（单元或集成） | **24** / 24（100%） |
| scaffold 信号 | **9** |
| **平均完成度** | **94%** █████████░ |

### 成熟度分布

| 标签 | 含义 | 数量 |
|------|------|------|
| `layout-incomplete` | 标准七项缺项 | 2 |
| `scaffold` | adapter/显式 scaffold 骨架 | 0 |
| `scaffold+mock` | scaffold 且具备 mock/测试入口 | 9 |
| `thin` | 布局齐但实质偏薄 | 0 |
| `partial` | 有测试 + 一定源码量 | 0 |
| `active` | 布局齐 + 测试 + 较厚实现 | 13 |

## 完成度公式

```text
completion = layout(7项)×50% + has_tests×25% + content×25%
content    = LOC 桶 + 可运行 example + docs/README 实质
scaffold   → content 上限 0.55（避免把内存桩当成生产实现）
```

## 成员明细

| Package | 路径 | 层 | 布局 | 测试 | LOC | 示例 | 成熟度 | 完成度 | SSOT |
|---------|------|----|:----:|:----:|----:|:----:|--------|--------|------|
| `kernel` | `crates/kernel` | L0 | 7/7 | ✅ 6i+u | 1702 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/kernel-ssot-alignment.md) |
| `testkit` | `crates/testkit` | T0 | 7/7 | ✅ 6i+u | 577 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/testkit-ssot-alignment.md) |
| `canonical` | `crates/types/canonical` | types | 7/7 | ✅ 2i+u | 1181 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/types-ssot-alignment.md) |
| `decimalx` | `crates/types/decimal` | types | 7/7 | ✅ 6i+u | 1476 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/types-ssot-alignment.md) |
| `bootstrap` | `crates/bootstrap` | L1 | 7/7 | ✅ 2i+u | 1027 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/bootstrap-ssot-alignment.md) |
| `configx` | `crates/configx` | L1 | 7/7 | ✅ 2i+u | 546 | 2 | `active` | **100%** ████████ | [✓](docs/ssot/configx-ssot-alignment.md) |
| `evidence` | `crates/evidence` | L1 | 7/7 | ✅ 2i+u | 536 | 2 | `active` | **100%** ████████ | [✓](docs/ssot/evidence-ssot-alignment.md) |
| `observex` | `crates/observex` | L1 | 7/7 | ✅ 2i+u | 613 | 2 | `active` | **100%** ████████ | [✓](docs/ssot/observex-ssot-alignment.md) |
| `resiliencx` | `crates/resiliencx` | L1 | 7/7 | ✅ 4i+u | 1011 | 2 | `active` | **100%** ████████ | [✓](docs/ssot/resiliencx-ssot-alignment.md) |
| `schedulex` | `crates/schedulex` | L1 | 7/7 | ✅ 2i+u | 550 | 2 | `active` | **100%** ████████ | [✓](docs/ssot/schedulex-ssot-alignment.md) |
| `contract-testkit` | `crates/test-support/contracts` | L1 | 7/7 | ✅ 1i | 1022 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/testkit-ssot-alignment.md) |
| `transportx` | `crates/transport` | L1 | 7/7 | ✅ 5i | 620 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/transport-ssot-alignment.md) |
| `goalctl` | `tools/goalctl` | L1 | 4/7 | ✅ 1i+u | 527 | · | `layout-incomplete` | **76%** ██████░░ | — |
| `verifyctl` | `tools/verifyctl` | L1 | 3/7 | ✅ 1i+u | 597 | · | `layout-incomplete` | **68%** █████░░░ | — |
| `contracts` | `crates/contracts` | contracts | 7/7 | ✅ 4i+u | 616 | 1 | `active` | **100%** ████████ | [✓](docs/ssot/contracts-ssot-alignment.md) |
| `binancex` | `crates/adapters/exchange/binance` | adapter | 7/7 | ✅ 1i+u | 644 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `okxx` | `crates/adapters/exchange/okx` | adapter | 7/7 | ✅ 1i+u | 506 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `clickhousex` | `crates/adapters/storage/clickhouse` | adapter | 7/7 | ✅ 1i+u | 477 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `kafkax` | `crates/adapters/storage/kafka` | adapter | 7/7 | ✅ 1i+u | 1066 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `natsx` | `crates/adapters/storage/nats` | adapter | 7/7 | ✅ 1i+u | 648 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `ossx` | `crates/adapters/storage/oss` | adapter | 7/7 | ✅ 1i+u | 698 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `postgresx` | `crates/adapters/storage/postgres` | adapter | 7/7 | ✅ 1i+u | 1671 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `redisx` | `crates/adapters/storage/redis` | adapter | 7/7 | ✅ 2i+u | 1458 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |
| `taosx` | `crates/adapters/storage/taos` | adapter | 7/7 | ✅ 1i+u | 723 | · | `scaffold+mock` | **89%** ███████░ | [✓](docs/ssot/adapters-ssot-alignment.md) |

### 图例

- 测试列：`i` = `tests/*.rs` 集成测试数，`u` = `src` 内 `#[cfg(test)]`
- 示例列：数字 = `examples/*.rs` 个数；`·` = 仅 `.gitkeep` 占位
- SSOT 列：链到 `docs/ssot/*-alignment.md`（存在即 ✓）

## 布局七项矩阵

| Package | src | tests | docs | benches | README | review | releases |
|---------|:---:|:-----:|:----:|:-------:|:------:|:------:|:--------:|
| `kernel` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `testkit` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `canonical` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `decimalx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bootstrap` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `configx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `evidence` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `observex` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `resiliencx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `schedulex` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `contract-testkit` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `transportx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `goalctl` | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ |
| `verifyctl` | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ |
| `contracts` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `binancex` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `okxx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `clickhousex` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `kafkax` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `natsx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `ossx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `postgresx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `redisx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `taosx` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

## 需关注（完成度 < 70%）

| Package | 完成度 | 成熟度 | 主要缺口 |
|---------|--------|--------|----------|
| `verifyctl` | 68% | `layout-incomplete` | 布局缺: docs/, benches/, review/, releases/；无 example |

## 维护（不必每次手同步）

```text
日常查看     make status                 → 写本地副本（gitignore，主仓可跑）
持续监控     make status-watch           → 同上 + 定时/变更重扫
入库更新     在 feature worktree 中 make status
            （非 main 会写根目录 STATUS.md，随 crate PR 一并提交）
强制入库     node scripts/docs/gen-crate-status.mjs --tracked
CI 门禁     node scripts/docs/gen-crate-status.mjs --check
```

**何时更新入库 STATUS.md**：`Cargo.toml` members / crate 标准布局 / 测试面实质变化时，
在同一 feature PR 里顺带刷新即可；**不要**为刷进度单独开 PR。

本地实时副本：`docs/status/CRATES_STATUS.local.md`（已 gitignore）。

相关：

- [crates/AGENTS.md](crates/AGENTS.md) — 子模块标准布局
- [docs/ssot/workspace-ssot-alignment.md](docs/ssot/workspace-ssot-alignment.md) — 镜像 vs 落地
- [docs/status/](docs/status/) — CI 状态快照
- [docs/plans/2026-07-21-core-crates-production-readiness.md](docs/plans/2026-07-21-core-crates-production-readiness.md) — 生产就绪计划（人签字）
