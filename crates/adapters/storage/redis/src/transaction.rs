//! MULTI/EXEC 事务封装（Standalone 或同 slot 语义）。
//!
//! Cluster 跨 slot 不承诺；调用方须保证 key 同槽。

use kernel::{XError, XResult};

use crate::client::RedisClient;
use crate::error_map::map_redis_result;

/// 事务内排队的一条命令描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxCmd {
    /// SET key value（无 TTL）。
    Set { key: String, value: Vec<u8> },
    /// DEL key。
    Del { key: String },
    /// INCR key。
    Incr { key: String },
}

impl RedisClient {
    /// `MULTI` → 排队命令 → `EXEC`；返回各命令的原始 [`redis::Value`]。
    ///
    /// `cmds` 为空 → Invalid。跨 Cluster slot **不**承诺原子。
    pub async fn multi_exec(&self, cmds: &[TxCmd]) -> XResult<Vec<redis::Value>> {
        if cmds.is_empty() {
            return Err(XError::invalid("MULTI/EXEC 至少需要一条命令"));
        }
        let cmds = cmds.to_vec();
        self.with_pool_conn(move |mut conn| async move {
            let mut pipe = redis::pipe();
            pipe.atomic();
            for c in &cmds {
                match c {
                    TxCmd::Set { key, value } => {
                        pipe.cmd("SET").arg(key).arg(value.as_slice());
                    }
                    TxCmd::Del { key } => {
                        pipe.cmd("DEL").arg(key);
                    }
                    TxCmd::Incr { key } => {
                        pipe.cmd("INCR").arg(key);
                    }
                }
            }
            let vals: Vec<redis::Value> = map_redis_result(pipe.query_async(&mut conn).await)?;
            Ok(vals)
        })
        .await
    }

    /// 便捷：在事务中 SET 多个 key（无 TTL）并 EXEC。
    pub async fn multi_set(&self, items: &[(&str, &[u8])]) -> XResult<()> {
        if items.is_empty() {
            return Ok(());
        }
        let cmds: Vec<TxCmd> = items
            .iter()
            .map(|(k, v)| TxCmd::Set { key: (*k).to_owned(), value: (*v).to_vec() })
            .collect();
        let _ = self.multi_exec(&cmds).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RedisPool;
    use kernel::ErrorKind;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn multi_exec_empty_invalid() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client.multi_exec(&[]).await.expect_err("empty");
        assert_eq!(err.kind(), ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn multi_set_empty_ok() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        client.multi_set(&[]).await.expect("empty");
    }

    #[tokio::test]
    async fn multi_exec_hits_driver() {
        let client = RedisPool::test_probe(Arc::new(AtomicUsize::new(0))).client();
        let err = client
            .multi_exec(&[TxCmd::Set { key: "k".into(), value: b"v".to_vec() }])
            .await
            .expect_err("probe");
        assert!(matches!(
            err.kind(),
            ErrorKind::Transient | ErrorKind::Unavailable | ErrorKind::Internal
        ));
    }
}
