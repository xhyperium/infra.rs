//! 进程内 scaffold / mock（feature `scaffold`）。
//!
//! **禁止**在生产路径依赖本模块；仅用于离线测试与迁移。

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

/// 进程内 HashMap Redis 桩（**忽略 TTL**）。
///
/// 亦可通过别名 [`InMemoryRedis`] 引用。
pub struct RedisAdapter {
    name: String,
    endpoint: String,
    kv: Mutex<HashMap<String, Vec<u8>>>,
    channels: Mutex<HashMap<String, Vec<BusMessage>>>,
}

/// 语义更清晰的 scaffold 别名。
pub type InMemoryRedis = RedisAdapter;

impl RedisAdapter {
    /// 新建 scaffold。
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            kv: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
        }
    }

    /// 本地默认。
    pub fn local() -> Self {
        Self::new("redis-local", "redis://127.0.0.1:6379")
    }

    /// 名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 端点（占位字符串）。
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock_kv(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<u8>>>> {
        self.kv.lock().map_err(|e| XError::internal(format!("kv lock poisoned: {e}")))
    }

    fn lock_ch(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<BusMessage>>>> {
        self.channels.lock().map_err(|e| XError::internal(format!("pubsub lock poisoned: {e}")))
    }
}

#[async_trait]
impl KeyValueStore for RedisAdapter {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        Ok(self.lock_kv()?.get(key).cloned())
    }

    async fn set(&self, key: &str, val: Vec<u8>, _ttl: Option<Duration>) -> XResult<()> {
        // scaffold：忽略 TTL
        self.lock_kv()?.insert(key.to_string(), val);
        Ok(())
    }
}

#[async_trait]
impl PubSub for RedisAdapter {
    async fn pub_message(&self, channel: &str, msg: Bytes) -> XResult<()> {
        let n = self.lock_ch()?.get(channel).map(|v| v.len()).unwrap_or(0);
        let bus_msg = BusMessage { id: n.to_string(), payload: msg };
        self.lock_ch()?.entry(channel.to_string()).or_default().push(bus_msg);
        Ok(())
    }

    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, BusMessage>> {
        let msgs = self.lock_ch()?.get(channel).cloned().unwrap_or_default();
        Ok(Box::pin(stream::iter(msgs)))
    }
}

/// 带过期时间的条目。
#[derive(Debug, Clone)]
struct Entry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

/// 进程内 mock Redis（TTL 模拟；**非**真实客户端）。
pub struct MockRedisAdapter {
    name: String,
    kv: Mutex<HashMap<String, Entry>>,
    channels: Mutex<HashMap<String, Vec<BusMessage>>>,
    seq: AtomicU64,
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

    /// 测试钩子：固定“当前时间”。
    pub fn set_now_for_test(&self, now: Instant) {
        *self.now_override.lock().expect("now lock") = Some(now);
    }

    /// 测试钩子：清除固定时间。
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
    async fn scaffold_kv_roundtrip() {
        let a = RedisAdapter::local();
        a.set("k", b"v".to_vec(), None).await.expect("set");
        assert_eq!(a.get("k").await.expect("get"), Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn in_memory_alias() {
        let a: InMemoryRedis = InMemoryRedis::local();
        assert_eq!(a.name(), "redis-local");
    }

    #[tokio::test]
    async fn mock_ttl_expires() {
        let a = MockRedisAdapter::local();
        let t0 = Instant::now();
        a.set_now_for_test(t0);
        a.set("k", b"v".to_vec(), Some(Duration::from_secs(10))).await.expect("set");
        assert_eq!(a.get("k").await.expect("get"), Some(b"v".to_vec()));
        a.set_now_for_test(t0 + Duration::from_secs(11));
        assert_eq!(a.get("k").await.expect("expired"), None);
    }

    #[tokio::test]
    async fn mock_pubsub_ids() {
        let a = MockRedisAdapter::local();
        a.pub_message("ch", Bytes::from_static(b"a")).await.expect("pub");
        let mut s = a.sub_channel("ch").await.expect("sub");
        let m = s.next().await.expect("msg");
        assert_eq!(m.id, "0");
    }
}
