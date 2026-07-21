# natsx

nats adapter：

- scaffold：`NatsAdapter`
- mock 验证入口：`MockNatsBus`（单调 `BusMessage.id`；**非**真实 NATS）

```rust
use bytes::Bytes;
use contracts::EventBus;
use natsx::MockNatsBus;

# async fn demo() {
let bus = MockNatsBus::local();
bus.publish("subj", Bytes::from_static(b"p")).await.unwrap();
# }
```
