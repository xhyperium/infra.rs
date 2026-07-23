# kernel

xhyper.rs / infra.rs 的 **L0 语义信任根**：错误分类、时间与生命周期的统一语义。

| 项 | 值 |
|----|-----|
| package | `kernel` |
| lib | `kernel` |
| path | `crates/kernel` |
| version | `0.3.1` |
| publish | `false`（internal only） |
| **生产层级** | **L1 Internal Ready + L4 Platform Ready** |
| 支持矩阵 | Linux x86_64 · MSRV 1.85 |

> **分层就绪**：可在声明的 L1+L4 范围内作为 L0 依赖使用。  
> **不是** 整体 workspace Production Ready；**不是** crates.io 发布。

| 发布 | 链接 |
|------|------|
| 内部发布记录 | [releases/0.3.0-internal.md](./releases/0.3.0-internal.md) |
| 内部 tag | `v0.3.0-four-crates` |
| 四包证据 | [docs/plans/releases/2026-07-21-four-crates-internal-release.md](../../docs/plans/releases/2026-07-21-four-crates-internal-release.md) |

契约 SSOT：`.agents/ssot/kernel/spec/spec.md`（**SPEC-KERNEL-002**）。  
本仓对齐：[docs/ssot/kernel-ssot-alignment.md](../../docs/ssot/kernel-ssot-alignment.md)。

## 职责

1. **错误分类与响应**（`error`）— 按「调用方应如何反应」分类，不按模块来源分类
2. **时间获取与表示**（`clock`）— 墙钟与单调钟分离，时间源必须显式注入
3. **生命周期与关停信号**（`lifecycle`）— 关停一次触发、多方观察、不可逆

| 模块 | 内容 |
|------|------|
| `error` | opaque `XError` + `ErrorKind`（9）/ `XResult` / `BoxError` |
| `clock` | `Timestamp` / `MonotonicInstant` / `Clock` / `SystemClock` / `ClockDomain` / `ClockError` |
| `lifecycle` | `ComponentState` / `ShutdownSignal` / `ShutdownGuard` / `LifecycleError`（无 `Component` trait） |

## 硬限制

- 不提供配置、日志、网络、异步运行时、依赖注入、持久化、serde wire 或业务能力
- 新增公开项、依赖或 feature 必须走 RFC（`[features] default = []`）
- 官方支持仅 Linux x86_64 + MSRV 1.85（见 `.agents/rules/support-matrix.md`）

## 最小用法

```rust
use kernel::{Clock, ErrorKind, SystemClock, XError, XResult};

fn tick(clock: &impl Clock) -> XResult<i64> {
    Ok(clock.now()?.as_unix_nanos())
}

fn main() {
    let clock = SystemClock::new();
    let now = tick(&clock).expect("wall clock");
    let err = XError::invalid("bad input");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    let _ = now;
}
```

```bash
cargo run -p kernel --example basic
```

## 依赖

| 依赖 | 用途 |
|------|------|
| `thiserror` | 生产错误派生（唯一生产依赖） |
| `proptest`（dev） | 属性测试 |
| `static_assertions`（dev） | 编译期负向面 |
| `loom`（`cfg(loom)`） | Shutdown 并发模型测试 |

## 验证

```bash
cargo test -p kernel --all-targets
cargo test -p kernel --doc
cargo clippy -p kernel --all-targets -- -D warnings
cargo bench -p kernel --bench hot_path -- --quick
RUSTFLAGS='--cfg loom' cargo test -p kernel --test lifecycle_concurrency_loom --release
```

公开 API 说明：[docs/API.md](./docs/API.md) · 变更日志：[CHANGELOG.md](./CHANGELOG.md)
