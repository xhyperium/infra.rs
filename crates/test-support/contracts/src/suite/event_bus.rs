//! EventBus 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use bytes::Bytes;
use contracts::EventBus;
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

const C: &str = "EventBus";

/// Snapshot/Replay profile：断言 publish 后 subscribe 可同步回放两条消息。
///
/// 本函数不是可移植 EventBus conformance；Kafka/NATS 实时订阅不得据此被判失败。
pub async fn assert_event_bus(bus: &dyn EventBus) -> ContractResult {
    bus.publish("orders", Bytes::from_static(b"o1"))
        .await
        .map_err(|e| ContractFailure::new(C, "publish", format!("publish 失败: {e}")))?;
    bus.publish("orders", Bytes::from_static(b"o2"))
        .await
        .map_err(|e| ContractFailure::new(C, "publish2", format!("publish 失败: {e}")))?;

    let mut stream = bus
        .subscribe("orders")
        .await
        .map_err(|e| ContractFailure::new(C, "subscribe", format!("subscribe 失败: {e}")))?;

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let m1 = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(m)) => m,
        other => {
            return Err(ContractFailure::new(
                C,
                "stream_first",
                format!("期望第一条消息，得到 {other:?}"),
            ));
        }
    };
    ensure(C, "msg_id_nonempty", !m1.id.is_empty(), "消息 id 不得为空")?;
    ensure(C, "msg1_payload", m1.payload.as_ref() == b"o1", format!("payload1={:?}", m1.payload))?;

    let m2 = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(m)) => m,
        other => {
            return Err(ContractFailure::new(
                C,
                "stream_second",
                format!("期望第二条消息，得到 {other:?}"),
            ));
        }
    };
    ensure(C, "msg2_payload", m2.payload.as_ref() == b"o2", format!("payload2={:?}", m2.payload))?;
    ensure(C, "msg_ids_distinct", m1.id != m2.id, "连续消息 id 应不同")
}

/// 可移植 EventBus surface：先 subscribe，再 publish，只验证入口可调用。
///
/// 本函数不 poll，也不承诺投递、回放、顺序或唯一 ID。
pub async fn assert_event_bus_surface(
    bus: &dyn EventBus,
    unique_topic: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_topic", !unique_topic.is_empty(), "测试 topic 不得为空")?;
    let _stream = bus.subscribe(unique_topic).await.map_err(|e| {
        ContractFailure::new(C, "surface_subscribe", format!("subscribe 失败: {e}"))
    })?;
    bus.publish(unique_topic, payload)
        .await
        .map_err(|e| ContractFailure::new(C, "surface_publish", format!("publish 失败: {e}")))
}
