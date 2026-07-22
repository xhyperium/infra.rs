//! EventBus 操作合同 suite。

use crate::failure::{ContractFailure, ContractResult};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::EventBus;

const C: &str = "EventBus";

/// 断言隔离 topic 的 subscribe 与 publish 操作均成功返回。
///
/// contracts 尚未统一 publish 后重放、交付顺序、ID 唯一范围或投递次数。本 suite
/// 因此不轮询流，也不把 `FakeEventBus` 的 buffered snapshot 行为外推为通用合同。
pub async fn assert_event_bus(bus: &dyn EventBus, fixture: &FixtureNamespace) -> ContractResult {
    let topic = fixture.resource("event_bus_smoke");
    let stream = bus
        .subscribe(&topic)
        .await
        .map_err(|error| ContractFailure::new(C, "subscribe", format!("订阅失败: {error}")))?;
    bus.publish(&topic, Bytes::from_static(b"contract-testkit-event-bus-v1"))
        .await
        .map_err(|error| ContractFailure::new(C, "publish", format!("发布失败: {error}")))?;
    let _ = stream;
    Ok(())
}
