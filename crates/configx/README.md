# configx

L1 **内存字符串键值配置存储**（active SSOT 0.1.0 合同）。

| 项 | 值 |
|----|-----|
| package | `xhyper-configx` |
| lib | `configx` |
| path | `crates/configx` |
| version | `0.1.0` |
| publish | `false`（internal only） |

> **诚实边界**：当前只是线程安全的内存 `String` map，**不是**多源加载、schema 校验或热更新系统。
> 上位目标（多源优先级 / 热更新 / secret）在 SSOT 中为 Unknown，未批准前不实现。

规范镜像：[`../../.agents/ssot/configx/spec/spec.md`](../../.agents/ssot/configx/spec/spec.md)  
对齐说明：[`../../docs/ssot/configx-ssot-alignment.md`](../../docs/ssot/configx-ssot-alignment.md)

## 公开面

```rust
use configx::ConfigStore;

let store = ConfigStore::new();
store.set("host", "localhost")?;
assert_eq!(store.get("host").as_deref(), Some("localhost"));
```

| API | 语义 |
|-----|------|
| `ConfigStore::new()` | 空存储 |
| `get(&self, key) -> Option<String>` | 克隆返回；缺失或读锁中毒 → `None` |
| `set(&self, key, val) -> XResult<()>` | 插入/覆盖；写锁中毒 → `XError::Invalid` |
| `Default` | 等价 `new()` |

## 依赖

- 生产：仅 `xhyper-kernel`（path `../kernel`）
- feature：`default = []`（无 feature）
- **不**依赖 `observex`、serde、tokio、文件 watcher

## 验证

```bash
cargo test -p configx --all-targets
cargo clippy -p configx --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p configx --example basic
cargo llvm-cov -p configx --summary-only
```

## 非职责

- 多源加载 / 优先级合并 / 热重载
- 类型化配置 / schema 校验
- secret 管理 / 脱敏 Debug
- 全局 service locator / 订阅通知

## 生产误用红线

| 禁止 | 原因 |
|------|------|
| 作为唯一生产配置源 | 无 schema / 多源 / 热更新（SSOT DEFER） |
| 存 secret 并假设脱敏 | value 为明文 `String` |

示例：`cargo run -p configx --example basic`

## Schema 边界（infra-s9t.7）

- `require_keys(&store, &["host", "port"])`：必填 key 存在性校验（最小 schema）。
- **不是**类型化 JSON Schema / 多源合并；生产仍须上层配置加载器。
