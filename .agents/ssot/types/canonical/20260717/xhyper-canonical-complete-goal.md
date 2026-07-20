# Goal — `canonical` 跨层 DTO 边界与兼容性闭合

| 字段           | 值                                                              |
| -------------- | --------------------------------------------------------------- |
| Goal ID        | `GOAL-TYPES-CANONICAL-002`                                      |
| Status         | **Draft / Non-normative candidate**                             |
| Package / lib  | `xhyper-canonical` / `canonical`                                |
| Path / version | `crates/types/canonical` / `0.1.0`                              |
| Candidate Spec | [SPEC-TYPES-CANONICAL-002](./xhyper-canonical-complete-spec.md) |
| Active SSOT    | [canonical-spec.md](../canonical-spec.md)                       |
| Snapshot       | `95102919`（2026-07-15）                                        |
| Supersedes     | `none while Draft`                                              |

> `[KNOWN]` / `[INFERRED]` 是证据标签；`PROPOSED` / `OPEN` 是需求状态。`[KNOWN] HIGH` 的失败条件是所引源码、Cargo 或 Approved ADR 已变化。

## 1. 当前基线

- `[KNOWN] HIGH` ADR-001/007 把 `canonical` 定位为 `/types/` 的跨层共享纯 DTO crate；`Money`/`Decimal` 族唯一归属 `decimalx`，本 crate 只复用。
- `[KNOWN] HIGH` 当前公开 4 个标识类型（含 deprecated `OrderId`）、2 个订单引用/取消类型、2 个枚举与 8 个业务 DTO；全部是公开字段数据形状，没有业务状态机。
- `[KNOWN] HIGH` 当前生产依赖是 `xhyper-decimalx` 与 `serde`，测试以 JSON fixture 固定部分 wire shape。
- `[KNOWN] HIGH` `OrderRef`、`CancelOrderRequest`、`VenueId`、`InstrumentId` 已落地，但 active spec 尚未登记。
- `[KNOWN] HIGH` `ts: i64` 的单位、ID namespace、symbol 规范、未知字段与 schema 演进仍没有统一批准语义。

关键反例：若源码出现业务状态方法或上层依赖，纯 DTO 结论失效；若 serde attributes/fixtures 或 Approved 时间/ID 决策已变化，当前 wire/OPEN 结论失效。

## 2. 目标结果

`canonical` 的终态不是通用编码器，而是一个最小、稳定、无业务行为的跨层 DTO 词汇表：

1. 每个公开 DTO 都有明确 owner、消费者、字段语义与兼容范围。
2. `OrderId` 遗留面通过 additive 迁移收敛到带 namespace 的 `OrderRef`，不破坏已存在 wire。
3. 时间、venue、instrument、symbol 等基础标识不再依赖调用方各自猜测。
4. serde 当前形状被明确区分为“已测试 wire”或“仅实现细节”，不再把 derive 自动等同稳定协议。
5. crate 永远不承载订单状态机、订单簿 diff、校验、canonical bytes、hash/sign/evidence 或协议驱动。

## 3. 对原草案方向的裁定

| 原候选                                             | 裁定       | 理由                                                               |
| -------------------------------------------------- | ---------- | ------------------------------------------------------------------ |
| 把 crate 改造成 Canonical Encoding Core            | `REJECTED` | 违反 ADR-001/007 的纯 DTO 定位，且会迫使现有 DTO 全量迁移          |
| 新建 `types/core`、`types/protocol` 并搬走全部 DTO | `OPEN`     | 属于模块布局/跨层 API 变更，缺少 Approved RFC 与迁移证据           |
| `canonical → ∅`                                    | `REJECTED` | 当前批准方向是 `canonical → decimalx`，保证数值类型单点定义        |
| 移除 serde                                         | `OPEN`     | 当前 adapters/fixtures 依赖其 wire shape；必须先完成消费与数据迁移 |
| 纯 DTO、零业务行为                                 | `APPROVED` | ADR-001/007 与架构 §4.2 明确规定                                   |

确定性 evidence 编码由 `xhyper-evidence` 自己的 versioned canonical 模块负责；名称相似不构成迁移本 crate 职责的理由。

## 4. 候选能力与决策门

- `PROPOSED`：为 ID、时间戳、symbol/venue/instrument 建立明确语义与版本化迁移方案。
- `PROPOSED`：建立 DTO/枚举 API 快照与 golden fixtures，区分 Rust API 兼容和 wire 兼容。
- `PROPOSED`：为每个 DTO 登记 owner/consumer 与 validation owner；validation 仍在 adapter/domain。
- `OPEN`：哪些 serde shape 对外稳定、未知字段如何处理、枚举新增如何兼容。
- `OPEN`：legacy `Order`/`OrderAck` 何时迁移到结构化 ID；在无下游清零证据前不得删除。

## 5. 非目标

- 不实现 schema registry、reader/writer、通用 canonical envelope 或跨语言 codec。
- 不定义 hash、签名、证据链、幂等键或序列化框架。
- 不校验订单状态迁移、价格/数量正值、盘口排序或 symbol 存在性。
- 不复制 `decimalx` 的 `Money`、`Decimal`、`Price`、`Qty`。

## 6. 分阶段结果

### M0 — 事实闭合

同步 active spec 的当前 API、fixture 与 consumers；建立字段级语义缺口表。

### M1 — 身份与时间语义

通过提案裁定 ID namespace、时间单位/范围、字符串规范和 validation owner；以 additive 新类型迁移。

### M2 — Wire 与兼容治理

为明确承诺稳定的类型建立版本化 fixture；未承诺的 serde 形状在文档中显式标为不稳定。

### M3 — 下游迁移

迁移 contracts、binance、okx、domain 与 contract-testkit；保留遗留 API 直至消费者和历史数据路径闭合。

## 7. 完成定义

- [x] active spec 精确登记当前全部公开类型与 serde fixtures。（agent-safe 2026-07-17；见 plan + active）
- [x] crate 仍是纯 DTO，依赖方向保持 `canonical → decimalx`，无 domain/contracts/L1 依赖。（gates + lint-deps）
- [x] 所有 ID、时间和字符串语义已批准或明确标为 OPEN；不得以猜测补齐。（**标 OPEN**，未假装批准；residual）
- [x] legacy 与新 DTO 的兼容/迁移矩阵完整，下游编译和 fixture 均通过。（consumer inventory + 抽样 check；全量迁移仍 DEFERRED）
- [x] 没有把通用 codec、hash/sign/evidence 职责引入本 crate。
- [x] 聚焦测试、API/依赖门禁与受影响 adapters/domain 测试通过。（`cargo test/check/clippy -p xhyper-canonical` + lint-deps + fmt）

完成只证明 DTO 边界与 **agent-safe 事实/测试闭合**；不证明 Spec Approved、package stable、或每个 adapter 输入 / domain 业务状态有效。  
全量语义批准与 M3 大迁移见 [residual-open.md](../plan/residual-open.md) / [todo.md](../todo.md)。
