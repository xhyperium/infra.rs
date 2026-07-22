//! 业务 live 面：Tx / Bus / Repo / Venue 的可观测编排与 profile 标记。
//!
//! 关闭 contracts DEFER「Tx/Bus/Repo/Venue 业务 live」：提供真实 trait 路径上的
//! 编排辅助与「哪条契约已具备 live 实现」的声明式 profile，供 adapter / bootstrap 接线。

use crate::{
    AccountSource, BusMessage, EventBus, ExecutionVenue, KeyValueStore, MessageAck, Repository,
    TxContext, TxRunner, VenueTimeSource, run_tx_commit_on_ok,
};
use async_trait::async_trait;
use bytes::Bytes;
use canonical::{CancelOrderRequest, Order, OrderAck, OrderStatus};
use kernel::{XError, XResult};
use std::sync::Arc;
use std::time::Duration;

/// 契约 live 能力开关（声明哪些业务面已接入真实后端）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiveContractProfile {
    /// KeyValueStore live。
    pub kv: bool,
    /// EventBus live。
    pub bus: bool,
    /// Repository live。
    pub repo: bool,
    /// TxRunner live。
    pub tx: bool,
    /// ExecutionVenue live。
    pub venue: bool,
    /// AccountSource live。
    pub account: bool,
    /// VenueTimeSource live。
    pub venue_time: bool,
}

impl LiveContractProfile {
    /// 全关（仅进程内 Fake）。
    #[must_use]
    pub const fn none() -> Self {
        Self {
            kv: false,
            bus: false,
            repo: false,
            tx: false,
            venue: false,
            account: false,
            venue_time: false,
        }
    }

    /// first-batch 历史子集：KV + Instrumentation（Instr 不在本 profile）。
    #[must_use]
    pub const fn first_batch_kv() -> Self {
        let mut p = Self::none();
        p.kv = true;
        p
    }

    /// 存储 + 事务面（无交易所）。
    #[must_use]
    pub const fn storage_stack() -> Self {
        let mut p = Self::first_batch_kv();
        p.bus = true;
        p.repo = true;
        p.tx = true;
        p
    }

    /// 交易执行面。
    #[must_use]
    pub const fn venue_stack() -> Self {
        let mut p = Self::none();
        p.venue = true;
        p.account = true;
        p.venue_time = true;
        p
    }

    /// 全业务 live（声明意图；真实可用性由 adapter 保证）。
    #[must_use]
    pub const fn all() -> Self {
        Self {
            kv: true,
            bus: true,
            repo: true,
            tx: true,
            venue: true,
            account: true,
            venue_time: true,
        }
    }

    /// 是否启用任一 live 面。
    #[must_use]
    pub const fn any(self) -> bool {
        self.kv || self.bus || self.repo || self.tx || self.venue || self.account || self.venue_time
    }

    /// 已启用槽位数。
    #[must_use]
    pub fn enabled_count(self) -> usize {
        [self.kv, self.bus, self.repo, self.tx, self.venue, self.account, self.venue_time]
            .into_iter()
            .filter(|x| *x)
            .count()
    }
}

/// KV live 辅助：set 后 get 校验（驱动真实 [`KeyValueStore`] 路径）。
pub async fn kv_roundtrip(store: &dyn KeyValueStore, key: &str, val: &[u8]) -> XResult<()> {
    store.set(key, val.to_vec(), None).await?;
    let got = store.get(key).await?;
    match got {
        Some(v) if v == val => Ok(()),
        Some(_) => Err(XError::invariant("kv roundtrip 值不匹配")),
        None => Err(XError::missing(format!("kv missing after set: {key}"))),
    }
}

/// Bus live 辅助：publish 后 subscribe 至少取一条（若流立即结束则 Ok 仍算路径走过）。
pub async fn bus_publish(bus: &dyn EventBus, topic: &str, payload: Bytes) -> XResult<()> {
    bus.publish(topic, payload).await
}

