# bootstrap（crate 名 `xhyper-bootstrap`）

L1 **唯一组合根**（R3.1 豁免 / ADR-016）：启动期把 instrumentation、关停信号、可选 evidence 组装成 typed 只读上下文。

库名（`lib`）为 `bootstrap`。  
契约 SSOT：`.agents/ssot/infra/bootstrap/spec/spec.md`。  
本仓对齐矩阵：[docs/bootstrap-ssot-alignment.md](../../docs/bootstrap-ssot-alignment.md)。

## 职责

1. **`Bootstrap` builder** — `new` / `with_instrumentation` / `with_evidence` / `require_evidence` / 四条 build 路径  
2. **`PlatformContext` / `AppContext`** — 横切只读依赖（禁止 Service Locator）  
3. **`BootstrappedApp` + `ShutdownController`** — 关停所有权（单次触发）  
4. **`MarketDataContext` / `ExecutionContext`** — 有界服务上下文  
5. **`BootstrapError`** — Missing / Invalid / Unavailable → `kernel::ErrorKind`

## 非目标

- 通用 DI / 插件框架、配置解析、重试/调度/传输实现  
- runtime `gate` / 字符串 `register`/`resolve`  
- 完整 monorepo `xhyper-contracts` / `xhyper-observex` / `xhyper-evidence` 生产实现（本仓以 `traits` 可移植替面保留组合语义）

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

## 验证

```bash
cargo test -p xhyper-bootstrap --all-targets
cargo clippy -p xhyper-bootstrap --all-targets -- -D warnings
cargo llvm-cov -p xhyper-bootstrap --all-targets --fail-under-lines 100 --summary-only
```
