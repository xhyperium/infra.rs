# contracts

R4 跨层 trait 出口（Additive Only）。Package `xhyper-contracts` / lib `contracts`。

依赖白名单：kernel + canonical + async-trait / bytes / futures-core。

Active Spec：`.agents/ssot/contracts/spec/spec.md`

## 生产入口

- **推荐**：`ExecutionVenue`（结构化 cancel/query，无 additive default）
- **迁移 facade**：`VenueAdapter`（`cancel_order_request` / `query_order_request` 有中文 Invalid default；树内 adapter 必须覆盖）

## contract-testkit（最小）

| 类型 | 用途 |
|------|------|
| `FakeTxContext` / `FakeTxRunner` / `RecordingTxRunner` | 事务 commit/rollback |
| `FakeEventBus` | at-most-once 消息 |
| `FakeKeyValueStore` | KV get/set |
| `FakeRepository` | 简单仓储 |
| `RecordingInstrumentation` | 可观测记录 |

## 文档

- 语义合同：[`docs/contracts/`](./docs/contracts/)
- SSOT 对齐：[`docs/ssot/contracts-ssot-alignment.md`](../../docs/ssot/contracts-ssot-alignment.md)

**非**整体 Production Ready（真实后端见后续工作项）。
