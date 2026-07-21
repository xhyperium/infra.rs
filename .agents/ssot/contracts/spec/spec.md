# `contracts` 当前实现规范

| 字段 | 值 |
|---|---|
| Status | 当前 `0.1.0` 公开契约说明；多数生产语义未稳定 |
| Package / lib | `xhyper-contracts` / `contracts` |
| Path | `crates/contracts` |
| Layer | Contract / R4 唯一跨层 trait 出口 |
| Authority | 本文件是 active current-state spec；上位架构批准状态仍优先 |
| Candidate | [SPEC-CONTRACTS-002](../../../draft/contracts-complete-spec.md)（Draft，非权威，不覆盖本文） |
| Implementation snapshot | `b0934baa`（2026-07-15） |
| Document commit | `e0b98df4` |
| Verified at | `e0b98df4`（相关实现路径未变化） |

> `[KNOWN]` 表示当前 Cargo/源码/消费方直接证明；代码存在不自动等于生产语义或上位架构已批准。trait 清单、消费者或上位规则变化会使对应结论失效。

## 1. 定位与依赖

`contracts` 只定义跨层 trait/type，不放实现；发布后受 Additive Only 约束。当前依赖：

```text
xhyper-kernel
xhyper-canonical
async-trait
bytes
futures-core
```

无 features。禁止依赖 L1、domain、adapter、service、app 或具体 async runtime。

## 2. 当前 15 个公开 trait

| 领域 | Trait | 当前实现/消费事实 | 生产语义状态 |
|---|---|---|---|
| Storage | `KeyValueStore` | redis | TTL/原子性等未统一 |
| Messaging | `EventBus` | kafka、nats | delivery/live/snapshot 未统一 |
| Storage | `Repository<T, Id>` | postgres | schema/一致性未统一 |
| Storage | `TxRunner` | postgres | transaction handle 语义有已知缺口 |
| Storage | `TimeSeriesStore` | taos | range/order/time 语义未统一 |
| Storage | `ObjectStore` | oss | missing/overwrite/metadata 未统一 |
| Analytics | `AnalyticsSink` | clickhouse | schema/delivery 未统一 |
| Messaging | `PubSub` | 实现/消费需逐项登记 | delivery/backpressure 未统一 |
| Observability | `Instrumentation` | observex 实现、resiliencx 消费 | 仅 retry/circuit 三方法 |
| Venue legacy | `VenueAdapter` | binance、okx 实现 | 13 方法宽接口；legacy compatibility |
| Venue | `MarketDataSource` | binance、okx；bootstrap bounded context 持有，crate 测试覆盖 | stream runtime error 未统一 |
| Venue | `InstrumentCatalog` | binance、okx；bootstrap bounded context 持有，crate 测试覆盖 | missing/cache 未统一 |
| Venue | `ExecutionVenue` | binance、okx；bootstrap bounded context 持有，crate 测试覆盖 | structured cancel；幂等未统一 |
| Venue | `AccountSource` | binance、okx；bootstrap bounded context 持有，crate 测试覆盖 | freshness/consistency 未统一 |
| Venue | `VenueTimeSource` | binance、okx；bootstrap bounded context 持有，crate 测试覆盖 | unit/accuracy 未统一 |

新增 5 个细粒度 venue traits 是当前代码事实，并出现在已实现 gate-retirement 计划；`docs/architecture/spec.md` 的正式契约清单仍需单独治理对齐，不由本文静默宣布全部生产批准。

## 3. 当前共同语义

- 所有 trait 为 `Send + Sync`；异步 trait 使用 `#[async_trait]`。
- stream 当前为 `BoxStream<'static, T>`；运行期 item 无 `XResult` 错误通道。
- 除 `Send + Sync`、签名与 `XResult` 外，多数 trait 不统一规定幂等、排序、背压、超时、取消、交付次数、鉴权或一致性。
- `TxRunner` 含 generic method，不应被描述为 object-safe。
- `VenueAdapter` 使用 deprecated `OrderId`；新 `ExecutionVenue` 使用 `CancelOrderRequest` 与 `VenueId`。

## 4. Additive Only 与兼容

- 不修改/删除已发布 trait 方法或类型。
- 缺口必须通过新 trait、extension trait 或新 request/response type additive 处理，并先完成上位批准。
- deprecated 不等于可删除；repository patch-only 版本策略不降低兼容要求。
- runtime implementation registry 不进入 contracts；实现选择属于 bootstrap。

Candidate Draft 中的 V2/deadline/stream/conformance 方案均不是当前合同。

## 5. 测试与 conformance 现状

- contracts crate 当前只有 2 个编译形状测试：`KeyValueStore`、`Instrumentation`。
- `xhyper-contract-testkit` 已有 7 类 suite：KeyValueStore、EventBus、MarketDataSource、InstrumentCatalog、ExecutionVenue、AccountSource、VenueTimeSource，另有 exchange mock profile。
- 其中空 stream、固定时间/余额等断言可能只适用于特定 mock profile，不得冒充所有真实 adapter 的通用语义。
- 其余 trait 尚无完整 conformance/object-safety/API baseline 证据。

## 6. 验收

```bash
cargo test -p xhyper-contracts
cargo test -p xhyper-contract-testkit
cargo check -p contracts --all-targets
cargo clippy -p contracts --all-targets -- -D warnings
cargo xtl lint-deps
cargo fmt -- --check
```

通过条件：当前 15-trait 清单与源码一致；R4 依赖通过；无实现进入 crate；Additive Only 未破坏。adapter 可编译不等于生产语义可替换。

## 7. 追溯

- `docs/architecture/spec.md` R4/R6、§4.3、§5
- [ADR-001](../../../../docs/architecture/adr/001-venue-adapter-boundary.md)
- [ADR-005](../../../../docs/architecture/adr/005-resiliencx-observability-boundary.md)
- [ADR-007](../../../../docs/architecture/adr/007-spec-consistency-revision.md)
- [PLAN-GATE-RETIRE-001](../gate/plan/xhyper-gate-retirement-complete-plan.md)
- `crates/contracts/{Cargo.toml,src/lib.rs}`
