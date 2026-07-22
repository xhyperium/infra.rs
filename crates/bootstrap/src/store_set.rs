//! StoreSet —— 组合根对 storage / venue 适配器的类型化接线面。
//!
//! 关闭「StoreSet/adapter 未接线」DEFER：在 build 期注入可选适配器句柄，
//! 运行期通过 [`StoreSet`] 只读访问，禁止动态 register/resolve。

use crate::traits::{
    BoundedAccountSource, BoundedExecutionVenue, BoundedInstrumentCatalog, BoundedKeyValueStore,
    BoundedMarketDataSource, BoundedVenueTimeSource,
};
use std::sync::Arc;

/// 已接线的适配器集合（全部可选；未注入为 `None`）。
///
/// 字段为有界对象安全替面，可包装真实 adapter 或测试 double。
#[derive(Clone, Default)]
pub struct StoreSet {
    kv: Option<Arc<dyn BoundedKeyValueStore>>,
    market_data: Option<Arc<dyn BoundedMarketDataSource>>,
    catalog: Option<Arc<dyn BoundedInstrumentCatalog>>,
    venue: Option<Arc<dyn BoundedExecutionVenue>>,
    account: Option<Arc<dyn BoundedAccountSource>>,
    venue_time: Option<Arc<dyn BoundedVenueTimeSource>>,
}

