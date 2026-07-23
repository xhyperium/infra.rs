//! 一等数据结构 API：Hash / List / Set / Sorted Set。
//!
//! 全部走 [`RedisClient`] 生产路径（池 + 超时 + 可选 call deadline），非 raw 旁路。

use kernel::XResult;
use redis::AsyncCommands;

use crate::client::RedisClient;
use crate::error_map::map_redis_result;

impl RedisClient {
    // ── Hash ──────────────────────────────────────────────────────────

    /// `HSET key field value`；返回是否新建字段。
    pub async fn hset(&self, key: &str, field: &str, value: Vec<u8>) -> XResult<bool> {
        let key = key.to_owned();
        let field = field.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.hset(key, field, value).await)?;
            Ok(n > 0)
        })
        .await
    }

    /// `HGET`；缺失字段返回 `Ok(None)`。
    pub async fn hget(&self, key: &str, field: &str) -> XResult<Option<Vec<u8>>> {
        let key = key.to_owned();
        let field = field.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let v: Option<Vec<u8>> = map_redis_result(conn.hget(key, field).await)?;
            Ok(v)
        })
        .await
    }

    /// `HDEL`；返回删除字段数。
    pub async fn hdel(&self, key: &str, fields: &[&str]) -> XResult<i64> {
        if fields.is_empty() {
            return Ok(0);
        }
        let key = key.to_owned();
        let fields: Vec<String> = fields.iter().map(|f| (*f).to_owned()).collect();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.hdel(key, fields).await)?;
            Ok(n)
        })
        .await
    }

    /// `HGETALL` → `(field, value)` 列表。
    pub async fn hgetall(&self, key: &str) -> XResult<Vec<(String, Vec<u8>)>> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let map: std::collections::HashMap<String, Vec<u8>> =
                map_redis_result(conn.hgetall(key).await)?;
            Ok(map.into_iter().collect())
        })
        .await
    }

    // ── List ──────────────────────────────────────────────────────────

    /// `LPUSH`；返回推入后列表长度。
    pub async fn lpush(&self, key: &str, value: Vec<u8>) -> XResult<i64> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.lpush(key, value).await)?;
            Ok(n)
        })
        .await
    }

    /// `RPUSH`；返回推入后列表长度。
    pub async fn rpush(&self, key: &str, value: Vec<u8>) -> XResult<i64> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.rpush(key, value).await)?;
            Ok(n)
        })
        .await
    }

    /// `LPOP`；空列表返回 `Ok(None)`。
    pub async fn lpop(&self, key: &str) -> XResult<Option<Vec<u8>>> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let v: Option<Vec<u8>> = map_redis_result(conn.lpop(key, None).await)?;
            Ok(v)
        })
        .await
    }

    /// `LRANGE start stop`（含端点）。
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> XResult<Vec<Vec<u8>>> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let v: Vec<Vec<u8>> = map_redis_result(conn.lrange(key, start, stop).await)?;
            Ok(v)
        })
        .await
    }

    /// `BLPOP` 阻塞弹出；`timeout` 为最长等待（秒，至少 1s 映射到 Redis 秒级）。
    ///
    /// 使用**阻塞命令预算**（`command_timeout.max(timeout + 1s)`），避免与短命令超时冲突。
    /// 超时无元素返回 `Ok(None)`。
    pub async fn blpop(
        &self,
        key: &str,
        timeout: std::time::Duration,
    ) -> XResult<Option<(String, Vec<u8>)>> {
        use crate::pool::RedisBackend;

        let secs = timeout.as_secs().max(1);
        let key = key.to_owned();
        let budget = self.pool().command_timeout().max(timeout + std::time::Duration::from_secs(1));
        self.pool()
            .with_conn_budget(budget, move |mut conn: RedisBackend| async move {
                let raw: Option<(String, Vec<u8>)> = map_redis_result(
                    redis::cmd("BLPOP").arg(&key).arg(secs).query_async(&mut conn).await,
                )?;
                Ok(raw)
            })
            .await
    }

    // ── Set ───────────────────────────────────────────────────────────

    /// `SADD`；返回新增成员数。
    pub async fn sadd(&self, key: &str, member: Vec<u8>) -> XResult<i64> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.sadd(key, member).await)?;
            Ok(n)
        })
        .await
    }

    /// `SISMEMBER`。
    pub async fn sismember(&self, key: &str, member: &[u8]) -> XResult<bool> {
        let key = key.to_owned();
        let member = member.to_vec();
        self.with_pool_conn(move |mut conn| async move {
            let ok: bool = map_redis_result(conn.sismember(key, member).await)?;
            Ok(ok)
        })
        .await
    }

    /// `SREM`；返回移除成员数。
    pub async fn srem(&self, key: &str, member: &[u8]) -> XResult<i64> {
        let key = key.to_owned();
        let member = member.to_vec();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.srem(key, member).await)?;
            Ok(n)
        })
        .await
    }

    // ── Sorted Set ────────────────────────────────────────────────────

    /// `ZADD`；返回新增成员数。
    pub async fn zadd(&self, key: &str, member: Vec<u8>, score: f64) -> XResult<i64> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.zadd(key, member, score).await)?;
            Ok(n)
        })
        .await
    }

    /// `ZSCORE`；缺失返回 `Ok(None)`。
    pub async fn zscore(&self, key: &str, member: &[u8]) -> XResult<Option<f64>> {
        let key = key.to_owned();
        let member = member.to_vec();
        self.with_pool_conn(move |mut conn| async move {
            let s: Option<f64> = map_redis_result(conn.zscore(key, member).await)?;
            Ok(s)
        })
        .await
    }

    /// `ZREM`；返回移除成员数。
    pub async fn zrem(&self, key: &str, member: &[u8]) -> XResult<i64> {
        let key = key.to_owned();
        let member = member.to_vec();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 = map_redis_result(conn.zrem(key, member).await)?;
            Ok(n)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::pool::RedisPool;
    use kernel::ErrorKind;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn hdel_empty_fields_ok() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        assert_eq!(client.hdel("k", &[]).await.expect("empty"), 0);
    }

    #[tokio::test]
    async fn structure_apis_hit_pool_path() {
        // probe driver 返回错误 → 证明调用进入 with_pool_conn
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.hset("k", "f", b"v".to_vec()).await.expect_err("probe");
        assert!(matches!(
            err.kind(),
            ErrorKind::Transient | ErrorKind::Unavailable | ErrorKind::Internal
        ));
    }
}
