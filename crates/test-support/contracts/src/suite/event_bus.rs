//! EventBus 合同 suite。

use crate::failure::{ContractFailure, ContractResult, ensure};
use bytes::Bytes;
use contracts::EventBus;
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

const C: &str = "EventBus";

/// 断言 publish 后 subscribe 流带非空 id 与 payload。
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