impl StoreSet {
    /// 空集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否至少接线了一个适配器。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kv.is_none()
            && self.market_data.is_none()
            && self.catalog.is_none()
            && self.venue.is_none()
            && self.account.is_none()
            && self.venue_time.is_none()
    }

    /// 已接线槽位数。
    #[must_use]
    pub fn wired_count(&self) -> usize {
        [
            self.kv.is_some(),
            self.market_data.is_some(),
            self.catalog.is_some(),
            self.venue.is_some(),
            self.account.is_some(),
            self.venue_time.is_some(),
        ]
        .into_iter()
        .filter(|x| *x)
        .count()
    }

    /// 注入 KV。
    #[must_use]
    pub fn with_kv(mut self, store: Arc<dyn BoundedKeyValueStore>) -> Self {
        self.kv = Some(store);
        self
    }

    /// 注入行情源。
    #[must_use]
    pub fn with_market_data(mut self, source: Arc<dyn BoundedMarketDataSource>) -> Self {
        self.market_data = Some(source);
        self
    }

    /// 注入标的目录。
    #[must_use]
    pub fn with_catalog(mut self, catalog: Arc<dyn BoundedInstrumentCatalog>) -> Self {
        self.catalog = Some(catalog);
        self
    }

    /// 注入执行场所。
    #[must_use]
    pub fn with_venue(mut self, venue: Arc<dyn BoundedExecutionVenue>) -> Self {
        self.venue = Some(venue);
        self
    }

    /// 注入账户源。
    #[must_use]
    pub fn with_account(mut self, account: Arc<dyn BoundedAccountSource>) -> Self {
        self.account = Some(account);
        self
    }

    /// 注入场所时间。
    #[must_use]
    pub fn with_venue_time(mut self, time: Arc<dyn BoundedVenueTimeSource>) -> Self {
        self.venue_time = Some(time);
        self
    }

    /// KV 只读访问。
    #[must_use]
    pub fn kv(&self) -> Option<&dyn BoundedKeyValueStore> {
        self.kv.as_deref()
    }

    /// 行情源。
    #[must_use]
    pub fn market_data(&self) -> Option<&dyn BoundedMarketDataSource> {
        self.market_data.as_deref()
    }

    /// 标的目录。
    #[must_use]
    pub fn catalog(&self) -> Option<&dyn BoundedInstrumentCatalog> {
        self.catalog.as_deref()
    }

    /// 执行场所。
    #[must_use]
    pub fn venue(&self) -> Option<&dyn BoundedExecutionVenue> {
        self.venue.as_deref()
    }

    /// 账户源。
    #[must_use]
    pub fn account(&self) -> Option<&dyn BoundedAccountSource> {
        self.account.as_deref()
    }

    /// 场所时间。
    #[must_use]
    pub fn venue_time(&self) -> Option<&dyn BoundedVenueTimeSource> {
        self.venue_time.as_deref()
    }

    /// 克隆 KV 句柄（供有界上下文构造）。
    #[must_use]
    pub fn kv_arc(&self) -> Option<Arc<dyn BoundedKeyValueStore>> {
        self.kv.clone()
    }

    /// 克隆行情源句柄。
    #[must_use]
    pub fn market_data_arc(&self) -> Option<Arc<dyn BoundedMarketDataSource>> {
        self.market_data.clone()
    }

    /// 克隆目录句柄。
    #[must_use]
    pub fn catalog_arc(&self) -> Option<Arc<dyn BoundedInstrumentCatalog>> {
        self.catalog.clone()
    }

    /// 克隆场所句柄。
    #[must_use]
    pub fn venue_arc(&self) -> Option<Arc<dyn BoundedExecutionVenue>> {
        self.venue.clone()
    }

    /// 克隆账户句柄。
    #[must_use]
    pub fn account_arc(&self) -> Option<Arc<dyn BoundedAccountSource>> {
        self.account.clone()
    }

    /// 克隆时间句柄。
    #[must_use]
    pub fn venue_time_arc(&self) -> Option<Arc<dyn BoundedVenueTimeSource>> {
        self.venue_time.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{
        BoundedAccountSource, BoundedExecutionVenue, BoundedInstrumentCatalog,
        BoundedKeyValueStore, BoundedMarketDataSource, BoundedVenueTimeSource,
    };

    struct Stub;
    impl BoundedKeyValueStore for Stub {
        fn label(&self) -> &str {
            "kv"
        }
    }
    impl BoundedMarketDataSource for Stub {
        fn label(&self) -> &str {
            "md"
        }
    }
    impl BoundedInstrumentCatalog for Stub {
        fn label(&self) -> &str {
            "cat"
        }
    }
    impl BoundedExecutionVenue for Stub {
        fn venue_id(&self) -> &str {
            "v"
        }
    }
    impl BoundedAccountSource for Stub {
        fn label(&self) -> &str {
            "a"
        }
    }
    impl BoundedVenueTimeSource for Stub {
        fn label(&self) -> &str {
            "t"
        }
    }

    #[test]
    fn store_set_wires_all_slots() {
        let s = Arc::new(Stub);
        let set = StoreSet::new()
            .with_kv(Arc::clone(&s) as Arc<dyn BoundedKeyValueStore>)
            .with_market_data(Arc::clone(&s) as Arc<dyn BoundedMarketDataSource>)
            .with_catalog(Arc::clone(&s) as Arc<dyn BoundedInstrumentCatalog>)
            .with_venue(Arc::clone(&s) as Arc<dyn BoundedExecutionVenue>)
            .with_account(Arc::clone(&s) as Arc<dyn BoundedAccountSource>)
            .with_venue_time(Arc::clone(&s) as Arc<dyn BoundedVenueTimeSource>);
        assert!(!set.is_empty());
        assert_eq!(set.wired_count(), 6);
        assert_eq!(set.kv().expect("kv").label(), "kv");
        assert_eq!(set.market_data().expect("md").label(), "md");
        assert_eq!(set.catalog().expect("cat").label(), "cat");
        assert_eq!(set.venue().expect("v").venue_id(), "v");
        assert_eq!(set.account().expect("a").label(), "a");
        assert_eq!(set.venue_time().expect("t").label(), "t");
        assert!(set.kv_arc().is_some());
        assert!(set.market_data_arc().is_some());
        assert!(set.catalog_arc().is_some());
        assert!(set.venue_arc().is_some());
        assert!(set.account_arc().is_some());
        assert!(set.venue_time_arc().is_some());
    }

    #[test]
    fn empty_store_set() {
        let set = StoreSet::new();
        assert!(set.is_empty());
        assert_eq!(set.wired_count(), 0);
        assert!(set.kv().is_none());
        assert!(set.market_data_arc().is_none());
        assert!(set.catalog_arc().is_none());
        assert!(set.account_arc().is_none());
        assert!(set.venue_time_arc().is_none());
    }
}
