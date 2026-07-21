# `canonical` 候选完整规范

```text
Spec ID:       SPEC-TYPES-CANONICAL-002
Status:        **Approved** (2026-07-17 liukongqiang5; ≠ package stable)
Active SSOT:   .agent/SSOT/types/canonical/canonical-spec.md
Source Goal:   GOAL-TYPES-CANONICAL-002
Package:       canonical 0.1.0
Lib / Path:    canonical / crates/types/canonical
Layer:         Types / shared DTO
Snapshot:      4fe8e988 (2026-07-17 campaign baseline)
Supersedes:    Draft campaign baseline
```

## 0. 文档定位

本文件记录当前实现合同与已批准语义，不是通用 canonical encoding 规范，也 **不是** package stable。变更须同步 [active spec](../canonical-spec.md)、架构链接、受影响 contracts/adapters/domain 与 CHANGELOG；涉及 crate 布局或跨层边界时先走 `docs/specs/`。

证据采用 `[KNOWN] <confidence>` / `[INFERRED] <confidence>`；需求状态独立采用 `APPROVED` / `PROPOSED` / `OPEN` / `REJECTED`。共同失败条件：snapshot 后所引权威或源码变化。

## 1. 事实漂移台账

| 主题 | active / 源码 | 处理 |
|---|---|---|
| ID 类型 | `VenueId`、`InstrumentId`、`OrderRef`；无 `OrderId` 类型 | CAN-ID Approved；id 字段 `String` |
| cancel DTO | `CancelOrderRequest` + fixture | Committed-candidate |
| legacy cancel | contracts `&str` deprecated wrapper | 主路径走 `*_order_request` |
| wire | cancel / OrderRef / legacy ack golden | 其余 Uncommitted |
| crate 定位 | 纯 DTO | codec core **REJECTED** |

反例条件：公开类型/serde attributes/fixture 变化会推翻 API/wire 清单；出现业务方法或上层依赖会推翻纯 DTO 结论。

## 2. 依赖合同

`[KNOWN] HIGH` 当前：

```text
canonical → decimalx
canonical → serde
dev: serde_json
```

禁止依赖 `contracts`、L1、domain、adapter、service、app 或 evidence。`Money` 只能 `pub use decimalx::Money`，不得复制定义。新增依赖必须证明仍是纯 DTO 所必需；通用 codec/hash/sign 依赖不满足该条件。

## 3. 当前公开 API 基线

### 3.1 标识与引用

| 类型 | 形状 | 当前状态 |
|---|---|---|
| ~~`OrderId`~~ | **removed 2026-07-17** | `Order`/`OrderAck`.id = `String` |
| `VenueId` | `String` alias | 代码事实；语义未冻结 |
| `InstrumentId` | `String` alias | 代码事实；语义未冻结 |
| `OrderRef` | `Client(String) \| Exchange(String)` | serde externally tagged enum |
| `CancelOrderRequest` | `venue, instrument, id` | JSON fixture 已覆盖 |

### 3.2 枚举与 DTO

| 类型 | 字段/变体摘要 |
|---|---|
| `OrderStatus` | Pending/Open/PartiallyFilled/Filled/Cancelled/Rejected |
| `Side` | Buy/Sell |
| `Order` | id/symbol/side/price/qty/status |
| `OrderAck` | id/status/ts |
| `Position` | symbol/qty/entry_price |
| `Tick` | symbol/bid/ask/ts |
| `PriceLevel` | price/qty |
| `OrderBookSnapshot` | symbol/bids/asks/ts |
| `Trade` | symbol/price/qty/ts |
| `SymbolMeta` | symbol/base/quote/tick_size/min_qty |

所有字段均公开。除 DTO derive 与数据转换外，本 crate 不应增加业务方法。

## 4. 已批准不变量

### CAN-BND-001 — 纯 DTO（`APPROVED`）

- 只表达跨层数据形状。
- `OrderBookSnapshot` 不包含 diff/merge/排序状态机。
- 状态迁移、行情校验、风控、订单合法性属于 domain/adapter。
- 不进行 I/O、重试、时间读取、全局注册或审计。

### CAN-NUM-001 — 数值单点定义（`APPROVED`）

金额、价格、数量、比率和币种复用 `decimalx`。不得用 `f32/f64` 或新 `(mantissa, scale)` 结构复制金融数值。

