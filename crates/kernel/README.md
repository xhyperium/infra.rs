# kernel（crate 名 `xhyper-kernel`）

xhyper.rs 的 **L0 语义信任根**：错误分类、时间与生命周期的统一语义。库名（`lib`）为 `kernel`。

## 职责

1. **错误分类与响应**（`error`）— 按「调用方应如何反应」分类，不按模块来源分类
2. **时间获取与表示**（`clock`）— 墙钟与单调钟分离，时间源必须显式注入
3. **生命周期与关停信号**（`lifecycle`）— 关停一次触发、多方观察、不可逆

## 非目标

不提供配置、日志、网络、异步运行时、依赖注入、持久化或业务能力。  
新增公开项、依赖或 feature 必须走 RFC。

## 最小用法

```rust
use kernel::{Clock, ErrorKind, SystemClock, XError};

let clock = SystemClock::new();
let now = clock.now().expect("wall clock");
let err = XError::invalid("bad input");
assert_eq!(err.kind(), ErrorKind::Invalid);
let _ = now;
```

集成测试见 `tests/`（公开 API 编译契约、clock 契约、lifecycle 并发等）。

## 依赖

| 依赖 | 用途 |
|------|------|
| `thiserror` | 错误派生 |
| `proptest`（dev） | 属性测试 |

版本跟随 workspace。

## 目录

见 [AGENTS.md](./AGENTS.md) 与父级 [crates/AGENTS.md](../AGENTS.md) 标准布局。

## 变更日志

见 [CHANGELOG.md](./CHANGELOG.md)。
