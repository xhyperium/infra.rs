# contracts 公开 API

**角色**：adapter trait 出口

## 公开消费面

- Storage：`KeyValueStore` / `EventBus` / `Repository` / `TxRunner` / `TxContext` / …
- Venue：`VenueAdapter` / `ExecutionVenue` / `MarketDataSource` / …
- 观测：`Instrumentation`
- 门禁辅助：`VENUE_*_DEFAULT_MSG` / `is_default_*`
- Fake / suite：**不在本 crate** → 见 `contract-testkit`
- 接线意图：`LiveContractProfile` / `LiveHandles::validate`（非 readiness 探测）
- 准确 helper：`publish_without_delivery_attestation`、
  `kv_set_then_commit_separate_resources`

兼容名称 `bus_publish` / `tx_kv_set` 保留，但分别只表示一次 producer 调用、
以及“独立 KV set 后提交 TxContext”的顺序；不承诺 E2E delivery 或跨资源原子性。

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
