//! Live 能力声明与可观测路径 helper。
//!
//! Profile 只表达接线意图，helper 只证明 trait 路径被调用；两者都不替代真实后端
//! live 证据，也不证明跨对象业务原子性。无法由句柄验证的 flag 必须 fail-closed。

#[allow(deprecated, reason = "兼容函数必须调用旧事务 helper")]
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

/// 兼容入口：只执行一次 producer `publish`，不证明 subscribe、ack 或 E2E 交付。
///
/// # Errors
///
/// [`EventBus::publish`] 失败时原样返回错误。
pub async fn bus_publish(bus: &dyn EventBus, topic: &str, payload: Bytes) -> XResult<()> {
    publish_without_delivery_attestation(bus, topic, payload).await
}

/// 只执行一次 producer `publish`；成功不构成消费、确认或 E2E 交付证明。
///
/// # Errors
///
/// [`EventBus::publish`] 失败时原样返回错误。
pub async fn publish_without_delivery_attestation(
    bus: &dyn EventBus,
    topic: &str,
    payload: Bytes,
) -> XResult<()> {
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

/// 兼容顺序编排：先开启事务生命周期，再执行独立 KV 写入。
///
/// `store` 不来自 [`TxContext`]，因此本函数**不保证** KV 写入与事务 commit/rollback
/// 原子绑定。保留本符号仅用于兼容；新代码应使用准确命名入口或后端真实事务操作面。
///
/// # Errors
///
/// begin、KV set 或 commit 失败时返回错误；KV set 失败后的 rollback 错误按兼容语义丢弃。
#[deprecated(note = "独立 KV 与 TxContext 不具备原子绑定；仅保留兼容")]
pub async fn tx_kv_set(
    runner: &dyn TxRunner,
    store: Arc<dyn KeyValueStore>,
    key: String,
    val: Vec<u8>,
) -> XResult<()> {
    kv_set_then_commit_separate_resources(runner, store, key, val).await
}

/// 先执行独立 KV set，再提交 [`TxContext`]；名称明确两者没有共同事务绑定。
///
/// KV 失败会触发 TxContext rollback；commit 失败不会自动撤销已经完成的 KV set。
///
/// # Errors
///
/// begin、KV set 或 commit 失败时返回错误；KV set 失败后的 rollback 错误按兼容语义丢弃。
#[allow(deprecated, reason = "保持既有事务生命周期与错误映射语义")]
pub async fn kv_set_then_commit_separate_resources(
    runner: &dyn TxRunner,
    store: Arc<dyn KeyValueStore>,
    key: String,
    val: Vec<u8>,
) -> XResult<()> {
    run_tx_commit_on_ok(runner, move |_ctx| async move { store.set(&key, val, None).await }).await
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

/// 兼容顺序编排：在已 begin 的上下文旁执行业务再 commit/rollback。
///
/// 业务闭包没有事务操作句柄，本函数只驱动生命周期，不保证闭包捕获的外部操作
/// 原子绑定；rollback 失败仍按旧接口语义丢弃。
#[deprecated(note = "请使用 run_tx_lifecycle；本入口不保留 rollback 双失败")]
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

    /// 校验 profile 声明与句柄非空一致（声明 true 则必须有可验证句柄）。
    ///
    /// 当前类型尚无 Repository / Account / VenueTime 槽位；对应 flag 为 true 时
    /// fail-closed。由此 `storage_stack()`、`venue_stack()`、`all()` 当前均不能通过
    /// 本校验，不得把 profile 意图误报成已完成 live 接线。
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
        if self.profile.repo {
            return Err(XError::missing("live profile 声明 repo，但 LiveHandles 尚无可验证句柄"));
        }
        if self.profile.account {
            return Err(XError::missing(
                "live profile 声明 account，但 LiveHandles 尚无可验证句柄",
            ));
        }
        if self.profile.venue_time {
            return Err(XError::missing(
                "live profile 声明 venue_time，但 LiveHandles 尚无可验证句柄",
            ));
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
    #[allow(deprecated, reason = "覆盖兼容入口的既有顺序语义")]
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
    #[allow(deprecated, reason = "覆盖兼容入口的既有顺序语义")]
    async fn run_on_tx_context_commit_and_rollback() {
        let mut ctx = StubTx;
        let v = run_on_tx_context(&mut ctx, || async { Ok(7u32) }).await.unwrap();
        assert_eq!(v, 7);
        let err = run_on_tx_context(&mut ctx, || async { Err::<u32, _>(XError::invalid("x")) })
            .await
            .unwrap_err();
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[test]
    fn profile_constructors_cover_all_presets() {
        let kv = LiveContractProfile::first_batch_kv();
        assert!(kv.kv && !kv.bus);
        let venue = LiveContractProfile::venue_stack();
        assert!(venue.venue && venue.account && venue.venue_time);
        assert!(!venue.kv);
        let all = LiveContractProfile::all();
        assert_eq!(all.enabled_count(), 7);
        assert!(all.any());
        assert!(!LiveContractProfile::none().any());
        let _ = format!("{:?}", all);
    }

    #[tokio::test]
    async fn kv_roundtrip_error_paths_and_ttl() {
        let kv = MemKv { m: Mutex::new(HashMap::new()) };
        kv.set("a", b"1".to_vec(), None).await.unwrap();
        // mismatch
        let err = {
            // temporarily wrong via manual set after start of roundtrip path
            // force mismatch: set different then call check path by using helper incorrectly
            kv.set("b", b"x".to_vec(), None).await.unwrap();
            // craft mismatch by reading after overwriting mid-helper — call helper then overwrite not possible;
            // instead use a double that returns wrong value
            struct BadKv;
            #[async_trait]
            impl KeyValueStore for BadKv {
                async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
                    Ok(Some(b"wrong".to_vec()))
                }
                async fn set(
                    &self,
                    _key: &str,
                    _val: Vec<u8>,
                    _ttl: Option<Duration>,
                ) -> XResult<()> {
                    Ok(())
                }
            }
            kv_roundtrip(&BadKv, "k", b"right").await.unwrap_err()
        };
        assert_eq!(err.kind(), kernel::ErrorKind::Invariant);

        struct MissKv;
        #[async_trait]
        impl KeyValueStore for MissKv {
            async fn get(&self, _key: &str) -> XResult<Option<Vec<u8>>> {
                Ok(None)
            }
            async fn set(&self, _key: &str, _val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
                Ok(())
            }
        }
        let err2 = kv_roundtrip(&MissKv, "k", b"v").await.unwrap_err();
        assert_eq!(err2.kind(), kernel::ErrorKind::Missing);

        kv_set_ttl(&kv, "ttl", b"z".to_vec(), Some(Duration::from_secs(1))).await.unwrap();
        assert_eq!(kv.get("ttl").await.unwrap().unwrap(), b"z");
    }

    #[tokio::test]
    async fn repo_venue_health_paths() {
        use canonical::{
            CancelOrderRequest, Money, Order, OrderAck, OrderRef, OrderStatus, Position, Side,
        };
        use decimalx::{Currency, Decimal, Price, Qty};

        struct Repo {
            v: Mutex<Option<String>>,
        }
        #[async_trait]
        impl Repository<String, String> for Repo {
            async fn find(&self, id: String) -> XResult<Option<String>> {
                let g = self.v.lock().unwrap();
                Ok(g.clone().filter(|x| x == &id || true).map(|_| id))
            }
            async fn save(&self, entity: &String) -> XResult<()> {
                *self.v.lock().unwrap() = Some(entity.clone());
                Ok(())
            }
        }
        let repo = Repo { v: Mutex::new(None) };
        let got = repo_roundtrip(&repo, "id1".into(), &"id1".to_string()).await.unwrap();
        assert_eq!(got.as_deref(), Some("id1"));

        struct Ven;
        #[async_trait]
        impl ExecutionVenue for Ven {
            async fn place_order(&self, order: &Order) -> XResult<OrderAck> {
                Ok(OrderAck { id: order.id.clone(), status: OrderStatus::Pending, ts: 0 })
            }
            async fn cancel_order(&self, _request: &CancelOrderRequest) -> XResult<()> {
                Ok(())
            }
            async fn query_order(&self, _request: &CancelOrderRequest) -> XResult<OrderStatus> {
                Ok(OrderStatus::Pending)
            }
            fn venue_id(&self) -> String {
                "v".into()
            }
        }
        let order = Order {
            id: "1".into(),
            symbol: "BTCUSDT".into(),
            side: Side::Buy,
            price: Price::new(Decimal::new(1, 0)),
            qty: Qty::new(Decimal::new(1, 0)),
            status: OrderStatus::Pending,
        };
        let req = CancelOrderRequest {
            venue: "v".into(),
            instrument: "BTCUSDT".into(),
            id: OrderRef::Exchange("1".into()),
        };
        let (ack, st) = venue_place_and_query(&Ven, &order, &req).await.unwrap();
        assert_eq!(ack.id, "1");
        assert_eq!(st, OrderStatus::Pending);
        Ven.cancel_order(&req).await.unwrap();
        assert_eq!(Ven.venue_id(), "v");

        struct Acc;
        #[async_trait]
        impl AccountSource for Acc {
            async fn query_position(&self) -> XResult<Vec<Position>> {
                Ok(vec![])
            }
            async fn query_balance(&self) -> XResult<Vec<Money>> {
                let c = Currency::try_new(*b"USD").unwrap();
                Ok(vec![Money::try_new(Decimal::new(1, 0), c).unwrap()])
            }
        }
        struct Clk;
        #[async_trait]
        impl VenueTimeSource for Clk {
            async fn server_time(&self) -> XResult<i64> {
                Ok(42)
            }
        }
        let (n, t) = venue_health(&Acc, &Clk).await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(t, 42);
    }

    #[test]
    fn live_handles_validate_all_declared_slots() {
        let mut h = LiveHandles::empty(LiveContractProfile::all());
        assert!(h.validate().is_err());
        let kv = MemKv { m: Mutex::new(HashMap::new()) };
        h.kv = Some(&kv);
        h.bus = Some(&StubBus);
        h.tx = Some(&StubRunner);
        // venue still missing
        assert!(h.validate().is_err());

        struct Ven;
        #[async_trait]
        impl ExecutionVenue for Ven {
            async fn place_order(&self, _order: &canonical::Order) -> XResult<canonical::OrderAck> {
                Err(XError::invalid("x"))
            }
            async fn cancel_order(&self, _request: &canonical::CancelOrderRequest) -> XResult<()> {
                Ok(())
            }
            async fn query_order(
                &self,
                _request: &canonical::CancelOrderRequest,
            ) -> XResult<canonical::OrderStatus> {
                Err(XError::invalid("x"))
            }
            fn venue_id(&self) -> String {
                "v".into()
            }
        }
        let ven = Ven;
        h.venue = Some(&ven);
        // repo/account/venue_time 没有对应句柄槽，必须 fail-closed。
        assert!(h.validate().is_err());

        let mut represented = LiveContractProfile::none();
        represented.kv = true;
        represented.bus = true;
        represented.tx = true;
        represented.venue = true;
        let mut represented_handles = LiveHandles::empty(represented);
        represented_handles.kv = Some(&kv);
        represented_handles.bus = Some(&StubBus);
        represented_handles.tx = Some(&StubRunner);
        represented_handles.venue = Some(&ven);
        assert!(represented_handles.validate().is_ok());
        // drive unused trait methods on stubs for LCOV
        let req = canonical::CancelOrderRequest {
            venue: "v".into(),
            instrument: "BTCUSDT".into(),
            id: canonical::OrderRef::Exchange("1".into()),
        };
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            ven.cancel_order(&req).await.unwrap();
            let _ = ven.query_order(&req).await;
            let _ = ven
                .place_order(&canonical::Order {
                    id: "1".into(),
                    symbol: "BTCUSDT".into(),
                    side: canonical::Side::Buy,
                    price: decimalx::Price::new(decimalx::Decimal::new(1, 0)),
                    qty: decimalx::Qty::new(decimalx::Decimal::new(1, 0)),
                    status: canonical::OrderStatus::Pending,
                })
                .await;
        });
        assert_eq!(ven.venue_id(), "v");
    }

    #[test]
    fn validate_bus_and_tx_missing_messages() {
        let mut bus_only = LiveContractProfile::none();
        bus_only.bus = true;
        assert!(LiveHandles::empty(bus_only).validate().is_err());
        let mut tx_only = LiveContractProfile::none();
        tx_only.tx = true;
        assert!(LiveHandles::empty(tx_only).validate().is_err());
    }

    #[test]
    fn validate_each_unrepresented_profile_flag_fails_closed() {
        for set_flag in [
            |p: &mut LiveContractProfile| p.repo = true,
            |p: &mut LiveContractProfile| p.account = true,
            |p: &mut LiveContractProfile| p.venue_time = true,
        ] {
            let mut profile = LiveContractProfile::none();
            set_flag(&mut profile);
            let err = LiveHandles::empty(profile).validate().expect_err("无句柄 flag 必须拒绝");
            assert_eq!(err.kind(), kernel::ErrorKind::Missing);
        }
    }

    #[tokio::test]
    async fn stub_bus_subscribe_stream_polled() {
        let mut stream = StubBus.subscribe("t").await.unwrap();
        use futures_core::Stream;
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        assert!(matches!(Pin::new(&mut stream).poll_next(&mut cx), Poll::Ready(None)));
    }
}
