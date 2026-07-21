//! 真实 Redis `KeyValueStore` 入口（feature `live`，infra-s9t.2）。
//!
//! 默认 **不** 启用；CI/本地可选 job 在有 Redis 时运行 `#[ignore]` 测。

use std::time::Duration;

use async_trait::async_trait;
use contracts::KeyValueStore;
use kernel::{XError, XResult};
use redis::AsyncCommands;

/// 基于 `redis` crate 的异步 KV 适配器。
pub struct RedisLiveKv {
    conn: redis::aio::MultiplexedConnection,
    endpoint: String,
}

impl RedisLiveKv {
    /// 连接到 `redis://...` URL。
    pub async fn connect(url: &str) -> XResult<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| XError::unavailable(format!("redis client: {e}")))?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| XError::unavailable(format!("redis connect: {e}")))?;
        Ok(Self { conn, endpoint: url.to_string() })
    }

    /// 端点 URL（调试用）。
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[async_trait]
impl KeyValueStore for RedisLiveKv {
    async fn get(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let mut conn = self.conn.clone();
        let v: Option<Vec<u8>> =
            conn.get(key).await.map_err(|e| XError::unavailable(format!("redis GET: {e}")))?;
        Ok(v)
    }

    async fn set(&self, key: &str, val: Vec<u8>, ttl: Option<Duration>) -> XResult<()> {
        let mut conn = self.conn.clone();
        if let Some(ttl) = ttl {
            let secs = ttl.as_secs().max(1) as i64;
            let _: () = redis::cmd("SETEX")
                .arg(key)
                .arg(secs)
                .arg(val)
                .query_async(&mut conn)
                .await
                .map_err(|e| XError::unavailable(format!("redis SETEX: {e}")))?;
        } else {
            let _: () = conn
                .set(key, val)
                .await
                .map_err(|e| XError::unavailable(format!("redis SET: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 需要本机/CI Redis：`REDIS_URL` 默认 `redis://127.0.0.1:6379`。
    #[tokio::test]
    #[ignore = "requires live Redis; run with --ignored when available"]
    async fn live_kv_roundtrip() {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
        let kv = RedisLiveKv::connect(&url).await.expect("connect");
        let key = format!("infra-s9t2:{}", std::process::id());
        kv.set(&key, b"v1".to_vec(), None).await.expect("set");
        assert_eq!(kv.get(&key).await.expect("get"), Some(b"v1".to_vec()));
        // cleanup best-effort
        let mut conn = kv.conn.clone();
        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut conn).await.unwrap_or(());
    }
}
