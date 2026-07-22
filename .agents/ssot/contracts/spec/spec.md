# contracts `0.1.2` maintenance 实现合同

| 字段 | 值 |
|---|---|
| Status | Active maintenance spec；R4 trait/type 出口，**非整体 Production Ready** |
| Package / lib | `contracts` / `contracts` |
| Path | `crates/contracts` |
| Baseline | `3cd29a942710c0fb42f3f6bc05e3c31570acad47`（2026-07-23 审计） |
| Target | `0.1.2`（`0.1.1` PATCH +1） |
| Mirror | `spec/spec.md` 与 `spec/xhyper-contracts-complete-spec.md` 必须 byte-identical |

## 1. 边界与兼容

- contracts 定义跨层 trait/type，也允许不绑定具体 backend 的轻量 orchestration helpers；禁止放 Redis/Postgres/交易所等 backend adapter 实现。
- 15 个已发布 trait 及其方法受 **Additive Only** 约束：不删、不改签名、不弱化既有默认错误。
- runtime implementation registry、凭据、连接生命周期与 readiness attestation 属于 bootstrap/adapter/部署治理，不进入本 crate。
- `LiveContractProfile` 只表达接线意图，不是后端可用、业务 live 或生产 readiness 证明。

## 2. 当前 trait 面

Storage/messaging/analytics：`KeyValueStore`、`EventBus`、`Repository`、`TxRunner`、`TimeSeriesStore`、`ObjectStore`、`AnalyticsSink`、`PubSub`；observability：`Instrumentation`；venue：`VenueAdapter`、`MarketDataSource`、`InstrumentCatalog`、`ExecutionVenue`、`AccountSource`、`VenueTimeSource`。

共同限制保持不变：stream item 没有统一 runtime error 通道；幂等、排序、背压、鉴权、交付次数、一致性与 freshness 多数未统一。真实 adapter 可编译不等于语义可替换或 package stable。

## 3. Live profile / handles 合同

- `LiveHandles::validate` 只验证当前句柄类型能直接证明的能力。
- `kv`、`bus`、`tx`、`venue` 为 true 时必须有对应句柄。
- 当前 `LiveHandles` 没有 `Repository`、`AccountSource`、`VenueTimeSource` 句柄；profile 若声明 `repo`、`account` 或 `venue_time`，`validate` 必须 fail-closed，禁止由其他句柄推断。
- validate 成功只表示“声明与引用形状一致”，不执行 I/O 探测，不构成 readiness attestation。

## 4. Helper 语义

- `kv_roundtrip` 只证明一次 set/get 观察一致，不证明持久性、复制或并发一致性。
- `bus_publish` 为兼容保留；它只调用一次 `EventBus::publish`，不 subscribe、不确认消费/交付。新增准确命名 helper 供新调用方使用。
- `tx_kv_set` 为兼容保留；KV store 与 `TxContext` 没有绑定，调用顺序不能证明一个原子事务。新增准确命名 helper 明示“set 成功后提交独立事务上下文”。
- `venue_place_and_query` / `venue_health` 只是调用编排，不能证明鉴权、签名、真实成交或行情 live。

## 5. 测试与 NO-GO

公共 API 测试必须覆盖所有不可证明的 profile flag、底层 helper 失败传播与兼容别名。contract-testkit 仍是 dev-only 测试支持，本轮禁止修改其源码。

保持 **OPEN / NO-GO**：全 trait 深度 conformance、交易所签名/下单/WS 业务 live、编译期 Venue override gate、跨 backend 事务原子性、EventBus E2E 交付与 Agent/Maintainer 生产签署。

```bash
cargo test -p contracts --all-targets
cargo test -p contract-testkit --all-targets
cargo clippy -p contracts --all-targets -- -D warnings
cargo doc -p contracts --no-deps
```