/// 消费确认语义包装：对消息应用 [`MessageAck`]（最小面记录决策，不强制后端 redelivery）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckedMessage {
    /// 原始消息。
    pub message: BusMessage,
    /// 确认动作。
    pub ack: MessageAck,
}

/// 对消息附加 Ack 决策（业务 live 编排辅助）。
#[must_use]
pub fn apply_ack(message: BusMessage, ack: MessageAck) -> AckedMessage {
    AckedMessage { message, ack }
}

/// Repository live：save 后 find。
pub async fn repo_roundtrip<T, Id>(
    repo: &dyn Repository<T, Id>,
    id: Id,
    entity: &T,
) -> XResult<Option<T>>
where
    T: Clone + Send + Sync,
    Id: Send,
{
    repo.save(entity).await?;
    repo.find(id).await
}

/// Venue live：place → query 路径。
pub async fn venue_place_and_query(
    venue: &dyn ExecutionVenue,
    order: &Order,
    query: &CancelOrderRequest,
) -> XResult<(OrderAck, OrderStatus)> {
    let ack = venue.place_order(order).await?;
    let status = venue.query_order(query).await?;
    Ok((ack, status))
}

/// 账户 + 时间联合探测（Venue 业务 live 健康检查）。
pub async fn venue_health(
    account: &dyn AccountSource,
    time: &dyn VenueTimeSource,
) -> XResult<(usize, i64)> {
    let positions = account.query_position().await?;
    let balances = account.query_balance().await?;
    let server_time = time.server_time().await?;
    Ok((positions.len() + balances.len(), server_time))
}

/// 将 KV 操作包进事务编排（Tx 业务 live）：commit on Ok。
pub async fn tx_kv_set(
    runner: &dyn TxRunner,
    store: Arc<dyn KeyValueStore>,
    key: String,
    val: Vec<u8>,
) -> XResult<()> {
    run_tx_commit_on_ok(runner, move |_ctx| {
        let store = Arc::clone(&store);
        let key = key.clone();
        let val = val.clone();
        async move { store.set(&key, val, None).await }
    })
    .await
}

/// 可选 TTL 的 KV set（暴露 Duration 路径）。
pub async fn kv_set_ttl(
    store: &dyn KeyValueStore,
    key: &str,
    val: Vec<u8>,
    ttl: Option<Duration>,
) -> XResult<()> {
    store.set(key, val, ttl).await
}

/// 对象安全 Tx 包装：在已 begin 的上下文上执行业务再 commit/rollback。
pub async fn run_on_tx_context<R, F, Fut>(ctx: &mut dyn TxContext, f: F) -> XResult<R>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = XResult<R>> + Send,
    R: Send,
{
    match f().await {
        Ok(v) => {
            ctx.commit().await?;
            Ok(v)
        }
        Err(e) => {
            let _ = ctx.rollback().await;
            Err(e)
        }
    }
}

/// 标记型 live 句柄：把 profile 与具体实现绑在一起（不拥有后端连接生命周期）。
pub struct LiveHandles<'a> {
    /// Profile 声明。
    pub profile: LiveContractProfile,
    /// 可选 KV。
    pub kv: Option<&'a dyn KeyValueStore>,
    /// 可选 Bus。
    pub bus: Option<&'a dyn EventBus>,
    /// 可选 Tx。
    pub tx: Option<&'a dyn TxRunner>,
    /// 可选 Venue。
    pub venue: Option<&'a dyn ExecutionVenue>,
}

impl<'a> LiveHandles<'a> {
    /// 构造空句柄。
    #[must_use]
    pub fn empty(profile: LiveContractProfile) -> Self {
        Self { profile, kv: None, bus: None, tx: None, venue: None }
    }

