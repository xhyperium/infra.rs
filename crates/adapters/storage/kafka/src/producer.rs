//! Kafka 生产者：等待 broker 确认（含 key/headers 公共面，gap-zero 0.3.7+）。

use std::collections::BTreeMap;
use std::future::Future;
use std::time::Duration;

use bytes::Bytes;
use chrono::Utc;
use kernel::{XError, XResult};
use rskafka::record::Record;
use tokio::sync::watch;

use crate::error_map::map_kafka_err;
use crate::lifecycle::wait_for_shutdown;
use crate::message::{Delivery, PublishRecord};
use crate::pool::KafkaPool;

/// 可克隆 producer。
#[derive(Clone)]
pub struct KafkaProducer {
    pub(crate) pool: KafkaPool,
}

/// produce 在 shutdown / delivery 超时 / 完成 之间的有界等待结果。
///
/// 由 [`limited_produce_await`] 产生；[`publish_record`] 与单测共用此路径更新 stats。
#[derive(Debug)]
pub(crate) enum LimitedProduceAwait<T, E> {
    /// pool 关闭抢先。
    Cancelled,
    /// delivery_timeout 到期。
    TimedOut,
    /// produce future 完成。
    Ready(Result<T, E>),
}

/// 在 **shutdown 信号** 与 **delivery 超时** 之间竞争执行 `produce`。
///
/// 这是 `publish_record` 热路径上的 shipped 竞态逻辑；单测用可控 future 直接覆盖两臂。
pub(crate) async fn limited_produce_await<F, Fut, T, E>(
    mut shutdown: watch::Receiver<bool>,
    delivery_timeout: Duration,
    produce: F,
) -> LimitedProduceAwait<T, E>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    tokio::select! {
        biased;
        () = wait_for_shutdown(&mut shutdown) => LimitedProduceAwait::Cancelled,
        result = tokio::time::timeout(delivery_timeout, produce()) => match result {
            Ok(inner) => LimitedProduceAwait::Ready(inner),
            Err(_) => LimitedProduceAwait::TimedOut,
        },
    }
}

/// 将 [`limited_produce_await`] 结果映射为 `Delivery` / 错误，并递增对应 pool stats。
///
/// 与 `publish_record` 在 produce 阶段使用同一套计数语义。
pub(crate) fn apply_limited_produce_outcome<E>(
    pool: &KafkaPool,
    partition: i32,
    outcome: LimitedProduceAwait<Vec<i64>, E>,
    map_err: impl FnOnce(E) -> XError,
) -> XResult<Delivery> {
    match outcome {
        LimitedProduceAwait::Cancelled => {
            pool.record_publish_cancelled();
            Err(XError::cancelled("kafkax produce 因 pool 关闭而取消"))
        }
        LimitedProduceAwait::TimedOut => {
            pool.record_publish_timeout();
            Err(XError::deadline_exceeded("kafkax produce 超时"))
        }
        LimitedProduceAwait::Ready(Ok(offsets)) => {
            let offset = offsets.first().copied().unwrap_or(0);
            pool.record_publish_ok();
            Ok(Delivery { partition, offset })
        }
        LimitedProduceAwait::Ready(Err(e)) => {
            pool.record_publish_err();
            Err(map_err(e))
        }
    }
}

impl KafkaProducer {
    /// 发布到 topic（默认分区 0），等待 produce 结果。
    pub async fn publish(&self, topic: &str, payload: Bytes) -> XResult<Delivery> {
        self.publish_to_partition(topic, 0, payload).await
    }

    /// 指定分区发布（无 key / headers）。
    pub async fn publish_to_partition(
        &self,
        topic: &str,
        partition: i32,
        payload: Bytes,
    ) -> XResult<Delivery> {
        self.publish_record(PublishRecord::payload(topic, partition, payload)).await
    }

    /// 指定分区 + key 发布。
    pub async fn publish_with_key(
        &self,
        topic: &str,
        partition: i32,
        key: Bytes,
        payload: Bytes,
    ) -> XResult<Delivery> {
        self.publish_record(PublishRecord::payload(topic, partition, payload).with_key(key)).await
    }

    /// 完整记录发布（key / headers / 分区）。
    ///
    /// # Errors
    ///
    /// topic 为空、partition 为负、pool 已关闭、delivery 超时或 broker 错误。
    pub async fn publish_record(&self, record: PublishRecord) -> XResult<Delivery> {
        self.pool.ensure_open()?;
        validate_publish_topic(&record.topic)?;
        if record.partition < 0 {
            return Err(XError::invalid("kafkax: partition 不能为负"));
        }
        // partition_client 在 pool 关闭时返回 Cancelled：计入 publish_cancelled
        // （整次 publish 因关闭取消，与 produce select 臂语义一致）
        let client = match self.pool.partition_client(&record.topic, record.partition).await {
            Ok(c) => c,
            Err(e) if e.kind() == kernel::ErrorKind::Cancelled => {
                self.pool.record_publish_cancelled();
                return Err(e);
            }
            Err(e) => return Err(e),
        };
        let _operation = self.pool.start_operation()?;
        let shutdown = self.pool.shutdown_receiver();
        let wire = Record {
            key: record.key.as_ref().map(|k| k.to_vec()),
            value: Some(record.payload.to_vec()),
            headers: record
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), v.to_vec()))
                .collect::<BTreeMap<_, _>>(),
            timestamp: Utc::now(),
        };
        let delivery_timeout = self.pool.config().delivery_timeout;
        let outcome = limited_produce_await(shutdown, delivery_timeout, || async {
            client.produce(vec![wire], KafkaPool::compression()).await
        })
        .await;
        apply_limited_produce_outcome(&self.pool, record.partition, outcome, |e| {
            map_kafka_err("kafkax produce", e)
        })
    }
}

