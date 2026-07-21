# Committed wire 升格候选（W0 冻结）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-CORE-PROD-002](../2026-07-21-core-crates-production-readiness.md) |
| Beads | `infra-asa.1`（W0）· 执行 `infra-asa.3`（W2，已 close） |
| 冻结日期 | 2026-07-21 |
| 源码权威 | `crates/types/canonical/src/wire.rs` · `COMMITTED_WIRE_V1`…`V1_3` |
| 状态 | **v1.1–v1.3 已合入**（PR #124）；envelope 仍 Accept |

## 1. 已冻结（Committed v1）

| 类型 | 等级 | 证据 |
|------|------|------|
| `CancelOrderRequest` | CommittedV1 | golden / deny_unknown / 缺字段拒绝 |
| `OrderRef` | CommittedV1 | 未知 variant 拒绝 |
| `OrderAck` | CommittedV1 | N-1 legacy fixture |
| `OrderStatus` | CommittedV1 | variant 名 = wire 字符串 |
| `Side` | CommittedV1 | 双向 golden |

清单常量：`COMMITTED_WIRE_V1`（len = 5）。

## 2. 升格批次（本轮目标）

| 批次 | 类型 | 依赖 | 目标状态 | 实现状态 |
|------|------|------|----------|----------|
| **v1.1** | `Order` | 字段集人审；decimal 字段走校验反序列化 | W2 优先 | **已合入** PR #124 · `COMMITTED_WIRE_V1_1` |
| **v1.2** | `Tick`, `Trade` | 行情回放；`ts` ns 语义 | W2 | **已合入** PR #124 · `COMMITTED_WIRE_V1_2` |
| **v1.3** | `Position`, `OrderBookSnapshot`, `PriceLevel`, `SymbolMeta` | 账户/盘口；可后置 | W2 可拆 | **已合入** PR #124 · `COMMITTED_WIRE_V1_3` |

每一类型升格必须满足计划 §7.3 八项（字段冻结、deny_unknown、双向 golden、N-1、拒绝样例、decimal 校验、清单更新、align 脚本）。

## 3. 明确不在本轮

| 项 | 处置 |
|----|------|
| 协议 envelope / `schema_version` | **Accept** 延后；破坏性迁移时再上，不阻塞 v1.x |
| 全量 DTO 一次升格 | **禁止** |
| 未列入上表的新 DTO | 默认 Uncommitted，需新 PR 改本表 |

## 4. 版本命名规则

- 源码可保持 `CommittedV1` 枚举名，或按批次增加 `COMMITTED_WIRE_V1_1` 等常量。
- **破坏性** wire 变更：新类型名或新清单常量 + 迁移文档；禁止静默改 committed 字段语义。
- README / 对齐文措辞：仅对清单内类型写 wire 承诺；禁止「全 wire Production Ready」。

## 5. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | W0 初冻：v1 已有；v1.1–v1.3 升格顺序 |
| 2026-07-21 | W2 合入后：v1.1–v1.3 状态列更新为已合入（PR #124） |
