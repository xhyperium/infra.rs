# `.cargo/draft` → SSOT 覆盖审查

**核验日期**：2026-07-23
**结论**：可转化的 draft 模块均有 SSOT 入口；订单簿是本轮新增主题。draft 中的空文件和与当前 Rust workspace 不相符的实现细节被记录为来源/待实现要求，没有被伪装成现状。

| draft 输入 | 对应 SSOT | 处理结论 |
|---|---|---|
| `.cargo/draft/goal.md` + `.cargo/draft/spec.md` | `.agents/ssot/binance/{goal,design,spec}/` | Binance 总体目标/技术规格；Kafka、Redis、ClickHouse 等仅在实现存在后关闭门禁 |
| `.cargo/draft/coinbase/goal.md` | `.agents/ssot/coinbase/{goal,design,spec}/` | 单文件内含 GOAL+SPEC；空的 `coinbase/spec.md` 不重复制造内容 |
| `.cargo/draft/coinglass/goal.md` | `.agents/ssot/coinglass/{goal,design,spec}/` | 单文件内含 GOAL+SPEC；认证、V4 schema 与 REST-only 边界按事实状态处理 |
| `.cargo/draft/hyperliquid/goal.md` | `.agents/ssot/hyperliquid/{goal,design,spec}/` | 单文件内含 GOAL+SPEC；`allMids` 不伪造 Quote，`webbook2` 保持 deferred |
| `.cargo/draft/okx/goal.md` | `.agents/ssot/okx/{goal,design,spec}/` | 单文件内含 GOAL+SPEC；checksum/sequence/重连需 fixture 才能 verified |
| `.cargo/draft/orderbook.md` | `.agents/ssot/orderbook/{goal,design,spec}/` | Binance engine v1 收敛为 service profile/model A |
| `.cargo/draft/orderbook/1.md` | `.agents/ssot/orderbook/{goal,design,spec}/` | 多交易所 core v2 作为主契约，建立三种同步模型和 adapter SPI |
| `.cargo/draft/arch.md`（0 bytes） | 无独立主题 | 空输入不产生伪造 SSOT；跨层架构已在 orderbook design 与现有 ARCHITECTURE.md 分别记录 |
| `.cargo/draft/*/spec.md`（coinbase/coinglass/hyperliquid/okx，0 bytes） | 各自 `spec/spec.md` | 内容实际合并在对应 draft `goal.md`，空文件只作为目录占位 |

## 关键裁决

1. 当前 workspace 是 Rust 类型/trait skeleton，不是 draft 中的 Go 1.22 runtime；SSOT 保留语义目标，但将实现、fixture、live 连接和基础设施标为 `pending`/`deferred`。
2. `domain_market::OrderBook` 是公共数据形状 owner；订单簿 provider-specific 恢复状态机不能复制到 `domain_market`。
3. draft 的“已覆盖”只表示需求文字存在，不表示 Cargo 编译或当前 adapter skeleton 已完成对应功能。

最新逐门禁追溯见 [`traceability-matrix.md`](traceability-matrix.md)；该矩阵必须覆盖每个主题 `spec/spec.md` 的门禁 ID。
