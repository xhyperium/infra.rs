# natsx

生产级 NATS 适配（`async-nats` Core NATS）：

- [`NatsPool`](src/pool.rs)：`connect` / `publish` / `subscribe` / `ping` / `health` / `close`
- [`NatsEventBus`](src/bus.rs)：`contracts::EventBus`（**at-most-once**）
- [`JetStreamConsumer`](src/jetstream.rs)：durable pull、有限等待与显式确认
- `JetStreamConsumerConfig::command_timeout`：约束确认类 broker 指令
- Core/JetStream 操作 deadline、有限 reconnect、channel capacity 与连接事件 stats
- feature `scaffold`：旧内存 `NatsAdapter` / `MockNatsBus`

## 配置

环境变量（`FOUNDATIONX_NATS_*` 优先，兼容 `FOUNDATIONX_NATSX_*`）：

| 变量 | 默认（本地草稿） |
|------|------------------|
| `URL` | `nats://127.0.0.1:4222` |
| `USER` / `USERNAME` | 无默认值 |
| `PASSWORD` | 无默认值 |
| `OPERATION_TIMEOUT_MS` | `5000` |
| `SUBSCRIPTION_CAPACITY` / `CLIENT_CAPACITY` | `256` |
| `MAX_RECONNECTS` / `RECONNECT_MAX_DELAY_MS` | `60` / `5000` |

**禁止**把草稿默认凭据用于生产；`Debug` 已脱敏密码。
远程地址必须使用 `TlsPolicy::Require`，URL userinfo 会被拒绝。

## 示例

```rust
use bytes::Bytes;
use contracts::EventBus;
use natsx::{NatsConfig, NatsEventBus, NatsPool};

# async fn demo() -> kernel::XResult<()> {
let pool = NatsPool::connect(NatsConfig::from_env()?).await?;
let bus = NatsEventBus::new(pool.clone());
bus.publish("events.demo", Bytes::from_static(b"p")).await?;
# Ok(())
# }
```

## EventBus 限制

- Core NATS：实时 pub/sub，无历史回放
- `BusMessage.id` = `{subject}/{session_seq}`（跨重连不可去重）
- 非 JetStream → 非 durable；持久消费必须显式选择 `JetStreamConsumer`
- JetStream 的 `term` / `max_deliver` 不等于自动 DLQ
- 同客户端 broker 重启恢复当前 **NO-GO**（`infra-2d9.3.1`）；不得把有限重连配置描述为已验证恢复

## 测试

```bash
cargo test -p natsx
cargo test -p natsx --features scaffold
cargo test -p natsx --test live_event_bus -- --ignored
cargo bench -p natsx --bench hot_path -- --quick
node scripts/broker-conformance.mjs
# 缺陷复现，当前预期非零：
node scripts/nats-reconnect-conformance.mjs
```

## 生产误用警示

默认实现为真实 `async-nats` 客户端。`scaffold` feature 才是进程内假实现。

文档：[docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)
