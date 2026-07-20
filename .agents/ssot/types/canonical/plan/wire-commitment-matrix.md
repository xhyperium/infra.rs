# Wire Commitment Matrix — `xhyper-canonical`

| 字段 | 值 |
|------|-----|
| SSOT | 本文件（候选）；人审后可并入 active spec §4 |
| 状态 | **提案 / 实现事实混合** · **≠** 全 crate wire stable |
| 更新 | 2026-07-17 |

## 等级定义

| 等级 | 含义 | 生产含义 |
|------|------|----------|
| **Committed-candidate** | 有 golden/fixture 双向证据；拟作为兼容面 | 变更须回归测试 + 版本说明 |
| **Committed-legacy** | 历史 shape 冻结至迁移完成 | 仅 bugfix；优先迁新类型 |
| **Uncommitted** | 仅有 serde RT 或构造测 | **禁止**假设跨版本/跨语言稳定 |
| **OPEN** | 策略未批（unknown-field 等） | 不得写死 |

## 类型矩阵

| 类型 | 等级 | 证据 | 备注 |
|------|------|------|------|
| `CancelOrderRequest` | Committed-candidate | `fixtures/market/order_cancel_okx.json` + 单测双向 | OKX 取消路径 |
| `OrderRef` | Committed-candidate | 随 cancel fixture；Client/Exchange 变体测 | externally tagged |
| `OrderAck` | Committed-legacy | 固定 JSON 字符串回归 | `id: String` |
| `Order` | Uncommitted | serde RT | 含 deprecated id |
| `OrderStatus` | Uncommitted | variants RT | 非状态机批准 |
| `Side` | Uncommitted | RT | |
| `Position` | Uncommitted | RT | |
| `Tick` | Uncommitted | RT | `ts` OPEN |
| `PriceLevel` | Uncommitted | RT | |
| `OrderBookSnapshot` | Uncommitted | RT + 空簿构造 | 无 diff 语义 |
| `Trade` | Uncommitted | RT | `ts` OPEN |
| `SymbolMeta` | Uncommitted | RT | |
| `Money` | N/A（decimalx） | 类型同一性测 | 数值 SSOT 在 decimalx |
| `VenueId` / `InstrumentId` | alias | — | shape 校验 |
| ~~`OrderId`~~ | **removed** | — | `Order`/`OrderAck`.id 为 `String` |

## Unknown-field（OPEN）

| 策略 | 状态 |
|------|------|
| serde 默认忽略未知字段 | **当前实现事实** |
| deny unknown | **未批准**；若需要在 adapter 做严格信封 |

## 版本

- 无独立 wire schema_version 字段。  
- 跨版本兼容 **未** 承诺。  
- 晋升 Committed 须：golden 向量 + 迁移说明 + 人审（见 production-upgrade Phase B）。
