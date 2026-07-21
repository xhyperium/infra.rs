#![allow(dead_code)]
use contracts::{
    AccountSource, AnalyticsSink, EventBus, ExecutionVenue, InstrumentCatalog, Instrumentation,
    KeyValueStore, MarketDataSource, ObjectStore, PubSub, Repository, TimeSeriesStore, TxRunner,
    VenueAdapter, VenueTimeSource,
};
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
fn _o<R: TxRunner>() {}
#[test]
fn fifteen_traits_are_reachable() {
    assert_eq!(15, 15);
}
