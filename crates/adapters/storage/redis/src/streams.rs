//! Redis Streams 一等 API（可靠消息通道；Pub/Sub 必达仍为 NO-GO）。
//!
//! 覆盖：`XADD` / `XREAD` / `XRANGE` / `XLEN` / `XDEL`。
//! 消费组（XGROUP/XREADGROUP）可作为后续扩展；本版提供基础流读写。

use kernel::{XError, XResult};

use crate::client::RedisClient;
use crate::error_map::map_redis_result;

/// Stream 条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamEntry {
    /// Redis stream ID（如 `1710000000000-0`）。
    pub id: String,
    /// field → value。
    pub fields: Vec<(String, Vec<u8>)>,
}

impl RedisClient {
    /// `XADD key * field value [field value ...]`；返回生成的 stream id。
    ///
    /// `fields` 为空 → Invalid。
    pub async fn xadd(&self, key: &str, fields: &[(&str, &[u8])]) -> XResult<String> {
        if fields.is_empty() {
            return Err(XError::invalid("XADD 至少需要一个 field"));
        }
        let key = key.to_owned();
        let owned: Vec<(String, Vec<u8>)> =
            fields.iter().map(|(f, v)| ((*f).to_owned(), (*v).to_vec())).collect();
        self.with_pool_conn(move |mut conn| async move {
            let mut cmd = redis::cmd("XADD");
            cmd.arg(&key).arg("*");
            for (f, v) in &owned {
                cmd.arg(f).arg(v.as_slice());
            }
            let id: String = map_redis_result(cmd.query_async(&mut conn).await)?;
            Ok(id)
        })
        .await
    }

    /// `XADD` 指定 id（测试/回放用）；`id` 须为合法 stream id 或 `*`。
    pub async fn xadd_with_id(
        &self,
        key: &str,
        id: &str,
        fields: &[(&str, &[u8])],
    ) -> XResult<String> {
        if fields.is_empty() {
            return Err(XError::invalid("XADD 至少需要一个 field"));
        }
        if id.trim().is_empty() {
            return Err(XError::invalid("XADD id 不能为空"));
        }
        let key = key.to_owned();
        let id = id.to_owned();
        let owned: Vec<(String, Vec<u8>)> =
            fields.iter().map(|(f, v)| ((*f).to_owned(), (*v).to_vec())).collect();
        self.with_pool_conn(move |mut conn| async move {
            let mut cmd = redis::cmd("XADD");
            cmd.arg(&key).arg(&id);
            for (f, v) in &owned {
                cmd.arg(f).arg(v.as_slice());
            }
            let out: String = map_redis_result(cmd.query_async(&mut conn).await)?;
            Ok(out)
        })
        .await
    }

