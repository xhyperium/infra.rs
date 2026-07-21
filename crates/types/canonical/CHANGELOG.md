# Changelog — canonical

本文件记录 canonical 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 Cargo.toml 的 [package] version 为准；递增规则见根 AGENTS.md「版本管理」。

## [Unreleased]

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法


### 新增

- workspace：根 `Cargo.toml` 登记 `crates/types/canonical`（package `xhyper-canonical` / lib `canonical`）。
- 依赖：`xhyper-decimalx` 使用 `path + version`，满足 cargo-deny `bans.wildcards=deny`。
- 标准布局：`AGENTS.md`、`examples/`、`docs/`、`tests/public_api.rs`（库外消费者契约）。
- API：公开模块 `shape::*` 形状检查、`proposed_time::*`（unix ns↔ms）；DTO `ts` 语义 = Unix ns。
- Golden：`fixtures/market/canonical/v1/`（cancel / OrderRef / legacy ack）。
- Golden v1.1/v1.2/v1.3：`Order` / `Tick` / `Trade` / `Position` / `PriceLevel` / `OrderBookSnapshot` / `SymbolMeta`。
- 战役计划包：`.agents/ssot/types/canonical/plan/` + `todo.md`（agent-safe 闭合台账；**≠** Spec Approved）。
- 生产晋级：`plan/production-upgrade.md`、`approval-packet-prod-m1.md`、`wire-commitment-matrix.md`、`validation-owners.md`（M1 已签；**≠** package stable）。
- Wire 常量：`COMMITTED_WIRE_V1_1` / `COMMITTED_WIRE_V1_2` / `COMMITTED_WIRE_V1_3`（infra-asa.3）。

### 变更

- 按 crates 子模块标准补齐 `examples/`、`docs/`、`tests/` 骨架与 `AGENTS.md`。
- **BREAKING（0.1 monorepo）**：删除 `OrderId` 类型别名；`Order.id` / `OrderAck.id` 改为 `String`。
- CAN-TIME-001：DTO `ts` 语义冻结为 Unix ns；`ns_from_unix_millis` 正式入口。
- 文档：active spec / alignment / residual 与 live crate 1:1；SAFE-15 DEFERRED、SAFE-16 HUMAN_ONLY。
- 测试：全公开 DTO/枚举 serde round-trip、全部 `OrderStatus`/`OrderRef` variants、`Money`≡`decimalx::Money`、cancel/legacy ack/v1 golden 双向。
- wire 矩阵：Tick/Trade `ts` 标注为 Unix ns（不再写 OPEN 单位）；Order 去掉 “deprecated id” 措辞。
- **Committed 晋升（infra-asa.3）**：`Order`（v1.1）、`Tick`/`Trade`（v1.2）、`Position`/`OrderBookSnapshot`/`PriceLevel`/`SymbolMeta`（v1.3）均 `deny_unknown_fields` + 双向 golden / N-1 / 拒绝样例 / 非法 scale。**≠** package Production Ready。