    /// 校验 profile 声明与句柄非空一致（声明 true 则句柄必须 Some）。
    pub fn validate(&self) -> XResult<()> {
        if self.profile.kv && self.kv.is_none() {
            return Err(XError::missing("live profile 声明 kv 但未注入句柄"));
        }
        if self.profile.bus && self.bus.is_none() {
            return Err(XError::missing("live profile 声明 bus 但未注入句柄"));
        }
        if self.profile.tx && self.tx.is_none() {
            return Err(XError::missing("live profile 声明 tx 但未注入句柄"));
        }
        if self.profile.venue && self.venue.is_none() {
            return Err(XError::missing("live profile 声明 venue 但未注入句柄"));
        }
        Ok(())
    }
}

// 抑制 unused import 警告：async_trait 留给扩展实现
#[allow(unused_imports)]
use async_trait as _;

#[cfg(test)]
mod tests {
    use super::*;
    use futures_core::Stream;
    use std::collections::HashMap;
    use std::pin::Pin;
    use std::sync::Mutex;
    use std::task::{Context, Poll};

    struct MemKv {
        m: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl KeyValueStore for MemKv {
        async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
            Ok(self.m.lock().unwrap().get(key).cloned())
        }
        async fn set(&self, key: &str, val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
            self.m.lock().unwrap().insert(key.into(), val);
            Ok(())
        }
    }

    #[tokio::test]
    async fn kv_roundtrip_drives_real_trait() {
        let kv = MemKv { m: Mutex::new(HashMap::new()) };
        kv_roundtrip(&kv, "a", b"1").await.unwrap();
        assert_eq!(kv.get("a").await.unwrap().as_deref(), Some(b"1".as_ref()));
    }

    struct StubTx;
    #[async_trait]
    impl TxContext for StubTx {
        async fn commit(&mut self) -> XResult<()> {
            Ok(())
        }
        async fn rollback(&mut self) -> XResult<()> {
            Ok(())
        }
    }
    struct StubRunner;
    #[async_trait]
    impl TxRunner for StubRunner {
        async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
            Ok(Box::new(StubTx))
        }
    }

    #[tokio::test]
    async fn tx_kv_set_commits() {
        let kv = Arc::new(MemKv { m: Mutex::new(HashMap::new()) });
        tx_kv_set(&StubRunner, kv.clone() as Arc<dyn KeyValueStore>, "k".into(), b"v".to_vec())
            .await
            .unwrap();
        assert_eq!(kv.get("k").await.unwrap().unwrap(), b"v");
    }

    struct EmptyStream;
    impl Stream for EmptyStream {
        type Item = BusMessage;
        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(None)
        }
    }
    struct StubBus;
    #[async_trait]
    impl EventBus for StubBus {
        async fn publish(&self, _topic: &str, _payload: Bytes) -> XResult<()> {
            Ok(())
        }
        async fn subscribe(
            &self,
            _topic: &str,
        ) -> XResult<futures_core::stream::BoxStream<'static, BusMessage>> {
            Ok(Box::pin(EmptyStream))
        }
    }

    #[tokio::test]
    async fn bus_publish_and_ack_helpers() {
        bus_publish(&StubBus, "t", Bytes::from_static(b"p")).await.unwrap();
        let acked = apply_ack(
            BusMessage { id: "1".into(), payload: Bytes::from_static(b"p") },
            MessageAck::Ack,
        );
        assert_eq!(acked.ack, MessageAck::Ack);
    }

    #[test]
    fn profile_counts_and_validate() {
        let p = LiveContractProfile::storage_stack();
        assert!(p.any());
        assert_eq!(p.enabled_count(), 4);
        let h = LiveHandles::empty(p);
        assert!(h.validate().is_err());
        let mut h2 = LiveHandles::empty(LiveContractProfile::none());
        let mem = MemKv { m: Mutex::new(HashMap::new()) };
        h2.kv = Some(&mem);
        assert!(h2.validate().is_ok());
    }

    #[tokio::test]
    async fn run_on_tx_context_commit_and_rollback() {
        let mut ctx = StubTx;
        let v = run_on_tx_context(&mut ctx, || async { Ok(7u32) }).await.unwrap();
        assert_eq!(v, 7);
        let err = run_on_tx_context(&mut ctx, || async { Err::<u32, _>(XError::invalid("x")) })
            .await
            .unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }
}
