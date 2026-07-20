# Validation Owners — `xhyper-canonical` 公开 DTO

| 字段 | 值 |
|------|-----|
| 原则 | **CAN-VALID-001**：canonical 只表达形状；**不**做业务校验 |
| 状态 | 表 v1 **Approved** 原则（T4）；成功反序列化 ≠ 业务有效 |
| 更新 | 2026-07-17 |

## 规则

1. 每个公开业务 DTO 必须有 **Primary owner**（谁在进 domain 前负责合法性）。  
2. Primary owner **必须是单一**值 ∈ {`adapter`, `domain`, `bootstrap`, `test-only`}。  
3. 次级职责写在「备注」列，不得塞进 Primary owner。  
4. canonical crate **禁止**增加拒绝非法业务值的 API。  
5. 未在本表登记的新 DTO **不得**进入生产组装路径。

## Owner 表

| 类型 | Primary owner | 校验内容（摘要） | 备注 |
|------|---------------|------------------|------|
| `Money` | adapter | 精度/币种/非有限值 | 规则定义在 decimalx |
| `VenueId` | adapter | 非空；ASCII slug 形状 | `shape::is_plausible_venue_slug` |
| `InstrumentId` | adapter | 非空；venue 原生字符串 | 不做跨所归一 |
| `OrderRef` | adapter | Client/Exchange 载荷非空 | |
| `CancelOrderRequest` | adapter | venue/instrument/id 齐全 | fixture 仅 shape |
| `OrderStatus` | domain | 状态迁移合法性 | 枚举存在≠迁移批准 |
| `Side` | adapter | Buy/Sell 映射 | |
| `Order` | adapter | 价格/数量/符号 wire 解析 | domain 负责业务状态 |
| `OrderAck` | adapter | id/status/ts 映射（ts=ns） | id 为原生 wire string |
| `Position` | domain | qty/entry 一致性 | adapter 只负责解析 |
| `Tick` | adapter | bid/ask 解析 | 交叉由 domain/策略决定 |
| `PriceLevel` | adapter | price/qty 解析 | |
| `OrderBookSnapshot` | adapter | 快照形状/解析 | 排序/交叉/新鲜度在 domain |
| `Trade` | adapter | price/qty/symbol | |
| `SymbolMeta` | adapter | tick_size/min_qty | config 可覆盖 |

## 生产接线检查（清单）

- [ ] 每个生产 VenueAdapter 实现写明对应 owner  
- [ ] domain 入口不信任「仅反序列化成功」的行情/订单  
- [ ] 负 qty、交叉盘口、非法状态在 **domain/adapter** 失败，不在 canonical  

## 反例

| 错误做法 | 正确做法 |
|----------|----------|
| 在 `Order` 上加 `fn validate_business()` | domain newtype 上实现 |
| Primary owner 写成 `adapter + domain` | 选单一 primary；次级写备注 |
| 认为 JSON 解析成功即可下单 | adapter 校验后交 domain |
| 用 canonical 拒绝未知 symbol | 符号表在 config/domain |
