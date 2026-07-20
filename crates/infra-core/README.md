# infra-core

`infra.rs` workspace 的基础层 crate：错误类型、`Result` 别名与可复用的基础工具。

## 职责

- 统一错误类型 `Error` 与 `Result<T>`
- 错误序列化 / 反序列化（serde），并保留 `source` 链
- 冒烟用工具函数（如 `hello`）

## 非目标

- 不依赖外部运行时（tokio、actix 等）
- 不提供日志、网络、配置框架或业务逻辑

## 最小用法

```rust
use infra_core::{Error, Result};

fn parse_port(raw: &str) -> Result<u16> {
    raw.parse()
        .map_err(|_| Error::InvalidArgument(format!("非法端口: {raw}")))
}
```

```rust
assert_eq!(infra_core::hello(), "你好，infra-core");
```

## 依赖

| 依赖 | 用途 |
|------|------|
| `thiserror` | 错误派生 |
| `serde` | 错误序列化 |

版本跟随 workspace（`workspace = true`）。

## 目录

见 [AGENTS.md](./AGENTS.md) 与父级 [crates/AGENTS.md](../AGENTS.md) 标准布局。

## 变更日志

见 [CHANGELOG.md](./CHANGELOG.md)。