/// 在 broker I/O 前校验 publish topic 形状。
fn validate_publish_topic(topic: &str) -> XResult<()> {
    if topic.trim().is_empty() {
        return Err(XError::invalid("kafkax: topic 不能为空"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KafkaConfig;
    use kernel::ErrorKind;

    #[test]
    fn empty_topic_rejected_before_broker_io() {
        let error = validate_publish_topic("  ").expect_err("空 topic 必须失败");
        assert_eq!(error.kind(), ErrorKind::Invalid);
        assert!(error.context().contains("topic"));
    }

    #[test]
    fn non_empty_topic_accepted() {
        validate_publish_topic("orders").expect("合法 topic");
    }

    #[test]
    fn publish_record_rejects_negative_partition_shape() {
        validate_publish_topic("ok").expect("ok");
    }

    /// **严格**：timeout 臂必须递增 `publish_timeouts`（非 OR published/failed）。
    #[tokio::test]
    async fn limited_await_timeout_arm_increments_publish_timeouts() {
        let pool = KafkaPool::stats_stub(KafkaConfig {
            delivery_timeout: Duration::from_millis(25),
            ..KafkaConfig::default()
        });
        let (_tx, rx) = watch::channel(false);
        let before = pool.stats();
        assert_eq!(before.publish_timeouts, 0);

        let outcome = limited_produce_await(rx, Duration::from_millis(25), || async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok::<Vec<i64>, String>(vec![1])
        })
        .await;
        assert!(
            matches!(outcome, LimitedProduceAwait::TimedOut),
            "应命中 TimedOut 臂: {outcome:?}"
        );

        let err = apply_limited_produce_outcome(&pool, 0, outcome, |e| {
            XError::internal(format!("map: {e}"))
        })
        .expect_err("timeout → err");
        assert_eq!(err.kind(), ErrorKind::DeadlineExceeded);

        let after = pool.stats();
        assert!(
            after.publish_timeouts > before.publish_timeouts,
            "timeout 臂必须严格递增 publish_timeouts: before={} after={}",
            before.publish_timeouts,
            after.publish_timeouts
        );
        assert_eq!(after.publish_cancelled, before.publish_cancelled);
        assert!(after.publish_failed > before.publish_failed);
    }

    /// **严格**：cancel 臂必须递增 `publish_cancelled`。
    #[tokio::test]
    async fn limited_await_cancel_arm_increments_publish_cancelled() {
        let pool = KafkaPool::stats_stub(KafkaConfig::default());
        let (tx, rx) = watch::channel(false);
        let before = pool.stats();
        assert_eq!(before.publish_cancelled, 0);

        let pool_task = pool.clone();
        let handle = tokio::spawn(async move {
            limited_produce_await(rx, Duration::from_secs(60), || async {
                // 永不完成：只能被 shutdown 取消
                std::future::pending::<Result<Vec<i64>, String>>().await
            })
            .await
        });

        // 确保 future 已进入 select
        tokio::time::sleep(Duration::from_millis(30)).await;
        tx.send(true).expect("signal shutdown");

        let outcome = handle.await.expect("join");
        assert!(
            matches!(outcome, LimitedProduceAwait::Cancelled),
            "应命中 Cancelled 臂: {outcome:?}"
        );

        let err = apply_limited_produce_outcome(&pool_task, 0, outcome, |e| {
            XError::internal(format!("map: {e}"))
        })
        .expect_err("cancel → err");
        assert_eq!(err.kind(), ErrorKind::Cancelled);

        let after = pool.stats();
        assert!(
            after.publish_cancelled > before.publish_cancelled,
            "cancel 臂必须严格递增 publish_cancelled: before={} after={}",
            before.publish_cancelled,
            after.publish_cancelled
        );
        assert_eq!(after.publish_timeouts, before.publish_timeouts);
        assert!(after.publish_failed > before.publish_failed);
    }

    /// Ready(Ok) 走 published，不碰 cancel/timeout 计数。
    #[tokio::test]
    async fn limited_await_ready_ok_increments_published_only() {
        let pool = KafkaPool::stats_stub(KafkaConfig::default());
        let (_tx, rx) = watch::channel(false);
        let before = pool.stats();
        let outcome = limited_produce_await(rx, Duration::from_secs(1), || async {
            Ok::<Vec<i64>, String>(vec![9])
        })
        .await;
        let d = apply_limited_produce_outcome(&pool, 2, outcome, |e| {
            XError::internal(format!("map: {e}"))
        })
        .expect("ok");
        assert_eq!(d.partition, 2);
        assert_eq!(d.offset, 9);
        let after = pool.stats();
        assert!(after.published > before.published);
        assert_eq!(after.publish_timeouts, before.publish_timeouts);
        assert_eq!(after.publish_cancelled, before.publish_cancelled);
    }

    /// Ready(Err) 走 publish_failed。
    #[tokio::test]
    async fn limited_await_ready_err_increments_failed() {
        let pool = KafkaPool::stats_stub(KafkaConfig::default());
        let (_tx, rx) = watch::channel(false);
        let before = pool.stats();
        let outcome = limited_produce_await(rx, Duration::from_secs(1), || async {
            Err::<Vec<i64>, String>("broker boom".into())
        })
        .await;
        let err = apply_limited_produce_outcome(&pool, 0, outcome, |e| {
            XError::unavailable(format!("produce: {e}"))
        })
        .expect_err("err");
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        let after = pool.stats();
        assert!(after.publish_failed > before.publish_failed);
        assert_eq!(after.publish_timeouts, before.publish_timeouts);
        assert_eq!(after.publish_cancelled, before.publish_cancelled);
    }
}