### CAN-LAYER-001 — 分层（`APPROVED`）

contracts 与 domain 可依赖 canonical；canonical 不反向依赖它们。adapter 只通过 canonical DTO 跨越 contracts 边界，不暴露 provider SDK 类型。

## 5. 候选语义合同

### CAN-ID-001 — 标识 namespace（`APPROVED` 2026-07-17）

- 新接口强制 `OrderRef`（Client / Exchange）；不得靠 `"symbol:id"` 字符串约定扩张新接口。
- `VenueId` / `InstrumentId` 保持 `String` alias；形状由 `shape::*` 在 adapter 入口校验；跨所归一 **不做**。
- `OrderId` **类型已删除**；DTO `id` 字段为 wire `String`。
- newtype 二期可 additive 引入；见 residual OPEN-ID-002。

### CAN-TIME-001 — 时间（`APPROVED` 2026-07-17）

DTO `ts: i64` = Unix epoch **纳秒**（与 `kernel::Timestamp` 同刻度）。canonical **不**依赖 kernel。交易所 ms 写入前经 `proposed_time::ns_from_unix_millis`。负值/溢出与 kernel 语义一致由消费者处理。

### CAN-WIRE-001 — serde 与 fixture（`PROPOSED` / 部分 candidate）

- serde derive 是实现事实，不自动构成长期 wire 承诺。
- fixture 证明 `CancelOrderRequest`、`OrderRef` 与 legacy `OrderAck` 的当前 JSON 形状；其他类型只在 round-trip 范围成立。
- 每个要承诺稳定的 wire type 必须登记格式、字段/variant 命名、未知字段策略、版本、golden vectors 和 migration reader。
- Rust API compatibility 与 wire compatibility 分开验收。

### CAN-VALID-001 — 校验 owner（`APPROVED` 原则 2026-07-17）

canonical 不拒绝负 Qty、交叉盘口、非法状态迁移或未知 symbol。每个消费者必须登记在进入 domain 前由谁校验；“成功反序列化”不等于“业务有效”。见 [validation-owners.md](../plan/validation-owners.md)。

### CAN-CODEC-001 — 通用 codec（`REJECTED`）

`CanonicalWriter/Reader`、schema registry、envelope、hash/sign/evidence 不进入本 crate。需要确定性编码的模块在自己的 versioned protocol 下定义；跨仓统一 codec 只有经独立 RFC 才能改变本条。

## 6. 兼容与迁移

1. 固定当前 public API 与 fixture baseline。
2. 建立 consumer inventory：contracts、exchange adapters、domain、contract-testkit、fixtures。
3. 先 additive 新增结构化类型，再迁移 contracts 的新 trait 和 adapters。
4. legacy `Order` / `OrderAck` DTO 保留；`OrderId` 类型已删除；legacy cancel 仅 deprecated wrapper。
5. 不以 repository patch-only 版本规则豁免破坏性变更审查；版本在 release 流程中决定。

## 7. 测试与 Evidence

必须覆盖：

- 当前每个 DTO/枚举的构造与 serde round-trip（仅在承诺 serde 的类型上）；
- 所有 `OrderStatus` / `OrderRef` variants；
- golden fixtures 的正向和反向读取；
- `Money` 与 `decimalx::Money` 同一类型的编译消费；
- 不出现 domain 行为或上层依赖；
- 新旧 ID/DTO 的下游迁移编译。

聚焦命令：

```bash
cargo test -p xhyper-canonical
cargo check -p canonical --all-targets
cargo clippy -p canonical --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

受影响时追加 binance、okx、contracts、domain 与 contract-testkit 测试。单 crate round-trip 不证明跨版本或跨语言兼容。

## 8. 完成与晋级

- [x] §1 事实漂移同步到 active/本文件。
- [x] CAN-BND/NUM/LAYER 由依赖图、源码和测试持续证明。
- [x] CAN-ID/TIME/VALID 原则 **Approved**；WIRE 仅 candidate 子集。
- [x] 生产路径 legacy cancel 清零 + OrderId 类型删除 + native ack id。
- [x] 通用 codec 定位反转未被静默引入。

**本文件 Status = Approved（S1）**。  
批准、实现、发布和 package stable 是四个不同状态；**当前 ≠ package stable / crates.io**。
