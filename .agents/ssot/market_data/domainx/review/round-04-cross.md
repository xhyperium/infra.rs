# Round 4 — 跨域 instrument 一致性

**结论**: ready with follow-ups

## 证据
- domainx `Order.instrument: String` 与 domain_market `InstrumentKey` 并存，spec 诚实标记 DX-CAN-001/DM-CAN-001 blocked
- domain_exchange 使用 `InstrumentKey`（行情）与 `Order`（交易，String instrument）双轨，与冻结契约一致
- 无静默引入第二种 struct 作为“新 canonical”

## 问题
- 无阻断项

## Follow-ups
- 待 xhyper-canonical 入仓后统一迁移（blocked 项）
