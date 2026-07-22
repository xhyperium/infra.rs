# contract-testkit

T0 **contract conformance** test-support（SPEC-TESTKIT-002 §3.2）。

| 项 | 值 |
|----|-----|
| package | `contract-testkit` |
| lib | `contract_testkit` |
| path | `crates/test-support/contracts` |
| version | `0.1.2` |
| publish | `false`（internal only） |
| plane | **test-support**（不是生产 runtime） |

> **仅** 作为业务 / adapter crate 的 `[dev-dependencies]`。  
> **禁止** 进入 production normal graph。

规范镜像：[`../../../.agents/ssot/testkit/spec/spec.md`](../../../.agents/ssot/testkit/spec/spec.md) §3.2  
对齐说明：[`../../../docs/ssot/testkit-ssot-alignment.md`](../../../docs/ssot/testkit-ssot-alignment.md) ·
[`../../../docs/ssot/contracts-ssot-alignment.md`](../../../docs/ssot/contracts-ssot-alignment.md)

## 公开面

### Fake / Recording

```rust
use contract_testkit::{
    FakeKeyValueStore, FakeEventBus, FakeTxRunner, RecordingTxRunner,
    FakeRepository, RecordingInstrumentation, FakeExecutionVenue,
    // …
};
```

### Per-trait suite

```rust
use contract_testkit::{
    FixtureNamespace, assert_event_bus_with_fixture, assert_key_value_store_isolated,
    assert_tx_runner,
};

let fixture = FixtureNamespace::new("adapter_contract")?;
assert_key_value_store_isolated(&my_kv, &fixture).await?;
assert_event_bus_with_fixture(&my_bus, &fixture).await?;
assert_tx_runner(&my_runner).await?;
```

套件按 trait 拆分（禁止单一 provider 大宏）：

| suite | 函数 |
|-------|------|
| KeyValueStore | `assert_key_value_store`（0.1.1 兼容）/ `assert_key_value_store_isolated` |
| EventBus | `assert_event_bus`（Snapshot/Replay profile）/ `assert_event_bus_surface` / `assert_event_bus_with_fixture` |
| TxRunner | `assert_tx_runner` |
| Repository | `assert_repository` |
| Instrumentation | `assert_instrumentation` |
| Instrumentation（可观察） | `assert_instrumentation_observed` |
| MarketDataSource | `assert_market_data_source` |
| InstrumentCatalog | `assert_instrument_catalog` |
| ExecutionVenue | `assert_execution_venue` |
| AccountSource | `assert_account_source` |
| VenueTimeSource | `assert_venue_time_source` |
| ObjectStore | `assert_object_store` / `assert_object_store_with_fixture` |
| TimeSeriesStore | `assert_time_series_store` / `assert_time_series_store_with_fixture` |
| AnalyticsSink | `assert_analytics_sink` / `assert_analytics_sink_callable` / `assert_analytics_sink_observed` |
| PubSub | `assert_pub_sub_surface` / `assert_pub_sub_smoke` |

`FixtureNamespace` 为需要资源名的 suite 提供显式、确定且可并行隔离的命名空间；不读取时间、随机数或环境变量。

可移植的 `assert_event_bus_surface` / `assert_event_bus_with_fixture` 与 PubSub surface/smoke 仅验证 subscribe/publish 可成功调用，不承诺必达、重放、顺序、确认、背压或投递次数。0.1.1 兼容入口 `assert_event_bus` 明确属于 Snapshot/Replay profile，不能用于判定实时 adapter。`AnalyticsSink` 与 `Instrumentation` 的 observed suite 依赖调用方提供观察函数，只按包含关系验证；它们不扩展生产 trait 合同，也不证明真实 exporter 或持久化后端 readiness。

## 硬限制

- 不提供通用 mock 框架 / FixtureBuilder
- 不提供 integration harness（真实网络 / Docker）
- 不与 `testkit`（ManualClock）混放
- 失败返回 [`ContractFailure`]（§9.6），避免无定位 unwrap
- `tests/negative_implementations.rs` 对 14 个 trait 提供 15 个故意破坏实现；每个反例断言精确 `contract/case`
- 不提供真实后端、网络、Docker、testnet 或 live readiness 证据

## 验证

```bash
cargo test -p contract-testkit --all-targets
cargo clippy -p contract-testkit --all-targets -- -D warnings
cargo run -p contract-testkit --example basic
cargo bench -p contract-testkit --bench hot_path -- --quick
```
