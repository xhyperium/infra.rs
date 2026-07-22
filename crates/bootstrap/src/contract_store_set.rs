//! 正式 storage contracts 的类型化接线面。
//!
//! 本类型与历史 [`crate::StoreSet`] 并存：后者只承载 `Bounded*` 诊断替面，
//! 本类型承载可实际调用的 [`contracts`] trait object。字段是固定槽位，禁止动态
//! `register` / `resolve`、字符串 key、`Any` 或 `TypeId`。

use contracts::{EventBus, KeyValueStore};
use std::sync::Arc;

/// 已接线的正式 storage contracts。
///
/// 本轮只冻结 KV 与 at-most-once EventBus 两个最小生产 seam。泛型
/// `Repository<T, Id>` 必须由业务组合根以具体类型注入，不进入全局容器；多个
/// 独立槽位同时存在也**不**表示具备跨资源事务。
#[derive(Clone, Default)]
pub struct ContractStoreSet {
    kv: Option<Arc<dyn KeyValueStore>>,
    event_bus: Option<Arc<dyn EventBus>>,
}

impl ContractStoreSet {
    /// 构造空集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否尚未接线任何正式 contract。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kv.is_none() && self.event_bus.is_none()
    }

    /// 已接线的正式 contract 数量。
    #[must_use]
    pub fn wired_count(&self) -> usize {
        usize::from(self.kv.is_some()) + usize::from(self.event_bus.is_some())
    }

    /// 注入 [`KeyValueStore`]。
    #[must_use]
    pub fn with_kv(mut self, store: Arc<dyn KeyValueStore>) -> Self {
        self.kv = Some(store);
        self
    }

    /// 注入 [`EventBus`]。
    #[must_use]
    pub fn with_event_bus(mut self, bus: Arc<dyn EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// 访问 KV contract。
    #[must_use]
    pub fn kv(&self) -> Option<&dyn KeyValueStore> {
        self.kv.as_deref()
    }

    /// 访问 EventBus contract。
    #[must_use]
    pub fn event_bus(&self) -> Option<&dyn EventBus> {
        self.event_bus.as_deref()
    }

    /// 克隆 KV trait object。
    #[must_use]
    pub fn kv_arc(&self) -> Option<Arc<dyn KeyValueStore>> {
        self.kv.clone()
    }

    /// 克隆 EventBus trait object。
    #[must_use]
    pub fn event_bus_arc(&self) -> Option<Arc<dyn EventBus>> {
        self.event_bus.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use bytes::Bytes;
    use futures_core::stream::BoxStream;
    use kernel::XResult;

    struct Probe;

    #[async_trait]
    impl KeyValueStore for Probe {
        async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
            Ok(Some(b"value".to_vec()))
        }

        async fn set(
            &self,
            _key: &str,
            _val: Vec<u8>,
            _ttl: Option<std::time::Duration>,
        ) -> XResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl EventBus for Probe {
        async fn publish(&self, _topic: &str, _payload: Bytes) -> XResult<()> {
            Ok(())
        }

        async fn subscribe(
            &self,
            _topic: &str,
        ) -> XResult<BoxStream<'static, contracts::BusMessage>> {
            Ok(Box::pin(futures_util::stream::empty()))
        }
    }

    #[tokio::test]
    async fn wires_callable_contracts_without_locator() {
        let probe = Arc::new(Probe);
        let kv: Arc<dyn KeyValueStore> = probe.clone();
        let event_bus: Arc<dyn EventBus> = probe;
        let set = ContractStoreSet::new().with_kv(kv).with_event_bus(event_bus);
        assert!(!set.is_empty());
        assert_eq!(set.wired_count(), 2);
        set.kv().expect("kv contract").set("key", b"value".to_vec(), None).await.expect("set");
        assert_eq!(
            set.kv().expect("kv contract").get("key").await.expect("get"),
            Some(b"value".to_vec())
        );
        set.event_bus()
            .expect("event bus contract")
            .publish("topic", Bytes::new())
            .await
            .expect("publish");
        assert!(set.kv_arc().is_some());
        assert!(set.event_bus_arc().is_some());
    }

    #[test]
    fn empty_contract_store_set() {
        let set = ContractStoreSet::new();
        assert!(set.is_empty());
        assert_eq!(set.wired_count(), 0);
        assert!(set.kv().is_none());
        assert!(set.event_bus().is_none());
    }
}