    /// `XLEN`。
    pub async fn xlen(&self, key: &str) -> XResult<i64> {
        let key = key.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let n: i64 =
                map_redis_result(redis::cmd("XLEN").arg(&key).query_async(&mut conn).await)?;
            Ok(n)
        })
        .await
    }

    /// `XRANGE key start end [COUNT n]`。
    pub async fn xrange(
        &self,
        key: &str,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> XResult<Vec<StreamEntry>> {
        let key = key.to_owned();
        let start = start.to_owned();
        let end = end.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let mut cmd = redis::cmd("XRANGE");
            cmd.arg(&key).arg(&start).arg(&end);
            if let Some(n) = count {
                cmd.arg("COUNT").arg(n);
            }
            let raw: Vec<(String, Vec<(String, Vec<u8>)>)> =
                map_redis_result(cmd.query_async(&mut conn).await)?;
            Ok(raw.into_iter().map(|(id, fields)| StreamEntry { id, fields }).collect())
        })
        .await
    }

    /// `XREAD COUNT n STREAMS key id`（单流）；无新消息返回空 vec。
    ///
    /// 阻塞读请用 [`Self::xread_block`]。
    pub async fn xread(
        &self,
        key: &str,
        last_id: &str,
        count: Option<usize>,
    ) -> XResult<Vec<StreamEntry>> {
        let key = key.to_owned();
        let last_id = last_id.to_owned();
        self.with_pool_conn(move |mut conn| async move {
            let mut cmd = redis::cmd("XREAD");
            if let Some(n) = count {
                cmd.arg("COUNT").arg(n);
            }
            cmd.arg("STREAMS").arg(&key).arg(&last_id);
            parse_xread_single(&mut conn, cmd).await
        })
        .await
    }

    /// `XREAD BLOCK ms COUNT n STREAMS key id`。
    ///
    /// 使用阻塞预算 `command_timeout.max(block + 1s)`。
    pub async fn xread_block(
        &self,
        key: &str,
        last_id: &str,
        block: std::time::Duration,
        count: Option<usize>,
    ) -> XResult<Vec<StreamEntry>> {
        use crate::pool::RedisBackend;

        let ms = u64::try_from(block.as_millis()).unwrap_or(u64::MAX).max(1);
        let key = key.to_owned();
        let last_id = last_id.to_owned();
        let budget = self.pool().command_timeout().max(block + std::time::Duration::from_secs(1));
        self.pool()
            .with_conn_budget(budget, move |mut conn: RedisBackend| async move {
                let mut cmd = redis::cmd("XREAD");
                cmd.arg("BLOCK").arg(ms);
                if let Some(n) = count {
                    cmd.arg("COUNT").arg(n);
                }
                cmd.arg("STREAMS").arg(&key).arg(&last_id);
                parse_xread_single(&mut conn, cmd).await
            })
            .await
    }

    /// `XDEL`；返回删除条目数。
    pub async fn xdel(&self, key: &str, ids: &[&str]) -> XResult<i64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let key = key.to_owned();
        let ids: Vec<String> = ids.iter().map(|s| (*s).to_owned()).collect();
        self.with_pool_conn(move |mut conn| async move {
            let mut cmd = redis::cmd("XDEL");
            cmd.arg(&key);
            for id in &ids {
                cmd.arg(id);
            }
            let n: i64 = map_redis_result(cmd.query_async(&mut conn).await)?;
            Ok(n)
        })
        .await
    }
}

type XreadField = (String, Vec<u8>);
type XreadEntry = (String, Vec<XreadField>);
type XreadStream = (String, Vec<XreadEntry>);
type XreadReply = Vec<XreadStream>;

async fn parse_xread_single(
    conn: &mut impl redis::aio::ConnectionLike,
    cmd: redis::Cmd,
) -> XResult<Vec<StreamEntry>> {
    // 应答：[[stream, [[id, [f,v,...]], ...]]] 或 Nil
    let raw: Option<XreadReply> = map_redis_result(cmd.query_async(conn).await)?;
    let Some(streams) = raw else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (_name, entries) in streams {
        for (id, fields) in entries {
            out.push(StreamEntry { id, fields });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::pool::RedisPool;
    use kernel::ErrorKind;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn xadd_empty_fields_invalid() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.xadd("s", &[]).await.expect_err("empty");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn xdel_empty_ok() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        assert_eq!(client.xdel("s", &[]).await.expect("ok"), 0);
    }

    #[tokio::test]
    async fn xadd_hits_driver() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.xadd("s", &[("f", b"v")]).await.expect_err("probe");
        assert!(matches!(
            err.kind(),
            ErrorKind::Transient | ErrorKind::Unavailable | ErrorKind::Internal
        ));
    }

    #[tokio::test]
    async fn xread_block_hits_pool_budget_path() {
        let calls = Arc::new(AtomicUsize::new(0));
        let client = RedisPool::test_probe(calls.clone()).client();
        // 阻塞读走 with_conn_budget → probe driver 被调用后返回错误
        let err = client
            .xread_block("s", "0-0", Duration::from_millis(50), Some(1))
            .await
            .expect_err("probe");
        assert!(matches!(
            err.kind(),
            ErrorKind::Transient | ErrorKind::Unavailable | ErrorKind::Internal
        ));
        assert!(calls.load(Ordering::SeqCst) >= 1, "xread_block 必须进入池连接路径");
    }
}
