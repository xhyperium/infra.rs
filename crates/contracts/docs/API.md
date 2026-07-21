# contracts 公开 API

**角色**：adapter trait 出口

## 公开消费面

- Storage：`KeyValueStore` / `EventBus` / `Repository` / `TxRunner` / `TxContext` / …
- Venue：`VenueAdapter` / `ExecutionVenue` / `MarketDataSource` / …
- 观测：`Instrumentation`
- 门禁辅助：`VENUE_*_DEFAULT_MSG` / `is_default_*`
- Fake / suite：**不在本 crate** → 见 `contract-testkit`

## 最小用法

```rust
use contracts::Instrumentation;

struct Noop;
impl Instrumentation for Noop {
    fn record_retry(&self, _: &str, _: u32) {}
    fn record_circuit_open(&self, _: &str) {}
    fn record_circuit_close(&self, _: &str) {}
}
```

测试侧：

```rust
use contract_testkit::{RecordingInstrumentation, assert_instrumentation};
use contracts::Instrumentation;

let r = RecordingInstrumentation::new();
assert_instrumentation(&r).unwrap();
```
