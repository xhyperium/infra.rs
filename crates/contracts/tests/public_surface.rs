//! 公开 trait 可达性 + 经 contract-testkit 的参考实现合同（禁止空断言）。

use bytes::Bytes;
use contract_testkit::{
    FakeEventBus, FakeKeyValueStore, FakeRepository, FakeTxRunner, FixtureNamespace, InstrEvent,
    RecordingInstrumentation, assert_event_bus, assert_key_value_store,
};
use contracts::{
    AccountSource, AnalyticsSink, BusMessage, EventBus, ExecutionVenue, InstrumentCatalog,
    Instrumentation, KeyValueStore, MarketDataSource, MessageAck, ObjectStore, PubSub, Repository,
    TimeSeriesStore, TxRunner, VenueAdapter, VenueTimeSource, run_tx_commit_on_ok,
};
use kernel::XError;
use std::time::Duration;

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

#[tokio::test]
async fn contract_testkit_tx_and_bus_are_runnable() {
    let _ = (_a as fn(&dyn KeyValueStore), _h as fn(&dyn EventBus), _o as fn(&dyn TxRunner));
    let runner = FakeTxRunner;
    let n =
        run_tx_commit_on_ok(&runner, |_ctx| async move { Ok::<_, XError>(7u8) }).await.expect("tx");
    assert_eq!(n, 7);

    let bus = FakeEventBus::new();
    let fixture = FixtureNamespace::new("ctk_contracts_public_bus").expect("valid fixture");
    assert_event_bus(&bus, &fixture).await.expect("bus");
    assert_eq!(MessageAck::Ack, MessageAck::Ack);
    let _ = BusMessage { id: "1".into(), payload: Bytes::from_static(b"x") };
}

#[test]
fn trait_surface_object_safe_bounds() {
    fn assert_tx(_: &dyn TxRunner) {}
    assert_tx(&FakeTxRunner);
}

#[tokio::test]
async fn fake_key_value_store_public_surface() {
    let store = FakeKeyValueStore::new();
    let fixture = FixtureNamespace::new("ctk_contracts_public_key_value").expect("valid fixture");
    assert_key_value_store(&store, &fixture).await.expect("kv");
    assert_eq!(store.len().expect("len"), 1);
    assert!(!store.is_empty().expect("empty"));
    let _ = Duration::from_secs(1);
}

#[tokio::test]
async fn fake_repository_public_surface() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Entity {
        id: u64,
        name: String,
    }
    let repo = FakeRepository::new(|e: &Entity| e.id);
    let r: &dyn Repository<Entity, u64> = &repo;
    assert!(r.find(1).await.expect("find").is_none());
    r.save(&Entity { id: 1, name: "a".into() }).await.expect("save");
    let got = r.find(1).await.expect("find2").expect("entity");
    assert_eq!(got.name, "a");
    assert_eq!(repo.len().expect("len"), 1);
}

#[test]
fn recording_instrumentation_public_surface() {
    let rec = RecordingInstrumentation::new();
    let instr: &dyn Instrumentation = &rec;
    instr.record_retry("op", 1);
    instr.record_circuit_open("op");
    instr.record_circuit_close("op");
    let snap = rec.snapshot().expect("snap");
    assert_eq!(
        snap,
        vec![
            InstrEvent::Retry { op: "op".into(), attempt: 1 },
            InstrEvent::CircuitOpen { op: "op".into() },
            InstrEvent::CircuitClose { op: "op".into() },
        ]
    );
    rec.clear().expect("clear");
    assert!(rec.snapshot().expect("snap2").is_empty());
}
