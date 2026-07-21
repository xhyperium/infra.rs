# kafkax

生产级 Kafka 适配（**纯 Rust `rskafka`**，无 librdkafka 系统依赖）：

- [`KafkaPool`](src/pool.rs)：`connect` / `producer` / `consumer` / `health` / `stats` / `close`
- [`KafkaProducer::publish`](src/producer.rs)：等待 broker produce 确认
- [`KafkaConsumer`](src/consumer.rs)：按分区流式消费（不依赖 group coordinator）
- [`KafkaEventBus`](src/bus.rs)：`contracts::EventBus` facade（**at-most-once**）
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`

详见 [docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)。

## 快速开始

```rust
use kafkax::{KafkaConfig, KafkaPool};
use bytes::Bytes;

# async fn demo() {
let pool = KafkaPool::connect(KafkaConfig::from_env()).await?;
let d = pool.producer().publish("topic", Bytes::from_static(b"hi")).await?;
assert!(d.offset >= 0);
pool.close(std::time::Duration::from_secs(3)).await?;
# Ok::<(), kernel::XError>(())
# }
```

## 配置

环境变量 `FOUNDATIONX_KAFKAX_*`（**无默认密码**；生产必须注入）：

| 变量 | 说明 |
|------|------|
| `BROKERS` | bootstrap，默认 `127.0.0.1:9092` |
| `SASL_MECHANISM` | 如 `PLAIN`；空则关闭 SASL |
| `SASL_USERNAME` / `SASL_PASSWORD` | SASL 凭据 |
| `TLS` | `true`/`false`（TLS 需额外 feature 路径时见 ops） |
