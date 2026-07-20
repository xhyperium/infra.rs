# kernel（crate 名 `xhyper-kernel`）

xhyper.rs 的 **L0 语义信任根**：错误分类、时间与生命周期的统一语义。库名（`lib`）为 `kernel`。

契约 SSOT：`.agents/ssot/kernel/spec/spec.md`（**SPEC-KERNEL-002**）。  
本仓对齐矩阵：[docs/kernel-ssot-alignment.md](../../docs/kernel-ssot-alignment.md)。

## 职责

1. **错误分类与响应**（`error`）— 按「调用方应如何反应」分类，不按模块来源分类
2. **时间获取与表示**（`clock`）— 墙钟与单调钟分离，时间源必须显式注入
3. **生命周期与关停信号**（`lifecycle`）— 关停一次触发、多方观察、不可逆

| 模块 | 内容 |
|------|------|
| `error` | opaque `XError` + `ErrorKind`（9）/ `XResult` / `BoxError` |
| `clock` | `Timestamp` / `MonotonicInstant` / `Clock` / `SystemClock` |
| `lifecycle` | `ComponentState` / `ShutdownSignal` / `ShutdownGuard`（无 `Component` trait） |

## 非目标

不提供配置、日志、网络、异步运行时、依赖注入、持久化、serde wire 或业务能力。  
新增公开项、依赖或 feature 必须走 RFC。准入四问见 SPEC §1.1。

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

```rust
use kernel::ShutdownSignal;

let (guard, signal) = ShutdownSignal::new();
// 分发 signal.clone() 给观察者；组合根持有 guard 并在关停路径调用 guard.trigger()
let _ = (guard, signal);
```

集成测试见 `tests/`（公开 API 编译契约、clock 契约、lifecycle 并发、loom 模型等）。

## 依赖

| 依赖 | 用途 |
|------|------|
| `thiserror` | 生产错误派生（唯一生产依赖） |
| `proptest`（dev） | 属性测试 §11.3 |
| `static_assertions`（dev） | 编译期负向面 §11.4 |
| `loom`（`cfg(loom)`） | Shutdown 并发模型测试 §11.2 |

`[features] default = []`。版本跟随 workspace。

## 验证

```bash
cargo test -p xhyper-kernel --all-targets
cargo test -p xhyper-kernel --doc   # rustdoc compile_fail 负向面（--all-targets 不含 doctest）
cargo clippy -p xhyper-kernel --all-targets -- -D warnings
RUSTFLAGS='--cfg loom' cargo test -p xhyper-kernel --test lifecycle_concurrency_loom --release
```

## 目录

见 [AGENTS.md](./AGENTS.md) 与父级 [crates/AGENTS.md](../AGENTS.md) 标准布局。

## 变更日志

见 [CHANGELOG.md](./CHANGELOG.md)。
