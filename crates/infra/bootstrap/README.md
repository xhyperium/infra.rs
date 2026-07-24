# bootstrap

L1 **唯一组合根**（R3.1 豁免 / ADR-016）：启动期把 instrumentation、关停信号、可选 evidence 与固定正式 storage contracts 组装成 typed 只读上下文。

Cargo package 与库名（`lib`）均为 `bootstrap`，当前版本 `0.3.3`，且 `publish = false`。
契约 SSOT：`.agents/ssot/bootstrap/spec/spec.md`。  
本仓对齐矩阵：[docs/ssot/bootstrap-ssot-alignment.md](../../docs/ssot/bootstrap-ssot-alignment.md)。

## 职责

1. **`Bootstrap` builder** — `new` / `with_instrumentation` / `with_evidence` / `require_evidence` / 四条 build 路径  
2. **`PlatformContext` / `AppContext`** — 横切只读依赖；`AppContext` 保有唯一关停所有权
3. **`BootstrappedApp` + `ShutdownController`** — 应用包装与兼容型所有权转移
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
- 完整 evidence wire 协议与完整 async venue trait（仍 DEFER，以 `traits` 最小对象安全替面保留组合语义）
- 跨资源事务、泛型 Repository 注册或具体 adapter 的生产凭据生命周期

## 最小用法

```rust
use bootstrap::Bootstrap;

fn main() {
let app = Bootstrap::new().build_app();
assert!(!app.context().shutdown_signal().is_triggered());
app.context().instrumentation().record_retry("boot", 1);
let signal = app.context().shutdown_signal().clone();
let results = app.graceful_shutdown().expect("应用持有 shutdown guard");
assert!(signal.is_triggered());
assert!(results.is_empty());
}
```

## 依赖

| 依赖 | 用途 |
|------|------|
| `kernel` | `ShutdownGuard` / `ShutdownSignal`、`XError` / `ErrorKind` |
| `contracts` | `Instrumentation` trait（ADR-005）以及正式 `KeyValueStore` / `EventBus` contracts |
| `observex` | 默认 `TracingInstrumentation` |
| `evidence` | `EvidenceAppender` 与进程内实现的 re-export 来源 |

Redis/NATS 仅作为固定摘要组合实验的 dev-dependencies；`ContractStoreSet` 只提供两个正式 trait-object 槽位，不把具体 adapter 引入生产依赖。

## 验证

```bash
cargo test -p bootstrap --all-targets
cargo clippy -p bootstrap --all-targets -- -D warnings
node scripts/storage-composition-conformance.mjs
node scripts/quality-gates/cov-gate-100.mjs -p bootstrap --filter crates/infra/bootstrap/src
```

## 关停与 drain 合同

1. 未显式取走 guard 时，四条公开 build 路径的成功产物都保有唯一 `ShutdownGuard` 所有权；普通 `AppContext` 可直接调用 `trigger_shutdown` 或 `graceful_shutdown`。
2. `graceful_shutdown(self)` 固定先触发 signal，再按单次快照内 LIFO 执行全部 drain hook，并以 `Result<Vec<DrainStepResult>, BootstrapError>` 返回结果；单步 hook `Err` 只进入步骤结果，不阻断后续 hook。
3. 若 context 没有 guard 且 signal 在检查点尚未由外部 owner 触发，graceful 返回 `MissingDependency("shutdown_guard")` 且不执行 hook。该方法消费 context，错误后未执行 hook 被丢弃，不能重试；并发外部触发只有发生在检查点前才使本次成功。
4. `trigger_shutdown(self)` 只触发 signal，不隐式 drain，以保持旧语义。`into_parts` 会把唯一 guard 显式移交给 `ShutdownController`；调用方须先 `controller.trigger()`，再对拆出的 context 调用 graceful。`run_drain` 仍是明确的 drain-only 逃生面。
5. `AsyncDrain` 虽沿用类型名，但执行同步 `FnOnce` hook。hook 可能永久阻塞或 panic；本 crate **不提供** deadline、取消或 panic 隔离。调用方必须在 hook 内实现有界等待。
6. `register` 与 drain 快照在同一 mutex 上线性化：快照前完成的注册进入本批，之后的注册留给下一批；hook 在锁外执行，不保证跨批全局 LIFO 或串行。注册锁中毒映射为 `DependencyUnavailable("drain")` 并保留下层原因。
7. 可用 `ShutdownSignal::wait` / `wait_timeout` 阻塞观察；async runtime 须自适配（kernel 无 tokio）。
8. `require_evidence`：`build`/`build_app` 在未注入时 release/debug 均 fail-closed（panic）；可恢复路径用 `try_build` / `try_build_app`。

## 错误语言边界

`BootstrapError` 的三类 Display 与转换后的 `XError` context 均使用简体中文：
`缺少必需依赖：{name}`、`bootstrap 配置无效：{reason}`、`依赖不可用：{name}`。其中
DependencyUnavailable 的顶层 Display 与 XError context 不内插任意下层 source 文本；bootstrap
只通过 `#[source]` 链结构化保留原因。bootstrap 自有的 drain 错误同样使用中文。re-export 类型、
下层 `source` 与 hook 返回的 opaque context 由其定义方负责，bootstrap 不翻译或吞掉下层原因。
