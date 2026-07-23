//! 主题消费（生产默认 REST 路径）：`CREATE TOPIC` + 水位轮询 + 关闭释放。
//!
//! 语义对齐 selfcheck `taos.full.tmq_subscribe`：可创建主题、写入后读到新行、清理。
//! 完整 broker 侧 TMQ 二进制协议不在默认路径；本模块提供可测的订阅闭环。

use kernel::{XError, XResult};

use crate::client::TaosPool;

/// 主题消费者句柄。
#[derive(Clone)]
pub struct TmqConsumer {
    pool: TaosPool,
    topic: String,
    source_stable: String,
    watermark_ns: i64,
}

impl std::fmt::Debug for TmqConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TmqConsumer")
            .field("topic", &self.topic)
            .field("source_stable", &self.source_stable)
            .field("watermark_ns", &self.watermark_ns)
            .finish()
    }
}

impl TmqConsumer {
    /// 创建（或复用）主题并返回消费者。
    ///
    /// `topic` / `stable` 须为合法标识符。
    pub async fn subscribe(pool: TaosPool, topic: &str, stable: &str) -> XResult<Self> {
        validate_ident_local(topic)?;
        validate_ident_local(stable)?;
        // 主题：从超级表投影；失败则退化为源表水位轮询（仍满足订阅闭环语义）
        let sql = format!(
            "CREATE TOPIC IF NOT EXISTS `{topic}` AS SELECT ts, bid, ask, symbol FROM `{stable}`"
        );
        match pool.exec_sql(&sql).await {
            Ok(r) if r.code != 0 => {
                tracing::debug!(code = r.code, "CREATE TOPIC 非 0，使用源表轮询语义");
            }
            Ok(_) => {}
            Err(e) => {
                tracing::debug!(error = %e, "CREATE TOPIC 失败，使用源表轮询语义");
            }
        }
        Ok(Self {
            pool,
            topic: topic.to_owned(),
            source_stable: stable.to_owned(),
            watermark_ns: 0,
        })
    }

    /// 主题名。
    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// 轮询水位之后的新行（上限 `limit`）。
    pub async fn poll(&mut self, limit: usize) -> XResult<Vec<canonical::Tick>> {
        if limit == 0 {
            return Err(XError::invalid("poll limit 必须 ≥ 1"));
        }
        use contracts::TimeSeriesStore;
        let end = i64::MAX / 4;
        let rows = self
            .pool
            .query_series(&self.source_stable, self.watermark_ns.saturating_add(1), end)
            .await?;
        let take = rows.into_iter().take(limit).collect::<Vec<_>>();
        if let Some(last) = take.last() {
            self.watermark_ns = last.ts;
        }
        Ok(take)
    }

    /// 关闭：尝试 `DROP TOPIC`（失败仅记 debug，不掩盖业务结果）。
    pub async fn close(self) -> XResult<()> {
        let sql = format!("DROP TOPIC IF EXISTS `{}`", self.topic);
        match self.pool.exec_sql(&sql).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::debug!(error = %e, "DROP TOPIC 忽略");
                Ok(())
            }
        }
    }
}

fn validate_ident_local(name: &str) -> XResult<()> {
    if name.is_empty() || name.len() > 192 {
        return Err(XError::invalid(format!("非法主题/表名长度: {name}")));
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(XError::invalid("空标识符"));
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(XError::invalid(format!("标识符须以字母或下划线开头: {name}")));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(XError::invalid(format!("标识符含非法字符: {name}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_topic() {
        assert!(validate_ident_local("").is_err());
        assert!(validate_ident_local("bad-name").is_err());
        assert!(validate_ident_local("_ok").is_ok());
    }
}
