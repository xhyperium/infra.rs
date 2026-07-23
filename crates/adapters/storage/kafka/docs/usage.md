# kafkax 用法

## 最小示例

生产默认入口：`connect` → 代表操作 → `close`。

配置通过 `FOUNDATIONX_KAFKAX_*` 环境变量注入，或使用 `KafkaConfigBuilder`；
**禁止**把密钥写入仓库。

```rust
use kafkax::{KafkaConfigBuilder, KafkaPool};
use bytes::Bytes;
use std::time::Duration;

async fn demo() -> Result<(), kernel::XError> {
    let cfg = KafkaConfigBuilder::new()
        .brokers("127.0.0.1:9092")
        .client_id("app")
        .build()?;
    let pool = KafkaPool::connect(cfg).await?;
    let d = pool.producer().publish("topic", Bytes::from_static(b"hi")).await?;
    assert!(d.offset >= 0);
    pool.close(Duration::from_secs(3)).await?;
    Ok(())
}
```

详见同目录 `config.md` 与 `operations.md`。

## 语义边界

| 入口 | 语义 |
|------|------|
| `KafkaEventBus` | at-most-once |
| `AtLeastOnceConsumer` | 单 owner 应用 ALO（显式 ack） |
| `ProduceThenCheckpointCoordinator` | 非原子；checkpoint 失败有重复窗口 |
| group / rebalance / native EOS | **NO-GO** |

## 测试

```bash
# 单元（离线）
cargo test -p kafkax --all-targets

# 可复现单节点 broker 语义
node scripts/kafka-broker-conformance.mjs
node scripts/kafka-tls-sasl-conformance.mjs

# 生产矩阵（顺序/checksum/1MiB/并发/ALO 恢复/故障重建）
node scripts/kafka-prod-matrix.mjs
node scripts/kafka-prod-matrix.mjs --fault-restart
# 可选有界 soak（默认不进 CI）
KAFKAX_SOAK_SECONDS=60 node scripts/kafka-prod-matrix.mjs --soak

# live（需真实 Kafka + 已 export 环境变量）
node scripts/live/build-foundationx-env.mjs --env dev --out /path/private.env
set -a; source /path/private.env; set +a
cargo test -p kafkax --test live_event_bus -- --ignored --nocapture

# bench（有界；无 broker 时自动 skip produce；多 payload + 分位延迟）
cargo bench -p kafkax --bench hot_path -- --quick
```

完整清单见 [测试矩阵-生产发布.md](./测试矩阵-生产发布.md)。

## 自验证

见 crate root `kafkax::selfcheck`（LIB-SELFCHECK-SPEC §6.2）。
库内自验证 ≠ `tools/verifyctl`。

## 生产 key / headers

```rust
use bytes::Bytes;
use kafkax::{KafkaPool, PublishRecord, partition_for_key};

# async fn demo(pool: &KafkaPool) -> Result<(), kernel::XError> {
let key = Bytes::from_static(b"order-1");
let part = partition_for_key(&key, 3);
let rec = PublishRecord::payload("orders", part, Bytes::from_static(b"{}"))
    .with_key(key)
    .header("x-trace", Bytes::from_static(b"1"));
let d = pool.producer().publish_record(rec).await?;
assert!(d.offset >= 0);
# Ok(())
# }
```

背压：consumer/EventBus 有界队列；produce 受 `delivery_timeout` 约束；超时计入 `KafkaPoolStats::publish_timeouts`。
无独立 librdkafka 本地 buffer 旋钮（rskafka 路径）。

