> **历史执行计划（2026-07-17，非当前权威）**：本文件保留 PR #510 战役台账，不描述当前 `canonical 0.1.2` committed inventory。当前计划入口见 [../README.md](../README.md) 与 [wire-commitment-matrix.md](wire-commitment-matrix.md)。

# Production Upgrade Path — `xhyper-canonical`

| 字段 | 值 |
|------|-----|
| Plan ID | `PLAN-TYPES-CANONICAL-PROD-001` |
| Parent | [plan.md](./plan.md) · `PLAN-TYPES-CANONICAL-002-v1` |
| Status | **M1 语义 + M3 生产路径 DONE · ≠ package stable / Production Ready 全称** |
| Spec | `SPEC-TYPES-CANONICAL-002` **Approved**（S1；≠ package stable） |
| 日期 | 2026-07-17 |
| Branch / PR | `feat/canonical-prod-upgrade` · PR #510 |

> 本文件是**生产晋级执行路径**与终态台账。  
> Spec Approved **不等于** package stable / crates.io / 跨版本 wire 全面承诺。

---

## 0. 生产门槛

| # | 门槛 | 当前 | Owner |
|---|------|------|-------|
| P0 | `ts` 单位/epoch 人审冻结（CAN-TIME） | **DONE** ns | liukongqiang5 T1 |
| P0 | Venue/Instrument/Order ID 规范人审冻结（CAN-ID） | **DONE** | liukongqiang5 T2 |
| P1 | 承诺 wire 类型清单 + golden + unknown-field 策略 | cancel/ack/OrderRef candidate；其余 Uncommitted | 见 wire 矩阵 |
| P1 | validation owner 表覆盖全部公开 DTO | **DONE** v1 | validation-owners.md |
| P1 | 生产路径 consumers 迁到 `OrderRef` / structured cancel | **DONE** 主路径；legacy wrapper 保留 | PROD-12…16 |
| P2 | Spec Draft → Approved | **DONE** S1 | liukongqiang5 |
| P2 | package quality stable / publish | **DEFER** | HUMAN S2 |

---

## 1. 执行阶段终态

### Phase A — 人审闸

| ID | 内容 | 交付 | 状态 |
|----|------|------|------|
| A-TIME | CAN-TIME-001 | [approval-packet-prod-m1.md](./approval-packet-prod-m1.md) §TIME | **DONE** Approved ns |
| A-ID | CAN-ID-001 | 同上 §ID | **DONE** Approved |
| A-WIRE | 承诺 wire 候选清单 | [wire-commitment-matrix.md](./wire-commitment-matrix.md) | **DONE** |
| A-VALID | validation owner 表 | [validation-owners.md](./validation-owners.md) | **DONE** |
| A-SIGN | T1–T4 / S1 人审签字 | 签字区 + 会话授权 | **DONE** 2026-07-17 |

### Phase B — Additive / 实现

| ID | 内容 | 状态 |
|----|------|------|
| B-TIME | `proposed_time` ns↔ms；DTO `ts: i64` = Unix **ns** | **DONE** |
| B-ID | `shape` 形状检查；OrderRef 路径 | **DONE** |
| B-WIRE | `fixtures/market/canonical/v1/` golden | **DONE** |
| B-VALID | owner 表 + 结构测 | **DONE** |
| B-FIELD | 不引入 kernel 依赖；保持 `i64` 不透明字段 | **DONE**（刻度冻结，类型不变） |

### Phase C — M3 下游与发布

| ID | 内容 | 状态 |
|----|------|------|
| C-INV | consumer 迁移清单 | **DONE** [m3-migration-checklist.md](./m3-migration-checklist.md) |
| C-MIG-1 | contracts additive cancel/query_request + legacy deprecate | **DONE** |
| C-MIG-2 | binance/okx adapter ns + shape + native ack.id | **DONE** |
| C-MIG-3 | 生产 OrderId cancel 调用清零；legacy 仅 wrapper | **DONE** |
| C-MIG-4 | 删除 `OrderId` **类型**；id 字段 `String` | **DONE** |
| C-MIG-5 | 停止 place_order `symbol:id` 编码 | **DONE** |
| C-SPEC | Spec Approved | **DONE** S1 |
| C-STABLE | package stable / publish | **DEFER** HUMAN S2 |

---

## 2. 已生效规范（非提案）

### TIME（对齐 kernel 刻度）

- **生效**：墙钟语义与 `kernel::Timestamp` 一致——**Unix epoch 纳秒**（`i64`）。
- **canonical 字段**：保持 `ts: i64`；文档与 adapter **必须**按 ns 写。
- **转换**：交易所 ms → 写入 DTO 前经 `ns_from_unix_millis` / `dto_ts_from_unix_millis`。

### ID

- **生效**：新接口优先 `OrderRef` / `CancelOrderRequest`；`VenueId`/`InstrumentId` 仍为 `String` alias + `shape` 校验。
- **venue slug**：小写 ASCII + 数字/连字符（`is_plausible_venue_slug`）。
- **instrument**：venue 原生字符串形状（`is_plausible_instrument_id`）；不做跨所归一。
- **legacy**：`OrderId` **类型已删除**；legacy cancel/query 签名为 `&str` 且 **deprecated**，仅 wrapper。

### WIRE

| 类型 | 等级 | 说明 |
|------|------|------|
| `CancelOrderRequest` | **Committed-candidate** | fixture 双向 |
| `OrderAck` (legacy) | **Committed-legacy** | 回归字符串；id 为原生 wire string |
| `OrderRef` | **Committed-candidate** | 随 cancel 覆盖 |
| 其余 DTO/枚举 | **Uncommitted** | 仅 RT；生产勿假设跨版本 |

### VALID

见 [validation-owners.md](./validation-owners.md)：canonical **永不**做业务校验；成功反序列化 ≠ 业务有效。

---

## 3. 完成定义（本 PR 生产路径）

- [x] 本文件 + wire 矩阵 + validation owner + M1 人审包  
- [x] residual / todo 登记生产路径 ID  
- [x] crate 文档标明 ns / OrderRef / 非 package stable  
- [x] `proposed_time` + `shape` 辅助（公开模块）  
- [x] golden v1 目录 + 回归  
- [x] M3 主路径迁移（contracts + binance/okx + testkit）  
- [x] OrderId 类型删除 + symbol:id 编码清除  
- [x] T1–T4 + S1 人审  
- [ ] package stable / crates.io（**DEFER**）  
- [ ] 全 DTO 跨版本 wire 承诺（Uncommitted 仍 OPEN）

---

## 4. 禁止

1. 将 `ts` 写成毫秒（与 Approved ns 冲突）  
2. 恢复 `OrderId` 类型或生产路径 `symbol:id` 编码  
3. 将 Uncommitted wire 写成 stable  
4. 将 Spec Approved 等同 package stable  
5. Canonical Encoding Core 回流  
