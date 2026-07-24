# domain_market 目标

## 已存在

- [x] crate 已建立：`crates/domain_market`（package/lib：`domain_market`）
- [x] Tick、Quote、Bar、OrderBook、聚合数据类型骨架
- [x] ProductLine、InstrumentKey、DataSource、MarketFactEnvelope 骨架

## 已完成（本轮）

- [x] DM-TIME-001：事件/接收时间和 Bar 边界纯校验 + fixture
- [x] DM-BOOK-001：档位排序 / update id 纯检查（adapter 恢复状态机仍外置）
- [x] DM-SER-001：版本化 serde fixture 与 Decimal 精度门禁

## 已完成（扩展）

- [x] DM-ENV-001：typed `MarketFact`/`MarketSubject` + envelope.sequence

## 仍待实现

- [ ] DM-CAN-001：唯一 canonical instrument owner
- [ ] DM-BOOK 完整恢复状态机：provider-specific（属各 adapter，不在本 crate）
- [ ] DM-ENV 深化：管道全量切换 typed（非兼容 envelope 退役）
