# natsx

生产级 NATS 适配（`async-nats` Core NATS）：

- [`NatsPool`](src/pool.rs)：`connect` / `publish` / `subscribe` / `ping` / `health` / `close`
- [`NatsEventBus`](src/bus.rs)：`contracts::EventBus`（**at-most-once**）
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

## 配置

环境变量（`FOUNDATIONX_NATS_*` 优先，兼容 `FOUNDATIONX_NATSX_*`）：

| 变量 | 默认（本地草稿） |
|------|------------------|
| `URL` | `nats://127.0.0.1:4222` |
| `USER` / `USERNAME` | `admin` |
| `PASSWORD` | （本地草稿默认；生产必须覆盖） |

**禁止**把草稿默认凭据用于生产；`Debug` 已脱敏密码。

## 示例

```rust
use bytes::Bytes;
use contracts::EventBus;
use natsx::{NatsConfig, NatsEventBus, NatsPool};

# async fn demo() -> kernel::XResult<()> {
let pool = NatsPool::connect(NatsConfig::from_env()).await?;
let bus = NatsEventBus::new(pool.clone());
bus.publish("events.demo", Bytes::from_static(b"p")).await?;
# Ok(())
# }
```

## EventBus 限制

- Core NATS：实时 pub/sub，无历史回放
- `BusMessage.id` = `{subject}/{session_seq}`（跨重连不可去重）
- 非 JetStream → 非 durable；at-least-once 需后续 JetStream 扩展

## 测试

```bash
cargo test -p natsx
cargo test -p natsx --features scaffold
cargo test -p natsx --test live_event_bus -- --ignored
cargo bench -p natsx --bench hot_path -- --quick
```

## 生产误用警示

默认实现为真实 `async-nats` 客户端。`scaffold` feature 才是进程内假实现。
