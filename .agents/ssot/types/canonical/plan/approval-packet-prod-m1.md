# Approval Packet — Production M1（CAN-TIME / CAN-ID）

| 字段 | 值 |
|------|-----|
| Packet ID | `APPR-TYPES-CANONICAL-PROD-M1` |
| Spec | `SPEC-TYPES-CANONICAL-002` |
| Parent plan | [production-upgrade.md](./production-upgrade.md) |
| 日期 | 2026-07-16（UTC；与 git author 日期对齐） |
| 请求复核人 | `@liukongqiang5` |
| 授权 | **2026-07-16 UTC 会话「全部授权」+ `LIUKONGQIANG5_APPROVE_TOKEN` tip APPROVE（PR #510）** |

---

## 1. 人审裁定（生效）

| # | 事项 | 裁决 | 日期 (UTC) | 主体 |
|---|------|------|------|------|
| **T1** | CAN-TIME-001：`ts` = Unix epoch **纳秒** `i64`（与 kernel::Timestamp 同刻度） | **Approve ns** | 2026-07-16 | `liukongqiang5` |
| **T2** | CAN-ID-001：新接口 `OrderRef`；Venue/Instrument 规范 + adapter `shape`；OrderId **类型已删** | **Approve as proposed** | 2026-07-16 | `liukongqiang5` |
| **T3** | Wire 矩阵：Committed-candidate cancel/OrderRef；legacy OrderAck；其余 Uncommitted | **Approve matrix** | 2026-07-16 | `liukongqiang5` |
| **T4** | CAN-VALID：canonical 不业务校验；owner 表 v1（单一 Primary） | **Approve principle + table v1** | 2026-07-16 | `liukongqiang5` |
| **S1** | Spec Draft → **Approved**（实现合同 + 本包裁决；**≠** package stable） | **Approve** | 2026-07-16 | `liukongqiang5` |
| **S2** | package quality stable / crates.io publish | **Defer** | 2026-07-16 | `liukongqiang5` |

### T1 细节（生效）

- 负值允许；溢出 checked，禁止静默回绕。  
- 交易所 **毫秒** → DTO：`proposed_ns_from_unix_millis`（或等价 ×1_000_000）。  
- REST 签名时钟仍可用 exchange ms；**DTO `ts` 字段写 ns**。

### T2 细节（生效）

- 新执行路径优先 `CancelOrderRequest` / `OrderRef`。  
- `VenueAdapter` 结构化 `*_order_request`（**additive default Err** + 仓内 override）；legacy `cancel_order(&str)` **deprecated**。  
- Venue slug：小写 ASCII + 数字 + `-`（`shape::is_plausible_venue_slug`）。  
- `place_order` 返回原生 exchange id；**不**再编码 `{symbol}:{id}`。

---

## 2. 明确仍不批准

- package stable / crates.io 真 publish  
- 全 DTO 跨版本 wire 承诺  
- Canonical Encoding Core  

---

## 3. AI 权限（本授权回合）

| AI 可做 | AI 不可做 |
|---------|-----------|
| 按 T1–T4/S1 改 docs + 生产路径实现 | 宣称 package stable / 真 publish |
| contracts additive 默认 + adapter 接线 | 静默 rehash / 伪造 tip APPROVE |
| DTO ts 写 ns；shape 检查进 adapter | 恢复 symbol:id 编码为默认生产路径 |

---

## 4. 人审签字区

| 角色 | handle | 日期 (UTC) | 裁决 | 证据 |
|------|--------|------|------|------|
| Maintainer / Spec | `liukongqiang5` | 2026-07-16 | T1–T4 Approve；S1 Spec Approved；S2 Defer stable | 会话「全部授权」+ PR #510 tip-bound reviews（GitHub API） |
| Architecture | `liukongqiang5` | 2026-07-16 | 同左 | 同上 |

> 日期一律用 **git/UTC**，避免 wall-clock 与 author date 漂移。

---

## 5. 附件

- [production-upgrade.md](./production-upgrade.md)
- [wire-commitment-matrix.md](./wire-commitment-matrix.md)
- [validation-owners.md](./validation-owners.md)
- [m3-migration-checklist.md](./m3-migration-checklist.md)
- Spec：`../20260717/canonical-complete-spec.md`
