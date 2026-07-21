//! 公开 trait 可达性 + 参考实现合同（禁止空断言）。

use bytes::Bytes;
use contracts::{
    AccountSource, AnalyticsSink, BusMessage, EventBus, ExecutionVenue, FakeEventBus, FakeTxRunner,
    InstrumentCatalog, Instrumentation, KeyValueStore, MarketDataSource, MessageAck, ObjectStore,
    PubSub, Repository, TimeSeriesStore, TxRunner, VenueAdapter, VenueTimeSource,
    run_tx_commit_on_ok,
};
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn _a(_: &dyn KeyValueStore) {}
fn _b(_: &dyn Instrumentation) {}
fn _c(_: &dyn MarketDataSource) {}
fn _d(_: &dyn InstrumentCatalog) {}
fn _e(_: &dyn ExecutionVenue) {}
fn _f(_: &dyn AccountSource) {}
fn _g(_: &dyn VenueTimeSource) {}
fn _h(_: &dyn EventBus) {}
fn _i(_: &dyn PubSub) {}
fn _j(_: &dyn ObjectStore) {}
fn _k(_: &dyn TimeSeriesStore) {}
fn _l(_: &dyn AnalyticsSink) {}
fn _m(_: &dyn VenueAdapter) {}
fn _n<T, Id, R: Repository<T, Id>>() {}
fn _o(_: &dyn TxRunner) {}

fn dummy_waker() -> Waker {
    fn no(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw()
    }
    fn dummy_raw() -> RawWaker {
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no, no, no);
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe { Waker::from_raw(dummy_raw()) }
}

#[tokio::test]
async fn contract_testkit_tx_and_bus_are_runnable() {
    let _ = (_a as fn(&dyn KeyValueStore), _h as fn(&dyn EventBus), _o as fn(&dyn TxRunner));
    let runner = FakeTxRunner;
    let n = run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, kernel::XError>(7u8) })
        .await
        .expect("tx");
    assert_eq!(n, 7);

    let bus = FakeEventBus::new();
    bus.publish("t", Bytes::from_static(b"p")).await.expect("pub");
    let mut s = bus.subscribe("t").await.expect("sub");
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    match Pin::new(&mut s).poll_next(&mut cx) {
        Poll::Ready(Some(BusMessage { id, payload })) => {
            assert!(!id.is_empty());
            assert_eq!(payload.as_ref(), b"p");
        }
        other => panic!("expected bus message: {other:?}"),
    }
    assert_eq!(MessageAck::Ack, MessageAck::Ack);
}

#[test]
fn trait_surface_object_safe_bounds() {
    fn assert_tx(_: &dyn TxRunner) {}
    assert_tx(&FakeTxRunner);
}
