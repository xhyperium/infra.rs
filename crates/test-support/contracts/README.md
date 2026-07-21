# contract-testkit

T0 **contract conformance** test-support（SPEC-TESTKIT-002 §3.2）。

| 项 | 值 |
|----|-----|
| package | `contract-testkit` |
| lib | `contract_testkit` |
| path | `crates/test-support/contracts` |
| version | `0.1.0` |
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
use contract_testkit::{assert_key_value_store, assert_event_bus, assert_tx_runner};

assert_key_value_store(&my_kv).await?;
assert_event_bus(&my_bus).await?;
assert_tx_runner(&my_runner).await?;
```

套件按 trait 拆分（禁止单一 provider 大宏）：

| suite | 函数 |
|-------|------|
| KeyValueStore | `assert_key_value_store` |
| EventBus | `assert_event_bus` |
| TxRunner | `assert_tx_runner` |
| Repository | `assert_repository` |
| Instrumentation | `assert_instrumentation` |
| MarketDataSource | `assert_market_data_source` |
| InstrumentCatalog | `assert_instrument_catalog` |
| ExecutionVenue | `assert_execution_venue` |
| AccountSource | `assert_account_source` |
| VenueTimeSource | `assert_venue_time_source` |

## 硬限制

- 不提供通用 mock 框架 / FixtureBuilder
- 不提供 integration harness（真实网络 / Docker）
- 不与 `testkit`（ManualClock）混放
- 失败返回 [`ContractFailure`]（§9.6），避免无定位 unwrap

## 验证

```bash
cargo test -p contract-testkit --all-targets
cargo clippy -p contract-testkit --all-targets -- -D warnings
cargo run -p contract-testkit --example basic
cargo bench -p contract-testkit --bench hot_path -- --quick
```
