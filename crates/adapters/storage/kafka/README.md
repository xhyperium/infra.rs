# kafkax

生产级 Kafka 适配（`rdkafka`）：

- [`KafkaPool`](src/pool.rs)：`connect` / `producer` / `consumer` / `health` / `stats` / `close`
- [`KafkaProducer::publish`](src/producer.rs)：等待 delivery report
- [`KafkaConsumer`](src/consumer.rs)：subscribe + stream
- [`KafkaEventBus`](src/bus.rs)：`contracts::EventBus` facade（**at-most-once**）
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`

## 配置

环境变量 `FOUNDATIONX_KAFKAX_*`：

| 变量 | 默认（本地草稿） |
|------|------------------|
| `BROKERS` | `127.0.0.1:9092` |
| `SASL_MECHANISM` | `PLAIN` |
| `SASL_USERNAME` | `admin` |
| `SASL_PASSWORD` | （本地草稿默认；生产必须覆盖） |
| `TLS` | `false` |

**禁止**把草稿默认凭据用于生产；`Debug` 已脱敏密码。

## 示例

```rust
use bytes::Bytes;
use contracts::EventBus;
use kafkax::{KafkaConfig, KafkaEventBus, KafkaPool};

# async fn demo() -> kernel::XResult<()> {
let pool = KafkaPool::connect(KafkaConfig::from_env()).await?;
let bus = KafkaEventBus::new(pool.clone());
bus.publish("orders", Bytes::from_static(b"p")).await?;
# Ok(())
# }
```

## EventBus 限制

- `BusMessage.id` = `topic/partition/offset`
- 无 ack API → 仅 at-most-once；可靠消费请用 `KafkaConsumer` 专属面
- 流错误结束 stream（合同 item 无法表达 `Result`）
- `KafkaConsumer::subscribe` 依赖 group coordinator；若 broker 返回
  `COORDINATOR_NOT_AVAILABLE`，可用 `KafkaConsumer::assign` 手动分配分区

## 测试

```bash
cargo test -p kafkax
cargo test -p kafkax --features scaffold
cargo test -p kafkax --test live_event_bus -- --ignored
cargo bench -p kafkax --bench hot_path -- --quick
```

## 生产误用警示

默认实现为真实 `rdkafka` 客户端（非内存 mock）。`scaffold` feature 才是进程内假实现。
