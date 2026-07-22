# contract-testkit 公开 API

**角色**：Fake/Recording + per-trait conformance suite（SPEC-TESTKIT-002 §3.2）

## 消费方式

```toml
[dev-dependencies]
contract-testkit = { path = "../test-support/contracts", version = "0.1.2" }
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
| `FakeObjectStore` / `FakeTimeSeriesStore` / `FakeAnalyticsSink` / `FakePubSub` | storage/analytics Batch-2 最小 Fake |
| `FixtureNamespace` | 显式、确定、无环境依赖的资源命名空间；`resource` 校验 63 字节可移植名 |

### Suite

| 函数 | 契约 |
|------|------|
| `assert_key_value_store` / `assert_key_value_store_isolated` | KeyValueStore 兼容入口 / 隔离入口 |
| `assert_event_bus` | EventBus Snapshot/Replay profile（0.1.1 兼容，非可移植） |
| `assert_event_bus_surface` / `assert_event_bus_with_fixture` | EventBus 可移植操作 surface |
| `assert_tx_runner` | TxRunner |
| `assert_repository` | Repository |
| `assert_instrumentation` | Instrumentation |
| `assert_instrumentation_observed` | Instrumentation 可观察行为（调用方观察 seam） |
| `assert_market_data_source` 等 | 拆分 venue 能力 |
| `assert_object_store` / `assert_object_store_with_fixture` | ObjectStore 精确 payload roundtrip |
| `assert_time_series_store` / `assert_time_series_store_with_fixture` | TimeSeriesStore 写入点可查询 |
| `assert_analytics_sink` / `assert_analytics_sink_callable` / `assert_analytics_sink_observed` | AnalyticsSink 核心 / fixture / 可观察包含 |
| `assert_pub_sub_surface` / `assert_pub_sub_smoke` | PubSub subscribe/publish surface |

### 失败类型

- `ContractFailure` / `ContractResult` / `ensure`

## 最小用法

```rust
use contract_testkit::{
    FakeKeyValueStore, FixtureNamespace, assert_key_value_store_isolated,
};

# async fn demo() {
let store = FakeKeyValueStore::new();
let fixture = FixtureNamespace::new("readme_demo").expect("fixture");
assert_key_value_store_isolated(&store, &fixture).await.expect("kv suite");
# }
```

## 语义边界

- EventBus/PubSub 的可移植 surface 不读取流，不声明交付、重放、顺序或次数；`assert_event_bus` 仅保留 Snapshot/Replay profile 兼容语义。
- observed suite 的观察函数属于 test-support seam；允许额外事件、重复事件与任意顺序。
- ObjectStore 不声明覆盖、删除、列表、持久化时长或跨进程一致性。
- TimeSeriesStore 不声明返回顺序、重复写策略、唯一性或查询端点闭合规则。
- `BackendProfile` 只探测接线所需环境；可用不等于真实后端已经通过合同。
