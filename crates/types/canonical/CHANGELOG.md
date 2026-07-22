# Changelog — canonical

本文件记录 canonical 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。
版本号以对应 Cargo.toml 的 [package] version 为准；递增规则见根 AGENTS.md「版本管理」。

## [Unreleased]

## [0.1.2] - 2026-07-23 — 精确 wire 版本与无损时间转换

### Added

- `WireVersion` 与 `committed_wire_version`：精确区分 v1/v1.1/v1.2/v1.3 committed shape。
- `unix_millis_from_ns_exact`：纳秒无法无损转为毫秒时返回 `None`。

### Notes

- 承诺仍限 strict serde JSON DTO shape；不宣称 canonical bytes、通用 codec 或跨语言协议。

## [0.1.1] - 2026-07-22 — DEFER close: schema_version envelope

### Added

- `envelope` 模块：`Envelope<T>`（`schema_version` + `payload`）、版本常量、
  `wrap` / `validate_version` / `into_payload_if_version`
- unit 测试：serde 往返、拒绝缺 `schema_version`、拒绝未知字段、版本不匹配

### Notes

- envelope **不含**业务校验；DTO committed wire 仍以 `wire` 模块为准
- 依赖 `decimalx` path version 对齐 `0.1.1`

## [0.1.0] - 2026-07-21 — four-crate production tranche（L2 committed wire）

### Added

- 可运行 `examples/basic.rs`（Order/CancelOrderRequest serde + wire_commitment）
- `tests/public_api_surface.rs` 覆盖 shape/time/wire 与全 DTO serde 往返
- 真实 `benches/hot_path`
- `docs/API.md` 完整面；README 声明 **L2 committed wire subset**（v1–v1.3）
- package 选择器统一为 `canonical`

### Notes

- 证据：`docs/plans/releases/2026-07-21-four-crates-internal-release.md`
- **≠** 全 crate Production Ready / crates.io

### Historical

- workspace 登记 `crates/types/canonical`（package `canonical` / lib `canonical`）。
- 依赖：`decimalx` 使用 `path + version`，满足 cargo-deny `bans.wildcards=deny`。
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
