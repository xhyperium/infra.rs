> **SUPERSEDED for current-state（2026-07-21）**
> 当前权威：alignment-matrix-infra-2026-07-21.md + spec/spec.md + residual-open.md + todo.md。
>

# Gap Matrix — GOAL/SPEC-TYPES-CANONICAL-002

| 字段 | 值 |
|------|-----|
| Plan | `PLAN-TYPES-CANONICAL-002-v1` |
| Baseline | `main@4fe8e988` |

图例：`MATCH` 已满足 · `PARTIAL` 部分 · `GAP` 可 agent-safe 闭合 · `OPEN` 需人审/后置 · `REJECT` 明确拒绝方向

---

## Goal § 对照

| Goal 条款 | 期望 | 现状 | 状态 | 任务 |
|-----------|------|------|------|------|
| §1 纯 DTO 基线 | 无业务行为；decimalx+serde | 源码符合 | MATCH | T-INV-* |
| §1 OrderRef 等未入 active | 登记 | active 已有表；链接坏 | PARTIAL | T-DOC-001 |
| §2 终态词汇表 | owner/consumer/兼容 | inventory 缺→补 | GAP | T-INV-001 T-CONS-001 |
| §2 OrderId→OrderRef 迁移 | additive 不破坏 wire | 并存；未删 | PARTIAL | residual OPEN-MIG |
| §2 时间/venue 语义 | 明确 | 未批准 | OPEN | residual |
| §2 serde 区分承诺 | 文档分层 | 部分 | GAP | T-DOC-002 |
| §3 Encoding Core | REJECTED | 未引入 | MATCH | 10x 负向 |
| §3 types/core 搬迁 | OPEN | 未做 | OPEN | residual |
| §3 移除 serde | OPEN | 仍依赖 | OPEN | residual |
| §3 纯 DTO APPROVED | 持续 | 符合 | MATCH | gates |
| §6 M0 事实闭合 | API/fixture/consumers | 链接/测试/README 缺口 | GAP | T-M0-* |
| §6 M1 身份时间 | 提案批准 | 未批准 | OPEN | residual HUMAN |
| §6 M2 wire 治理 | golden/版本 | cancel+ack only | PARTIAL | T-M2-* |
| §6 M3 下游 | 迁移+保留 legacy | 未全迁；保留正确 | OPEN/MATCH | inventory only |
| §7 完成定义勾选 | 全部 | agent-safe 子集 | PARTIAL | todo 分处置 |

---

## Spec § 对照

| Spec 条款 | 期望 | 现状 | 状态 | 任务 |
|-----------|------|------|------|------|
| §0 Draft 边界 | 不覆盖 active | Candidate 链接错 | GAP | T-DOC-001 |
| §1 漂移台账 | active 同步 | 类型已同步；路径错 | PARTIAL | T-DOC-001 |
| §2 依赖合同 | decimalx+serde | MATCH | MATCH | T-GATE-DEP |
| §3 API 基线 | 16 类型 | 源码一致 | MATCH | T-TEST-API |
| §4 CAN-BND/NUM/LAYER | 证明 | 基本 | PARTIAL→GAP | T-TEST-* |
| §5 CAN-ID | PROPOSED | OPEN 字段 | OPEN | residual |
| §5 CAN-TIME | OPEN | OPEN | OPEN | residual |
| §5 CAN-WIRE | fixture+文档 | 部分 | GAP | T-M2-* |
| §5 CAN-VALID | owner 登记 | 未冻结 | OPEN | residual |
| §5 CAN-CODEC | REJECTED | 未引入 | MATCH | 10x |
| §6 兼容迁移 | matrix+保留 | 需 inventory | GAP | T-CONS-001 |
| §7 测试覆盖 | 全 DTO RT 等 | 5 tests | GAP | T-TEST-001… |
| §8 晋级 | 人审 | Draft | OPEN | approval HUMAN |

---

## 字段级语义缺口（M0 表）

| 字段/类型 | 缺口 | 处置 |
|-----------|------|------|
| `ts: i64`（OrderAck/Tick/Book/Trade） | 单位未知 | OPEN-TIME；文档禁猜 |
| `OrderId` / `Order.id` / `OrderAck.id` | 无 namespace | legacy；推 OrderRef |
| `VenueId` / `InstrumentId` | 字符集/大小写 | OPEN-ID |
| `symbol` / `base` / `quote` | 规范化 | OPEN-ID |
| `OrderStatus` | 非状态机 | CAN-BND；VALID 在 domain |
| `OrderBookSnapshot.bids/asks` | 排序/交叉 | CAN-BND；VALID 在 adapter/domain |
| `qty`/`price`/`Money` | 负值/精度 | decimalx；VALID 在 consumer |
| serde 默认 shape | 未知字段 | OPEN-WIRE |

---

## 测试覆盖缺口

| 要求 | 当前 | 目标任务 |
|------|------|----------|
| 每 DTO/枚举构造 + serde RT（承诺 serde 者） | Order only | T-TEST-001 |
| 全部 OrderStatus variants | 未覆盖 | T-TEST-002 |
| 全部 OrderRef variants | Exchange only | T-TEST-003 |
| golden 正反 | cancel OK | 保持 T-TEST-004 |
| Money 类型同一 | 无 | T-TEST-005 |
| 无上层依赖 | lint-deps | T-GATE-DEP |
| 无 f32/f64 | grep | T-TEST-006 |
