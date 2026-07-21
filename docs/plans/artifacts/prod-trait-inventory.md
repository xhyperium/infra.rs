# 生产 trait 子集清单（W0 冻结）

| 字段 | 值 |
|------|-----|
| Plan | [PLAN-CORE-PROD-002](../2026-07-21-core-crates-production-readiness.md) |
| Beads | `infra-asa.1`（W0） |
| 冻结日期 | 2026-07-21 |
| 状态 | **Frozen**（本轮 Production Ready 范围；变更须新 PR + 更新本表） |
| 权威 trait 出口 | `crates/contracts` · package `xhyper-contracts` |

## 1. 首批（本轮 L3 目标）

| Trait | 批次 | 合同测（目标） | 真实验证入口（目标） | 备注 |
|-------|------|----------------|----------------------|------|
| `KeyValueStore` | Batch-1 | Fake + suite | `redisx` live feature | 语义简单 |
| `TxContext` | Batch-1 | Fake / Recording | `postgresx` live | 与 `TxRunner` 成对 |
| `TxRunner` | Batch-1 | Fake / Recording + `run_tx_commit_on_ok` | `postgresx` live | 已对象安全 |
| `EventBus` | Batch-1 | `FakeEventBus` | `kafkax` **或** `natsx` 其一 | at-most-once 冻结 |
| `Instrumentation` | Batch-1 | Recording 计数 | `observex` 实现 | 已有实现面 |
| `Repository` | Batch-1 | Fake（find/save/缺失） | `postgresx` | 分页策略见合同文 |
| `ExecutionVenue` | Batch-1 | mock HTTP | `okxx`/`binancex` mock | **优先于**整包 `VenueAdapter` |
| `MarketDataSource` | Batch-1 | mock 流 | exchange mock | 能力拆分 |
| `InstrumentCatalog` | Batch-1 | Fake | exchange mock | 能力拆分 |
| `AccountSource` | Batch-1 | Fake | exchange mock | 能力拆分 |
| `VenueTimeSource` | Batch-1 | Fake | exchange mock | 能力拆分 |

## 2. 条件 / 迁移期

| Trait | 状态 | 规则 |
|-------|------|------|
| `VenueAdapter` | **迁移期 facade** | 本轮不作为 L3 签字主入口；in-tree 必须 override additive defaults（DEFER-8 / W3）；能力 trait 优先；删除条件见下 |

**`VenueAdapter` 删除或降级条件（全部满足后可执行）：**

1. 全部 in-tree adapter 仅经能力 trait 被 bootstrap/消费者使用，或 facade 仅为薄委托；
2. DEFER-8 override 门禁绿 ≥ 1 个 release 周期；
3. 文档与 SSOT 对齐文声明迁移完成。

## 3. 二期（Accept · 不阻塞本轮 L3/L5）

| Trait | 理由 |
|-------|------|
| `ObjectStore` | put/get 即可；非资金主路径 |
| `TimeSeriesStore` | 依赖 `Tick` wire 升格（W2 v1.2） |
| `AnalyticsSink` | 可标 experimental |
| `PubSub` | 与 `EventBus` 能力重叠；二期收敛或独立合同 |

二期 trait 在合同文档中标注 `experimental` / `batch-2`，**禁止**写入 Production Ready 签字面。

## 4. 非目标

- 不在本轮要求每个 trait 的完整产品级 adapter（只需验证入口，见 W4）。
- 不在本轮强制独立 `crates/test-support/contracts` crate（可先 in-crate Fake，W3 再裁定）。

## 5. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-07-21 | W0 初冻：采纳计划 §5.3 建议并固化 Batch-1 / Batch-2 |
