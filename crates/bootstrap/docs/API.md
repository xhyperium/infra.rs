# bootstrap 公开 API

**版本 / 角色**：`bootstrap 0.3.3` · L1 组合根

## 公开消费面

- `Bootstrap::{new, with_instrumentation, with_evidence, with_store_set, with_contract_store_set, register_drain, require_evidence, build, try_build, build_app, try_build_app, take_shutdown_guard}`
- `BootstrappedApp` / `AppContext` / `PlatformContext` / `ShutdownController`
- `AppContext::{trigger_shutdown, graceful_shutdown, run_drain}` / `BootstrappedApp::{trigger_shutdown, graceful_shutdown}`
- `AsyncDrain` / `DrainStepResult` / `StoreSet`
- `ContractStoreSet`：正式 `KeyValueStore` / `EventBus` 固定可选槽位
- `MarketDataContext` / `ExecutionContext` + Bounded* traits
- `BootstrapError` / `into_xresult`

## 最小用法

```rust
use bootstrap::Bootstrap;

let app = Bootstrap::new().build_app();
app.context().instrumentation().record_retry("boot", 1);
let signal = app.context().shutdown_signal().clone();
let results = app.graceful_shutdown().expect("应用持有 shutdown guard");
assert!(signal.is_triggered());
assert!(results.is_empty());
```

`ContractStoreSet` 不提供动态 `register` / `resolve`、泛型 Repository 注册或跨资源事务；Redis/NATS 组合实验只证明正式 contracts 可从 `AppContext` 调用。

## 关停语义

- 未显式取走 guard 时，四条成功 build 路径都把唯一关停所有权放入返回产物。
- `graceful_shutdown(self)` 返回 `Result<Vec<DrainStepResult>, BootstrapError>`：先触发 signal，再 drain 本次快照，按 LIFO 返回全部正常返回步骤的结果；hook `Err` 不短路。
- 若本地 guard 缺失且 signal 在检查点尚未由外部 owner 触发，返回 `MissingDependency { name: "shutdown_guard" }` 且不执行 hook。方法消费 `self`，错误后 context 和 hook 被丢弃，不能重试。
- `trigger_shutdown(self)` 仅触发 signal。`run_drain(&self)` 仅 drain；需要完整顺序时使用 `graceful_shutdown`。
- `into_parts` 是显式所有权转移：guard 进入 `ShutdownController`，拆出的 context 不再能自行触发 signal；须先触发 controller，才能调用 graceful drain。
- drain hook 是同步闭包，可能永久阻塞或 panic。本 crate 没有 timeout、取消或 panic 隔离；调用方负责在 hook 内建立边界。
- 注册与 drain 快照由同一 mutex 线性化；快照后注册的 hook 进入下一批。不同批 hook 可能并发，不承诺跨批全局 LIFO。
- drain 注册锁中毒返回 `DependencyUnavailable { name: "drain", source }`，不伪装为配置非法。

## 错误文本

- `MissingDependency`：`缺少必需依赖：{name}`。
- `InvalidConfiguration`：`bootstrap 配置无效：{name}`。
- `DependencyUnavailable`：顶层 Display 与转换后的 `XError` context 均为 `依赖不可用：{name}`；
  任意下层 source 文本不内插到顶层，只通过 `#[source]` 链结构化保留。
- re-export 类型、下层 source 与 hook context 由定义方负责，bootstrap 不包装或翻译 opaque 错误。
