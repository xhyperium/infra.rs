# Changelog — canonical

本文件记录 canonical 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 Cargo.toml 的 [package] version 为准；递增规则见根 AGENTS.md「版本管理」。

## [Unreleased]

### 变更

- **BREAKING（0.1 monorepo）**：删除 `OrderId` 类型别名；`Order.id` / `OrderAck.id` 改为 `String`。
- **BREAKING**：`VenueAdapter::cancel_order/query_order` 参数由 `&OrderId` 改为 `&str`（仍 deprecated）。


- CAN-TIME-001：DTO `ts` 语义冻结为 Unix ns；`ns_from_unix_millis` 正式入口。
- CAN-ID：`VenueAdapter::{cancel,query}_order_request` additive 路径由 contracts 导出。


### 变更

- 文档：active spec 补齐 `OrderRef` / `CancelOrderRequest` / Venue/Instrument ID、serde fixtures 与非权威 Candidate 边界。
- 文档：README 补充「非职责」与「限制与安全」（crate-standard wave2）。
- 文档：Candidate 链接迁至 `.agents/ssot/types/canonical/20260717/`；标注 OPEN 时间/ID 语义与 wire 承诺边界。
- 文档/测试：生产晋级路径（wire 矩阵、validation owners、M1 人审包）；`ts` 保持不透明 i64；legacy `OrderAck` golden fixture。
- 测试：全公开 DTO/枚举 serde round-trip、全部 `OrderStatus`/`OrderRef` variants、`Money`≡`decimalx::Money`、cancel fixture 双向与 legacy `OrderAck` 保持。

### 新增

- 建立初始文档骨架（CHANGELOG / AGENTS / README / docs）。
- 战役计划包：`.agents/ssot/types/canonical/plan/` + `todo.md`（agent-safe 闭合台账；**≠** Spec Approved）。
- 生产晋级：`plan/production-upgrade.md`、`approval-packet-prod-m1.md`、`wire-commitment-matrix.md`、`validation-owners.md`（M1 已签；**≠** package stable）。
- API：公开模块 `shape::*` 形状检查、`proposed_time::*`（unix ns↔ms）；DTO `ts` 语义 = Unix ns。
- Golden：`fixtures/market/canonical/v1/`（cancel / OrderRef / legacy ack）。
- 文档：`plan/m3-migration-checklist.md`；`OrderId` 类型已删除。
