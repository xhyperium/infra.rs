# bootstrap 公开 API

**角色**：L1 组合根

## 公开消费面

- `Bootstrap::{new, with_instrumentation, with_evidence, with_contract_store_set, require_evidence, build, try_build, build_app, try_build_app, take_shutdown_guard}`
- `BootstrappedApp` / `AppContext` / `PlatformContext` / `ShutdownController`
- `MarketDataContext` / `ExecutionContext` + Bounded* traits
- `ContractStoreSet`：正式 `KeyValueStore` / `EventBus` 的固定可选槽位
- `BootstrapError` / `into_xresult`

## 最小用法

```rust
use bootstrap::Bootstrap;

let app = Bootstrap::new().build_app();
app.context().instrumentation().record_retry("boot", 1);
let (ctx, sc) = app.into_parts();
sc.trigger();
assert!(ctx.shutdown_signal().is_triggered());
```

`ContractStoreSet` 不提供动态 register/resolve、泛型 Repository 或跨资源事务。
