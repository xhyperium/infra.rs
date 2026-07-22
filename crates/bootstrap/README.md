# bootstrap（package / lib 均为 `bootstrap`）

L1 **唯一组合根**（R3.1 豁免 / ADR-016）：启动期把 instrumentation、关停信号、可选 evidence 与固定正式 storage contracts 组装成 typed 只读上下文。

库名（`lib`）为 `bootstrap`。  
契约 SSOT：`.agents/ssot/bootstrap/spec/spec.md`。  
本仓对齐矩阵：[docs/ssot/bootstrap-ssot-alignment.md](../../docs/ssot/bootstrap-ssot-alignment.md)。

## 职责

1. **`Bootstrap` builder** — `new` / `with_instrumentation` / `with_evidence` / `require_evidence` / 四条 build 路径  
2. **`PlatformContext` / `AppContext`** — 横切只读依赖（禁止 Service Locator）  
3. **`BootstrappedApp` + `ShutdownController`** — 关停所有权（单次触发）  
4. **`MarketDataContext` / `ExecutionContext`** — 有界服务上下文  
5. **`BootstrapError`** — Missing / Invalid / Unavailable → `kernel::ErrorKind`
6. **`ContractStoreSet`** — 正式 `KeyValueStore` / `EventBus` 固定槽位；无动态注册

## ADR-005 注入链

| 角色 | 本仓 |
|------|------|
| trait 权威 | `contracts::Instrumentation`（本 crate re-export 为 `Instrumentation`） |
| 默认实现 | `observex::TracingInstrumentation`（`Bootstrap::new`） |
| 静默替面 | `NoopInstrumentation`（可选 `with_instrumentation`） |
| 消费方 | resiliencx 等只依赖 `contracts`，**禁止**依赖 observex |

## 非目标

- 通用 DI / 插件框架、配置解析、重试/调度/传输实现  
- runtime `gate` / 字符串 `register`/`resolve`  
- 跨资源事务、泛型 Repository 注册、完整应用启动编排

## 最小用法

```rust
use bootstrap::Bootstrap;

fn main() {
    let app = Bootstrap::new().build_app();
    assert!(!app.context().shutdown_signal().is_triggered());
    app.context().instrumentation().record_retry("boot", 1);
    let (ctx, shutdown) = app.into_parts();
    shutdown.trigger();
    assert!(ctx.shutdown_signal().is_triggered());
}
```

## 依赖

| 依赖 | 用途 |
|------|------|
| `xhyper-kernel` | `ShutdownGuard` / `ShutdownSignal`、`XError` / `ErrorKind` |
| `xhyper-contracts` | `Instrumentation` trait（ADR-005） |
| `xhyper-observex` | 默认 `TracingInstrumentation` |

## 验证

```bash
cargo test -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
node scripts/storage-composition-conformance.mjs
cargo llvm-cov -p bootstrap --all-targets --fail-under-lines 100 --summary-only
```

## 关停与 drain 合同（infra-s9t.5）

1. `Bootstrap::build_app` 给出 `ShutdownController`（触发）与 `AppContext.shutdown_signal`（观察）。
2. `register_drain` 可登记同步 `FnOnce() -> XResult<()>` hook，`run_drain` 按 LIFO 执行；异步资源须由应用包装或先行终结。
3. 该 hook 面不负责完整异步启动、分布式编排或自动发现依赖；应用仍须定义停收、排空和关闭顺序。
4. 可用 `ShutdownSignal::wait` / `wait_timeout` 阻塞观察；async runtime 须自适配（kernel 无 tokio）。
5. `require_evidence`：release 下 `build`/`build_app` 在未注入时 **fail-closed（panic）**；可恢复路径用 `try_build` / `try_build_app`。
