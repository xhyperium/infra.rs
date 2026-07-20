# Alignment Matrix — infra.rs `types/canonical` 1:1 审查

| 字段 | 值 |
|------|-----|
| Matrix ID | `ALIGN-TYPES-CANONICAL-INFRA-20260721` |
| Scope | `.agents/ssot/types/canonical/**` ↔ `crates/types/canonical` (+ `crates/types/decimal`) |
| Upstream ref (read-only) | `/home/workspace/xhyper.rs/crates/types/canonical` |
| Authority | Approved production path（20260717 Spec S1 + residual + production-upgrade）；active spec 已按本矩阵重写 |
| 更新 | 2026-07-21 |
| 规则 | 每行必须有状态 + 指针；禁止未映射 claim |
| R6 说明 | 本矩阵与 active/pipeline 改写为 **intentional local SSOT maintenance**；同时写回 xhyper `.agent/SSOT` 源，避免 mirror sync 覆盖 |

## 图例

| 标签 | 含义 |
|------|------|
| **MATCH** | 文档 claim 与 live 源码一致 |
| **STALE→FIXED** | 审查时发现矛盾；已按 Approved 路径改写 |
| **OPEN** | 仍开放（见 residual） |
| **HUMAN_ONLY** | 须人类 |
| **DEFERRED** | 明确后置 |
| **POLICY** | 永久约束 |
| **REJECTED** | 持续禁止 |

## A. 公开 API 表面

| Claim | Live | 状态 | 指针 |
|-------|------|------|------|
| Package `xhyper-canonical` / lib `canonical` | 根 workspace members + Cargo.toml | MATCH | `crates/types/canonical`（2026-07-21 重登记 members，修复源码在盘但未入 workspace） |
| `VenueId`/`InstrumentId` String alias | lib.rs | MATCH | |
| `OrderRef::{Client,Exchange}` | enum | MATCH | |
| `CancelOrderRequest` | venue/instrument/id | MATCH | |
| `OrderStatus` 六变体 | Pending…Rejected | MATCH | |
| `Side::{Buy,Sell}` | enum | MATCH | |
| `Order`/`OrderAck` id=`String` | 无 OrderId 类型 | MATCH | |
| `Position`/`Tick`/`PriceLevel`/`OrderBookSnapshot`/`Trade`/`SymbolMeta` | lib.rs | MATCH | |
| `Money` = decimalx::Money | pub use + 单测 | MATCH | |
| ~~OrderId~~ 类型已删 | 无 type OrderId | MATCH | residual OPEN-ID-002 |
| 无 DTO 业务方法 / 无 reverse deps / 无 f32/f64 字段 | bounds audit | MATCH | |

## B. CAN-*

| Claim | 状态 | 指针 |
|-------|------|------|
| CAN-BND/NUM/LAYER APPROVED | MATCH | lib + decimal |
| CAN-ID-001 APPROVED | MATCH | shape.rs |
| CAN-TIME-001 `ts`=Unix ns APPROVED | MATCH | proposed_time.rs |
| CAN-WIRE 部分 candidate | MATCH/PARTIAL | wire matrix + fixtures |
| CAN-VALID 原则 APPROVED | MATCH | validation-owners |
| CAN-CODEC REJECTED | MATCH | residual REJ-* |

## C. Helpers

| Claim | 状态 | 指针 |
|-------|------|------|
| ns_from_unix_millis / unix_millis_from_ns | MATCH | proposed_time |
| is_plausible_venue_slug 正反例 | MATCH | shape |
| 1000ms → 1_000_000_000 ns 合同 | MATCH | checked_mul(1_000_000) |

## D. Tests / fixtures

| Claim | 状态 |
|-------|------|
| 全 DTO serde RT + OrderStatus/OrderRef variants | MATCH |
| cancel/legacy ack/v1 golden | MATCH |
| Money 类型同一 | MATCH |
| test/clippy/fmt 绿 | MATCH |

## E. SSOT 一致性（审查前→后）

| 对 | 后 | 状态 |
|----|----|------|
| OrderId | 类型已删 | STALE→FIXED |
| ts 单位 | Unix ns Approved | STALE→FIXED |
| Spec S1 | Approved ≠ stable | STALE→FIXED |
| pipeline not started | honest PASS/OPEN | STALE→FIXED |
| SAFE-15/16 假 DONE | DEFERRED/HUMAN | STALE→FIXED |

## F. Residual honesty

| Claim | 状态 |
|-------|------|
| package stable S2 | HUMAN_ONLY / DEFER |
| OPEN-WIRE-001/002 | OPEN |
| DEFER-M3 / NEWTYPE / LAYOUT / SERDE | DEFERRED |
| SAFE-15 10x | DEFERRED（无 fresh 证据） |
| SAFE-16 PR readback | HUMAN_ONLY 历史 |
| Encoding Core | REJECTED |

## G. 覆盖声明

公开 API 行、CAN-*、residual OPEN/DEFER/REJECT、11 管线层入口均已映射；**零未映射 claim**。
