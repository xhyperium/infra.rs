# domainx 目标

## 已存在

- [x] crate 已建立：`crates/domainx`（package/lib：`domainx`）
- [x] Order、Position、Trade、ExecutionReport、Portfolio 类型骨架
- [x] 共享枚举、Decimal、Timestamp 与 serde 派生

## 已完成（本轮）

- [x] DX-VAL-001..005：`validate_order` 及细分纯函数 + 单元测试
- [x] DX-API-002：版本化 JSON fixture round-trip（camelCase / Decimal / TimeInForce Gtd）

## 已完成（扩展）

- [x] DX-COMP-001：Position.status + Portfolio.commissions + 按资产汇总

## 仍待实现

- [ ] DX-CAN-001：与 domain_market 统一 instrument canonical owner
