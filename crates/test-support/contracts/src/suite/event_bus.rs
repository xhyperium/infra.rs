//! EventBus profile 与可移植 surface suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use crate::fixture::FixtureNamespace;
use bytes::Bytes;
use contracts::EventBus;
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

const C: &str = "EventBus";

/// Snapshot/Replay profile：断言 publish 后 subscribe 可同步回放两条消息。
///
/// 本函数保留 0.1.1 API，但不是可移植 EventBus conformance；Kafka/NATS 实时订阅
/// 不得据此被判失败。可移植入口使用 [`assert_event_bus_surface`]。
pub async fn assert_event_bus(bus: &dyn EventBus) -> ContractResult {
    bus.publish("orders", Bytes::from_static(b"o1"))
        .await
        .map_err(|error| ContractFailure::new(C, "publish", format!("publish 失败: {error}")))?;
    bus.publish("orders", Bytes::from_static(b"o2"))
        .await
        .map_err(|error| ContractFailure::new(C, "publish2", format!("publish 失败: {error}")))?;

    let mut stream = bus.subscribe("orders").await.map_err(|error| {
        ContractFailure::new(C, "subscribe", format!("subscribe 失败: {error}"))
    })?;

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let first = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(message)) => message,
        other => {
            return Err(ContractFailure::new(
                C,
                "stream_first",
                format!("期望第一条消息，得到 {other:?}"),
            ));
        }
    };
    ensure(C, "msg_id_nonempty", !first.id.is_empty(), "消息 id 不得为空")?;
    ensure(
        C,
        "msg1_payload",
        first.payload.as_ref() == b"o1",
        format!("payload1={:?}", first.payload),
    )?;

    let second = match Pin::new(&mut stream).poll_next(&mut cx) {
        Poll::Ready(Some(message)) => message,
        other => {
            return Err(ContractFailure::new(
                C,
                "stream_second",
                format!("期望第二条消息，得到 {other:?}"),
            ));
        }
    };
    ensure(
        C,
        "msg2_payload",
        second.payload.as_ref() == b"o2",
        format!("payload2={:?}", second.payload),
    )?;
    ensure(C, "msg_ids_distinct", first.id != second.id, "连续消息 id 应不同")
}

/// 可移植 EventBus surface：先 subscribe，再 publish，只验证入口可调用。
///
/// 本函数不 poll，也不承诺投递、回放、顺序、确认、背压、唯一 ID 或投递次数。
pub async fn assert_event_bus_surface(
    bus: &dyn EventBus,
    unique_topic: &str,
    payload: Bytes,
) -> ContractResult {
    ensure(C, "unique_topic", !unique_topic.is_empty(), "测试 topic 不得为空")?;
    let stream = bus.subscribe(unique_topic).await.map_err(|error| {
        ContractFailure::new(C, "surface_subscribe", format!("subscribe 失败: {error}"))
    })?;
    bus.publish(unique_topic, payload).await.map_err(|error| {
        ContractFailure::new(C, "surface_publish", format!("publish 失败: {error}"))
    })?;
    let _ = stream;
    Ok(())
}

/// 使用确定性 fixture 运行 [`assert_event_bus_surface`]。
pub async fn assert_event_bus_with_fixture(
    bus: &dyn EventBus,
    fixture: &FixtureNamespace,
) -> ContractResult {
    let topic = fixture.resource("event_bus_smoke")?;
    assert_event_bus_surface(bus, &topic, Bytes::from_static(b"contract-testkit-event-bus-v1"))
        .await
}
