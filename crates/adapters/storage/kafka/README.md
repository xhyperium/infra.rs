# kafkax

生产级 Kafka 适配（**纯 Rust `rskafka`**，无 librdkafka 系统依赖）：

- [`KafkaPool`](src/pool.rs)：`connect` / `producer` / `consumer` / `health` / `stats` / `close`
- [`KafkaProducer::publish`](src/producer.rs) / `publish_with_key` / `publish_record`：等待 broker 确认（key/headers）
- [`KafkaConsumer`](src/consumer.rs)：按分区流式消费（不依赖 group coordinator）
- [`KafkaEventBus`](src/bus.rs)：`contracts::EventBus` facade（**at-most-once**）
- [`AtLeastOnceConsumer`](src/at_least_once.rs)：单 owner、应用自管单调 checkpoint
- [`ProduceThenCheckpointCoordinator`](src/eos.rs)：非原子 produce/checkpoint，存在重复窗口
- feature `scaffold`：旧内存 `KafkaAdapter` / `MockKafkaBus`
- [`selfcheck`](src/selfcheck/)：LIB-SELFCHECK-SPEC §6.2 库内自验证（`Basic` / `ReadWrite` / `Full`）

`KafkaPool::close(deadline)` 会先拒绝新操作，再取消 broker I/O 与后台消费任务，并在
deadline 内等待在途操作释放。consumer/EventBus 使用固定容量队列；慢消费者通过等待
施加背压，关闭信号可打断等待。该行为不是自动重连或 consumer group 能力。

## 自验证边界

| 组件 | 职责 |
|------|------|
| **`kafkax::selfcheck`** | 运行时依赖可用性 / produce-consume 闭环 / Full 子集 |
| **`tools/verifyctl`** | Goal Contract 变更门禁 CLI（**不是**本模块） |

- catalog 覆盖 §6.2 全部 9 个 `kafka.*` ID；`group_lag` / `isr_health` 为 **Skipped（NO-GO）**
- `offset_commit` 验证应用层 `OffsetCommitStore`，**非** broker group commit
- `KafkaMessage::headers` + key 生产公共面已交付（selfcheck 同源）
- **未**实现跨模块 `SelfValidator` / HTTP 探针；**未**宣称 package stable

详见 [docs/usage.md](docs/usage.md) · [docs/config.md](docs/config.md) · [docs/operations.md](docs/operations.md)。

## 快速开始

```rust
use kafkax::{KafkaConfig, KafkaPool};
use bytes::Bytes;

# async fn demo() {
let pool = KafkaPool::connect(KafkaConfig::from_env()?).await?;
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
| `SASL_MECHANISM` | 仅 `PLAIN`；空则关闭 SASL |
| `SASL_USERNAME` / `SASL_PASSWORD` | SASL 凭据 |
| `TLS` | 启用 rustls transport；远程 broker 必须为 `true` |
| `TLS_CA_FILE` | 可选 PEM CA；未设置时使用 webpki roots |
| `CONNECT_TIMEOUT_MS` / `OPERATION_TIMEOUT_MS` | 内部连接与控制面 deadline |
| `CLIENT_ID` | 客户端标识，默认 `kafkax` |

可复现语义测试：`node scripts/kafka-broker-conformance.mjs`；
TLS/SASL：`node scripts/kafka-tls-sasl-conformance.mjs`。这些结果不证明
group/rebalance/HA/native EOS。
