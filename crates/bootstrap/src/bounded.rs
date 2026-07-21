//! 有界上下文（PLAN-GATE-RETIRE-001 §4 / DEFER-BOUND-CTX）。
//!
//! 服务只拿所需依赖；**不**把整个 [`AppContext`](crate::AppContext) 传遍所有层。
//! 字段在构造后不可替换；无动态注册 API。

use crate::PlatformContext;
use crate::traits::{
    AccountSource, ExecutionVenue, InstrumentCatalog, KeyValueStore, MarketDataSource,
    VenueTimeSource,
};
use std::sync::Arc;

/// 行情服务有界上下文（最小可用字段集）。
pub struct MarketDataContext {
    source: Arc<dyn MarketDataSource>,
    catalog: Arc<dyn InstrumentCatalog>,
    kv: Arc<dyn KeyValueStore>,
    platform: PlatformContext,
}

impl MarketDataContext {
    /// 构造（全部字段必须在组合边界注入）。
    pub fn new(
        source: Arc<dyn MarketDataSource>,
        catalog: Arc<dyn InstrumentCatalog>,
        kv: Arc<dyn KeyValueStore>,
        platform: PlatformContext,
    ) -> Self {
        Self { source, catalog, kv, platform }
    }

    /// 行情源。
    pub fn source(&self) -> &dyn MarketDataSource {
        self.source.as_ref()
    }

    /// 标的目录。
    pub fn catalog(&self) -> &dyn InstrumentCatalog {
        self.catalog.as_ref()
    }

    /// KV 存储。
    pub fn kv_store(&self) -> &dyn KeyValueStore {
        self.kv.as_ref()
    }

    /// 横切平台依赖。
    pub fn platform(&self) -> &PlatformContext {
        &self.platform
    }
}

/// 执行服务有界上下文（最小可用字段集）。
pub struct ExecutionContext {
    venue: Arc<dyn ExecutionVenue>,
    account: Arc<dyn AccountSource>,
    time: Arc<dyn VenueTimeSource>,
    platform: PlatformContext,
}

impl ExecutionContext {
    /// 构造。
    pub fn new(
        venue: Arc<dyn ExecutionVenue>,
        account: Arc<dyn AccountSource>,
        time: Arc<dyn VenueTimeSource>,
        platform: PlatformContext,
    ) -> Self {
        Self { venue, account, time, platform }
    }

    /// 执行场所。
    pub fn venue(&self) -> &dyn ExecutionVenue {
        self.venue.as_ref()
    }

    /// 账户源。
    pub fn account(&self) -> &dyn AccountSource {
        self.account.as_ref()
    }

    /// 场所时间。
    pub fn time(&self) -> &dyn VenueTimeSource {
        self.time.as_ref()
    }

    /// 横切平台依赖。
    pub fn platform(&self) -> &PlatformContext {
        &self.platform
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Bootstrap;

    struct Stub;
    impl MarketDataSource for Stub {
        fn label(&self) -> &str {
            "md"
        }
    }
    impl InstrumentCatalog for Stub {
        fn label(&self) -> &str {
            "cat"
        }
    }
    impl KeyValueStore for Stub {
        fn label(&self) -> &str {
            "kv"
        }
    }
    impl ExecutionVenue for Stub {
        fn venue_id(&self) -> &str {
            "venue-a"
        }
    }
    impl AccountSource for Stub {
        fn label(&self) -> &str {
            "acct"
        }
    }
    impl VenueTimeSource for Stub {
        fn label(&self) -> &str {
            "clock"
        }
    }

    #[test]
    fn market_data_context_accessors() {
        let platform = Bootstrap::new().build().platform_cloned();
        let s = Arc::new(Stub);
        let mdx = MarketDataContext::new(
            Arc::clone(&s) as Arc<dyn MarketDataSource>,
            Arc::clone(&s) as Arc<dyn InstrumentCatalog>,
            Arc::clone(&s) as Arc<dyn KeyValueStore>,
            platform,
        );
        assert_eq!(mdx.source().label(), "md");
        assert_eq!(mdx.catalog().label(), "cat");
        assert_eq!(mdx.kv_store().label(), "kv");
        assert!(!mdx.platform().shutdown_signal().is_triggered());
        mdx.platform().instrumentation().record_retry("mdx", 1);
    }

    #[test]
    fn execution_context_accessors() {
        let platform = Bootstrap::new().build().platform_cloned();
        let s = Arc::new(Stub);
        let ex = ExecutionContext::new(
            Arc::clone(&s) as Arc<dyn ExecutionVenue>,
            Arc::clone(&s) as Arc<dyn AccountSource>,
            Arc::clone(&s) as Arc<dyn VenueTimeSource>,
            platform,
        );
        assert_eq!(ex.venue().venue_id(), "venue-a");
        assert_eq!(ex.account().label(), "acct");
        assert_eq!(ex.time().label(), "clock");
        assert!(!ex.platform().shutdown_signal().is_triggered());
    }
}
