# kafkax

kafka adapter：

- scaffold：`KafkaAdapter`
- mock 验证入口：`MockKafkaBus`（单调 `BusMessage.id`；**非**真实 Kafka）

```rust
use bytes::Bytes;
use contracts::EventBus;
use kafkax::MockKafkaBus;

# async fn demo() {
let bus = MockKafkaBus::local();
bus.publish("orders", Bytes::from_static(b"p")).await.unwrap();
# }
```
