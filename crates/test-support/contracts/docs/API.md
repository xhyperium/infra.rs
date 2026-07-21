# contract-testkit 公开 API

**角色**：Fake/Recording + per-trait conformance suite（SPEC-TESTKIT-002 §3.2）

## 消费方式

```toml
[dev-dependencies]
contract-testkit = { path = "../test-support/contracts", version = "0.1.0" }
```

**禁止** 作为 production normal dependency。

## 公开面摘要

### Fake / Recording

| 符号 | 用途 |
|------|------|
| `FakeKeyValueStore` | 内存 KV |
| `FakeEventBus` | 进程内 at-most-once 总线 |
| `FakeTxContext` / `FakeTxRunner` / `RecordingTxRunner` | 事务路径 |
| `FakeRepository` | 内存仓储 |
| `RecordingInstrumentation` / `InstrEvent` | 可观测记录 |
| `FakeMarketDataSource` 等 | exchange 能力最小 Fake |

### Suite

| 函数 | 契约 |
|------|------|
| `assert_key_value_store` | KeyValueStore |
| `assert_event_bus` | EventBus |
| `assert_tx_runner` | TxRunner |
| `assert_repository` | Repository |
| `assert_instrumentation` | Instrumentation |
| `assert_market_data_source` 等 | 拆分 venue 能力 |

### 失败类型

- `ContractFailure` / `ContractResult` / `ensure`

## 最小用法

```rust
use contract_testkit::{FakeKeyValueStore, assert_key_value_store};

# async fn demo() {
let store = FakeKeyValueStore::new();
assert_key_value_store(&store).await.expect("kv suite");
# }
```
