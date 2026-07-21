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

## 生产误用警示（infra-s9t.14）

**默认实现是进程内 scaffold/mock，不是生产客户端。**

- 禁止把 `*Adapter` 类型名当成已对接真实 Binance/Postgres/Redis/…
- 真实入口须有显式 feature（如 redisx `live`）与文档/CI 证据
- 详见 `docs/plans/artifacts/prod-consume-surface.md`
