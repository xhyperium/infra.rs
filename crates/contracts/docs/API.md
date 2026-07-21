# contracts 公开 API

**角色**：adapter trait 出口

## 公开消费面

- Storage：`KeyValueStore` / `EventBus` / `Repository` / `TxRunner` / `TxContext` / …
- Venue：`VenueAdapter` / `ExecutionVenue` / `MarketDataSource` / …
- 观测：`Instrumentation`
- Fakes：`FakeKeyValueStore` / `FakeEventBus` / `FakeTxRunner` / `RecordingInstrumentation` / …

## 最小用法

```rust
use contracts::{Instrumentation, RecordingInstrumentation};

let r = RecordingInstrumentation::new();
r.record_retry("op", 1);
assert_eq!(r.snapshot().unwrap().len(), 1);
```
