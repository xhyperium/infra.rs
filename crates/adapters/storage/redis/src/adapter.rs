//! Redis 内存 scaffold：`KeyValueStore` + `PubSub`。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use contracts::{KeyValueStore, PubSub};
use futures_core::stream::BoxStream;
use futures_util::stream;
use kernel::{XError, XResult};

/// Redis 适配器（进程内 HashMap；非真实客户端）。
pub struct RedisAdapter {
    name: String,
    endpoint: String,
    kv: Mutex<HashMap<String, Vec<u8>>>,
    channels: Mutex<HashMap<String, Vec<Bytes>>>,
}

impl RedisAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            kv: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
        }
    }

    pub fn local() -> Self {
        Self::new("redis-local", "redis://127.0.0.1:6379")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock_kv(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<u8>>>> {
        self.kv.lock().map_err(|e| XError::internal(format!("kv lock poisoned: {e}")))
    }

    fn lock_ch(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Vec<Bytes>>>> {
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
        self.lock_ch()?.entry(channel.to_string()).or_default().push(msg);
        Ok(())
    }

    async fn sub_channel(&self, channel: &str) -> XResult<BoxStream<'static, Bytes>> {
        let msgs = self.lock_ch()?.get(channel).cloned().unwrap_or_default();
        Ok(Box::pin(stream::iter(msgs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn kv_roundtrip() {
        let a = RedisAdapter::local();
        a.set("k", b"v".to_vec(), None).await.expect("set");
        assert_eq!(a.get("k").await.expect("get"), Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn pubsub_replay() {
        let a = RedisAdapter::local();
        a.pub_message("ch", Bytes::from_static(b"m1")).await.expect("pub");
        let mut s = a.sub_channel("ch").await.expect("sub");
        assert_eq!(s.next().await, Some(Bytes::from_static(b"m1")));
    }

    #[test]
    fn name_endpoint() {
        let a = RedisAdapter::local();
        assert_eq!(a.name(), "redis-local");
        assert_eq!(a.endpoint(), "redis://127.0.0.1:6379");
    }
}
