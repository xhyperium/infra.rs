//! 进程内 `MockRedisAdapter`：带 **TTL 模拟** 的 KeyValueStore + PubSub。
//!
//! 与 scaffold [`crate::RedisAdapter`]（忽略 TTL）不同：
//! - `set(..., Some(ttl))` 后，过期键在 `get` 时返回 `None` 并惰性清理；
//! - PubSub 消息带单调递增 `BusMessage.id`。
//!
//! **非**真实 Redis 客户端；默认 `cargo test` 离线可跑。

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{BusMessage, KeyValueStore, PubSub};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// 带过期时间的条目。
#[derive(Debug, Clone)]
struct Entry {
    value: Vec<u8>,
    /// `None` = 永不过期。
    expires_at: Option<Instant>,
}

/// 进程内 mock Redis（TTL 模拟）。
pub struct MockRedisAdapter {
    name: String,
    kv: Mutex<HashMap<String, Entry>>,
    channels: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
    /// 可注入时钟（测试用）；`None` 时用 `Instant::now()`。
    now_override: Mutex<Option<Instant>>,
}

impl MockRedisAdapter {
    /// 新建空 mock。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kv: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            seq: AtomicU64::new(0),
            now_override: Mutex::new(None),
        }
    }

    /// 本地命名。
    pub fn local() -> Self {
        Self::new("mock-redis-local")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 测试钩子：固定“当前时间”（用于确定性 TTL 断言）。
    pub fn set_now_for_test(&self, now: Instant) {
        *self.now_override.lock().expect("now lock") = Some(now);
    }

    /// 测试钩子：清除固定时间，恢复系统时钟。
    pub fn clear_now_for_test(&self) {
        *self.now_override.lock().expect("now lock") = None;
    }

    fn now(&self) -> Instant {
        self.now_override.lock().expect("now lock").unwrap_or_else(Instant::now)
    }

    fn lock_kv(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Entry>>> {
        self.kv.lock().map_err(|e| XError::internal(format!("kv lock poisoned: {e}")))
    }

    fn lock_ch(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<BusMessage>>>> {
        self.channels.lock().map_err(|e| XError::internal(format!("pubsub lock poisoned: {e}")))
    }

    /// 惰性删除过期键；返回是否仍存活。
    fn is_alive(entry: &Entry, now: Instant) -> bool {
        match entry.expires_at {
            Some(exp) => exp > now,
            None => true,
        }
    }
}

#[async_trait]
impl KeyValueStore for MockRedisAdapter {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let now = self.now();
        let mut g = self.lock_kv()?;
        match g.get(key) {
            Some(e) if Self::is_alive(e, now) => Ok(Some(e.value.clone())),
            Some(_) => {
                // 过期：惰性清理
                g.remove(key);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        let now = self.now();
        let expires_at = ttl.map(|d| now + d);
        self.lock_kv()?.insert(key.to_string(), Entry { value: val, expires_at });
        Ok(())
    }
}

#[async_trait]
impl PubSub for MockRedisAdapter {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()> {
        let id = self.seq.fetch_add(1, Ordering::Relaxed).to_string();
        let bus_msg = BusMessage { id, payload: msg };
        self.lock_ch()?.entry(channel.to_string()).or_default().push(bus_msg);
        Ok(())
    }

    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = self.lock_ch()?.get(channel).cloned().unwrap_or_default();
        Ok(Box::pin(stream::iter(msgs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn kv_roundtrip_no_ttl() {
        let a = MockRedisAdapter::local();
        a.set("k", b"v".to_vec(), None).await.expect("set");
        assert_eq!(a.get("k").await.expect("get"), Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn ttl_expires_returns_none() {
        let a = MockRedisAdapter::local();
        let t0 = Instant::now();
        a.set_now_for_test(t0);
        a.set("k", b"v".to_vec(), Some(Duration::from_secs(10))).await.expect("set");
        // 未过期
        assert_eq!(a.get("k").await.expect("get"), Some(b"v".to_vec()));
        // 推进时钟越过 TTL
        a.set_now_for_test(t0 + Duration::from_secs(11));
        assert_eq!(a.get("k").await.expect("get expired"), None);
        // 惰性删除后仍为 None
        assert_eq!(a.get("k").await.expect("get again"), None);
    }

    #[tokio::test]
    async fn pubsub_monotonic_ids() {
        let a = MockRedisAdapter::local();
        a.pub_message("ch", Bytes::from_static(b"a")).await.expect("pub");
        a.pub_message("ch", Bytes::from_static(b"b")).await.expect("pub");
        let mut s = a.sub_channel("ch").await.expect("sub");
        let m1 = s.next().await.expect("m1");
        let m2 = s.next().await.expect("m2");
        assert_eq!(m1.id, "0");
        assert_eq!(m2.id, "1");
        assert_eq!(m1.payload.as_ref(), b"a");
        assert_eq!(m2.payload.as_ref(), b"b");
    }

    #[tokio::test]
    async fn dyn_key_value_store() {
        let a = MockRedisAdapter::local();
        let kv: &dyn KeyValueStore = &a;
        kv.set("x", b"1".to_vec(), None).await.expect("set");
        assert_eq!(kv.get("x").await.expect("get"), Some(b"1".to_vec()));
    }
}
