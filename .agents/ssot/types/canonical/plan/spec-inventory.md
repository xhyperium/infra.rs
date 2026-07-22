> **SUPERSEDED for current-state（2026-07-21）**
> 当前权威：alignment-matrix-infra-2026-07-21.md + spec/spec.md + residual-open.md + todo.md。
>

# Spec Inventory — PLAN-TYPES-CANONICAL-002

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TYPES-CANONICAL-002-v1` |
| Baseline | `main@4fe8e988` |
| 证据源 | `crates/types/canonical/src/lib.rs` · `Cargo.toml` · `fixtures/market/order_cancel_okx.json` · `cargo metadata` |

> 用途：防遗漏枚举。条目关闭须有机器/文件证据，禁止文档自证。

---

## I-API — 公开类型（必须与源码 1:1）

| ID | 类型 | 形状 | 状态 |
|----|------|------|------|
| I-API-01 | `Money` | `pub use decimalx::Money` | 复用，非本 crate 定义 |
| I-API-02 | `OrderId` | `type = String` + `#[deprecated]` | legacy wire；禁删至 consumer=0 |
| I-API-03 | `VenueId` | `type = String` | 语义 OPEN |
| I-API-04 | `InstrumentId` | `type = String` | 语义 OPEN |
| I-API-05 | `OrderRef` | `Client(String) \| Exchange(String)` | externally tagged serde |
| I-API-06 | `CancelOrderRequest` | `venue, instrument, id` | OKX fixture 覆盖 |
| I-API-07 | `OrderStatus` | 6 variants | 非状态机批准 |
| I-API-08 | `Side` | Buy/Sell | |
| I-API-09 | `Order` | id/symbol/side/price/qty/status | legacy id 字段 |
| I-API-10 | `OrderAck` | id/status/ts | legacy wire 有回归 |
| I-API-11 | `Position` | symbol/qty/entry_price | |
| I-API-12 | `Tick` | symbol/bid/ask/ts | |
| I-API-13 | `PriceLevel` | price/qty | |
| I-API-14 | `OrderBookSnapshot` | symbol/bids/asks/ts | 仅快照 |
| I-API-15 | `Trade` | symbol/price/qty/ts | |
| I-API-16 | `SymbolMeta` | symbol/base/quote/tick_size/min_qty | |

**计数**：16 公开绑定（含 re-export）。业务 `impl` 方法数：**0**。

---

## I-DEP — 依赖合同

| ID | 规则 | 期望 |
|----|------|------|
| I-DEP-01 | 生产依赖 | 仅 `decimalx`（别名 `xhyper-decimalx` 已废弃） + `serde` |
| I-DEP-02 | dev 依赖 | `serde_json` |
| I-DEP-03 | 禁止 | contracts / L1 / domain / adapter / service / app / evidence |
| I-DEP-04 | Money | 不得本地重定义 |
| I-DEP-05 | 金融浮点 | 源码无 f32/f64 金额字段 |

---

## I-INV — 已批准不变量

| ID | 名称 | 状态 |
|----|------|------|
| I-INV-01 | CAN-BND-001 纯 DTO | APPROVED |
| I-INV-02 | CAN-NUM-001 数值单点 | APPROVED |
| I-INV-03 | CAN-LAYER-001 分层 | APPROVED |
| I-INV-04 | CAN-CODEC-001 禁通用 codec | REJECTED 方向（持续禁止） |

---

## I-OPEN — 未批准语义（不得谎称闭合）

| ID | 主题 | Spec 状态 |
|----|------|-----------|
| I-OPEN-01 | CAN-ID-001 namespace/字符集/映射 | PROPOSED/OPEN |
| I-OPEN-02 | CAN-TIME-001 ts 单位 | OPEN |
| I-OPEN-03 | CAN-WIRE-001 未知字段/schema 版本/全量 golden | PROPOSED |
| I-OPEN-04 | CAN-VALID-001 validation owner 登记 | PROPOSED |
| I-OPEN-05 | 移除 serde | OPEN |
| I-OPEN-06 | types/core·protocol 布局 | OPEN |
| I-OPEN-07 | 删除 legacy OrderId/Order/OrderAck | OPEN 直至 consumer=0 |

---

## I-WIRE — 已有 fixture / 测试证据

| ID | 覆盖 | 证据 |
|----|------|------|
| I-WIRE-01 | `CancelOrderRequest` 正向+反向 | `fixtures/market/order_cancel_okx.json` + 单测 |
| I-WIRE-02 | legacy `OrderAck` JSON shape | 单测固定字符串 |
| I-WIRE-03 | `Order` round-trip | 单测 |
| I-WIRE-04 | 空 `OrderBookSnapshot` 可构造 | 单测（非 wire 承诺） |
| I-WIRE-05 | 其他 DTO wire | **无** golden；仅允许 RT 若 serde 承诺未写死 |

---

## I-CONS — 直接消费者（Cargo path dep）

| ID | Package | 用途摘要 |
|----|---------|----------|
| I-CONS-01 | `contracts`（别名 `xhyper-contracts` 已废弃） | trait 边界 DTO |
| I-CONS-02 | `binancex`（别名 `xhyper-binance` 已废弃） | REST/WS 解析与下单 |
| I-CONS-03 | `okxx`（别名 `xhyper-okx` 已废弃） | REST/WS 解析与取消 |
| I-CONS-04 | `domainx`（历史别名 `xhyper-domainx`） (domain/core) | Order/Position/Status |
| I-CONS-05 | `domain-market`（历史别名 `xhyper-domain-market`） | OrderBookSnapshot |
| I-CONS-06 | `domain-exchange`（历史别名 `xhyper-domain-exchange`） | DomainOrder ↔ Order |
| I-CONS-07 | `domain-macro`（历史别名 `xhyper-domain-macro`） | path dep |
| I-CONS-08 | `bootstrap`（别名 `xhyper-bootstrap` 已废弃） | e2e CancelOrderRequest |
| I-CONS-09 | `contract-testkit`（别名 `xhyper-contract-testkit` 已废弃） | suite fixtures |
| I-CONS-10 | `taosx`（别名 `xhyper-taosx` 已废弃） | path dep |

> 注意：`tools/goalctl` / `evidence`（别名 `xhyper-evidence` 已废弃） 中的 `canonical::` 模块名是 **另一套** encoding，不是本 crate。

---

## I-GATE — 门禁命令

| ID | 命令 |
|----|------|
| I-GATE-01 | `cargo test -p canonical（别名 xhyper-canonical 已废弃，不可用于 -p）` |
| I-GATE-02 | `cargo check -p canonical --all-targets` |
| I-GATE-03 | `cargo clippy -p canonical --all-targets -- -D warnings` |
| I-GATE-04 | `cargo xtl lint-deps` |
| I-GATE-05 | `cargo fmt -- --check` |

---

## I-FORBID — 十条禁止（同 plan §4）

见 [plan.md](./plan.md) **I-CAN-FORBID** 1–10；10x 每轮扫描。

---

## I-DOC — 文档集

| ID | 路径 | 角色 |
|----|------|------|
| I-DOC-01 | `canonical-spec.md` | active SSOT |
| I-DOC-02 | `20260717/canonical-complete-goal.md` | Goal Draft |
| I-DOC-03 | `20260717/canonical-complete-spec.md` | Spec Draft |
| I-DOC-04 | `plan/*` | 本执行包 |
| I-DOC-05 | `todo.md` | 工作台账 |
| I-DOC-06 | crate README/CHANGELOG/AGENTS | 实现侧 |
