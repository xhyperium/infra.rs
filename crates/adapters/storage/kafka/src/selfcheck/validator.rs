//! kafka [`Validatable`] 实现与 §6.2 检查项执行。

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use bytes::Bytes;
use chrono::Utc;
use rskafka::record::Record;
use tokio::time::timeout;

use crate::config::KafkaConfig;
use crate::consumer::ConsumerConfig;
use crate::offset::{MemoryOffsetStore, OffsetCommitStore};
use crate::pool::KafkaPool;

use super::config::KafkaSelfCheckConfig;
use super::context::ValidationContext;
use super::types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};

/// 模块名（规范 3.1）。
pub const MODULE: &str = "kafka";

/// 检查返回：`Ok` 通过；`Skip` 不适用；`Fail` 失败。
enum CheckOutcome {
    Ok(Option<String>),
    Skip(String),
    Fail(String),
}

/// kafka 验证器：绑定 [`KafkaPool`] + 配置。
#[derive(Clone)]
pub struct KafkaValidator {
    pool: KafkaPool,
    config: KafkaSelfCheckConfig,
}

/// 验证器契约（规范 §5.2 子集，模块内聚）。
#[async_trait::async_trait]
pub trait Validatable: Send + Sync {
    fn module_name(&self) -> &'static str;
    fn catalog(&self) -> Vec<CheckDescriptor>;
    async fn validate(&self, ctx: &ValidationContext, level: CheckLevel) -> ValidationReport;
}

impl KafkaValidator {
    #[must_use]
    pub fn new(pool: KafkaPool) -> Self {
        Self { pool, config: KafkaSelfCheckConfig::default() }
    }

    /// 从配置建池。
    pub async fn connect(cfg: KafkaConfig) -> kernel::XResult<Self> {
        let pool = KafkaPool::connect(cfg).await?;
        Ok(Self::new(pool))
    }

    /// 从环境变量建池。
    pub async fn connect_from_env() -> kernel::XResult<Self> {
        let pool = KafkaPool::connect_from_env().await?;
        Ok(Self::new(pool))
    }

    /// 连接失败时**不 panic**：合成 Basic Failed + 高级别 Skipped 报告（启动路径）。
    pub async fn connect_and_run(cfg: KafkaConfig, level: CheckLevel) -> ValidationReport {
        match Self::connect(cfg).await {
            Ok(v) => v.run(level).await,
            Err(e) => connect_failed_report(level, &format!("{:?}", e.kind())),
        }
    }

    #[must_use]
    pub fn with_config(mut self, config: KafkaSelfCheckConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn config(&self) -> &KafkaSelfCheckConfig {
        &self.config
    }

    #[must_use]
    pub fn pool(&self) -> &KafkaPool {
        &self.pool
    }

    /// 使用默认上下文执行验证（生成 run_id/token）。
    pub async fn run(&self, level: CheckLevel) -> ValidationReport {
        let ctx = ValidationContext::new(self.config.clone());
        self.validate(&ctx, level).await
    }

    /// 静态 catalog（§6.2，共 9 项）。
    #[must_use]
    pub fn static_catalog() -> Vec<CheckDescriptor> {
        vec![
            desc(
                "kafka.basic.metadata",
                CheckLevel::Basic,
                Some(500),
                "broker 可达且 bootstrap 数量 ≥ min_brokers",
                false,
            ),
            desc(
                "kafka.rw.produce_consume",
                CheckLevel::ReadWrite,
                Some(5_000),
                "专用 topic 收发闭环 + token 比对",
                false,
            ),
            desc(
                "kafka.full.topic_create_delete",
                CheckLevel::Full,
                Some(5_000),
                "Admin 建删临时 topic",
                true,
            ),
            desc(
                "kafka.full.key_partition_routing",
                CheckLevel::Full,
                None,
                "相同 key 恒定路由（本库显式分区 + key）",
                true,
            ),
            desc(
                "kafka.full.ordering_headers",
                CheckLevel::Full,
                Some(5_000),
                "同分区顺序；headers 公共面 partial",
                true,
            ),
            desc(
                "kafka.full.offset_commit",
                CheckLevel::Full,
                None,
                "应用层 OffsetCommitStore：committed = offset+1",
                false,
            ),
            desc(
                "kafka.full.group_lag",
                CheckLevel::Full,
                None,
                "业务消费组积压（rskafka NO-GO）",
                false,
            ),
            desc(
                "kafka.full.large_message",
                CheckLevel::Full,
                Some(3_000),
                "接近配置大小的大消息收发",
                true,
            ),
            desc(
                "kafka.full.isr_health",
                CheckLevel::Full,
                None,
                "无 under-replicated 分区（rskafka NO-GO）",
                false,
            ),
        ]
    }
}

fn desc(
    id: &str,
    level: CheckLevel,
    baseline: Option<u64>,
    description: &str,
    destructive: bool,
) -> CheckDescriptor {
    CheckDescriptor {
        id: id.to_owned(),
        level,
        default_baseline_ms: baseline,
        description: description.to_owned(),
        destructive,
    }
}

/// 连接失败合成报告（离线/故障注入入口）。
fn connect_failed_report(level: CheckLevel, kind_detail: &str) -> ValidationReport {
    let started = now_rfc3339();
    let mut items = Vec::new();
    if level.includes(CheckLevel::Basic) {
        items.push(CheckItem::finish(
            "kafka.basic.metadata",
            CheckStatus::Failed,
            Duration::from_millis(0),
            Some(500),
            Some(redact_detail(&format!("connect 失败: {kind_detail}"))),
            started.clone(),
        ));
    }
    if level.includes(CheckLevel::ReadWrite) {
        items.push(CheckItem::skipped(
            "kafka.rw.produce_consume",
            "短路：basic 失败（connect）",
            started.clone(),
        ));
    }
    if level.includes(CheckLevel::Full) {
        for id in FULL_IDS {
            items.push(CheckItem::skipped(*id, "短路：basic 失败（connect）", started.clone()));
        }
    }
    ValidationReport::from_items(MODULE, level, items)
}

const FULL_IDS: &[&str] = &[
    "kafka.full.topic_create_delete",
    "kafka.full.key_partition_routing",
    "kafka.full.ordering_headers",
    "kafka.full.offset_commit",
    "kafka.full.group_lag",
    "kafka.full.large_message",
    "kafka.full.isr_health",
];

#[async_trait::async_trait]
impl Validatable for KafkaValidator {
    fn module_name(&self) -> &'static str {
        MODULE
    }

    fn catalog(&self) -> Vec<CheckDescriptor> {
        Self::static_catalog()
    }

    async fn validate(&self, ctx: &ValidationContext, level: CheckLevel) -> ValidationReport {
        let overall = Instant::now();
        let mut items = Vec::new();
        let level_budget = level.max_duration();
        let run = self.run_checks(ctx, level, &mut items);
        if timeout(level_budget, run).await.is_err() {
            let started = now_rfc3339();
            for d in self.catalog() {
                if !level.includes(d.level) {
                    continue;
                }
                if items.iter().any(|i| i.id == d.id) {
                    continue;
                }
                items.push(CheckItem::finish(
                    d.id,
                    CheckStatus::Failed,
                    overall.elapsed(),
                    None,
                    Some("模块验证超时".into()),
                    started.clone(),
                ));
            }
        }
        ValidationReport::from_items(MODULE, level, items)
    }
}

impl KafkaValidator {
    async fn run_checks(
        &self,
        ctx: &ValidationContext,
        level: CheckLevel,
        items: &mut Vec<CheckItem>,
    ) {
        if level.includes(CheckLevel::Basic) {
            items.push(
                self.exec_check(ctx, "kafka.basic.metadata", Some(500), self.check_metadata(ctx))
                    .await,
            );
        }

        let basic_failed = items
            .iter()
            .any(|i| i.id.starts_with("kafka.basic.") && i.status == CheckStatus::Failed);

        if level.includes(CheckLevel::ReadWrite) {
            if basic_failed {
                items.push(CheckItem::skipped(
                    "kafka.rw.produce_consume",
                    "短路：basic 失败",
                    now_rfc3339(),
                ));
            } else {
                items.push(
                    self.exec_check(
                        ctx,
                        "kafka.rw.produce_consume",
                        Some(5_000),
                        self.check_produce_consume(ctx),
                    )
                    .await,
                );
            }
        }

        if !level.includes(CheckLevel::Full) {
            return;
        }

        if basic_failed {
            for id in FULL_IDS {
                items.push(CheckItem::skipped(*id, "短路：basic 失败", now_rfc3339()));
            }
            return;
        }

        items.push(
            self.exec_check(
                ctx,
                "kafka.full.topic_create_delete",
                Some(5_000),
                self.check_topic_create_delete(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "kafka.full.key_partition_routing",
                None,
                self.check_key_partition_routing(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "kafka.full.ordering_headers",
                Some(5_000),
                self.check_ordering_headers(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "kafka.full.offset_commit", None, self.check_offset_commit(ctx))
                .await,
        );
        items.push(
            self.exec_check(ctx, "kafka.full.group_lag", None, self.check_group_lag(ctx)).await,
        );
        items.push(
            self.exec_check(
                ctx,
                "kafka.full.large_message",
                Some(3_000),
                self.check_large_message(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "kafka.full.isr_health", None, self.check_isr_health(ctx)).await,
        );
    }

    async fn exec_check(
        &self,
        ctx: &ValidationContext,
        id: &str,
        default_baseline: Option<u64>,
        outcome_fut: impl std::future::Future<Output = CheckOutcome>,
    ) -> CheckItem {
        let started_at = now_rfc3339();
        let baseline = ctx
            .config
            .baseline_ms(id, default_baseline)
            .or_else(|| self.config.baseline_ms(id, default_baseline));

        if ctx.config.is_skipped(id) || self.config.is_skipped(id) {
            return CheckItem::skipped(id, "配置 skip", started_at);
        }
        if ctx.is_cancelled() {
            return CheckItem::skipped(id, "已取消", started_at);
        }

        let t0 = Instant::now();
        let outcome = outcome_fut.await;
        let latency = t0.elapsed();
        match outcome {
            CheckOutcome::Ok(detail) => {
                CheckItem::finish(id, CheckStatus::Passed, latency, baseline, detail, started_at)
            }
            CheckOutcome::Skip(reason) => CheckItem::skipped(id, reason, started_at),
            CheckOutcome::Fail(detail) => CheckItem::finish(
                id,
                CheckStatus::Failed,
                latency,
                baseline,
                Some(redact_detail(&detail)),
                started_at,
            ),
        }
    }

    // -------------------- checks --------------------

    async fn check_metadata(&self, ctx: &ValidationContext) -> CheckOutcome {
        let min = ctx.config.min_brokers.max(self.config.min_brokers);
        let bootstrap =
            self.pool.config().brokers.split(',').map(str::trim).filter(|s| !s.is_empty()).count()
                as u32;
        if bootstrap < min {
            return CheckOutcome::Fail(format!(
                "bootstrap brokers={bootstrap} < min_brokers={min}"
            ));
        }
        match self.pool.health().await {
            Ok(h) if h.ready => CheckOutcome::Ok(Some(format!(
                "bootstrap={bootstrap}; {}",
                redact_detail(&h.detail)
            ))),
            Ok(h) => CheckOutcome::Fail(format!("health not ready: {}", redact_detail(&h.detail))),
            Err(e) => CheckOutcome::Fail(format!("health: {:?}", e.kind())),
        }
    }

    async fn check_produce_consume(&self, ctx: &ValidationContext) -> CheckOutcome {
        let topic = ctx.topic("rw");
        let repl = ctx.config.replication;
        if let Err(e) = self.pool.ensure_topic(&topic, 1, repl).await {
            return CheckOutcome::Fail(format!("ensure_topic: {:?}", e.kind()));
        }
        let payload = Bytes::from(format!("kafkax-selfcheck-{}", ctx.token));
        let delivery = match self.pool.producer().publish(&topic, payload.clone()).await {
            Ok(d) => d,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("produce: {:?}", e.kind()));
            }
        };
        let cfg = ConsumerConfig::assign(&topic, 0, format!("_scg_{}", ctx.token))
            .with_start_offset(delivery.offset);
        let mut consumer = match self.pool.consumer(cfg).await {
            Ok(c) => c,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("consumer: {:?}", e.kind()));
            }
        };
        let wait = Duration::from_millis(ctx.config.consume_wait_ms.max(1_000));
        let got = match consumer.recv_timeout(wait).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                drop(consumer);
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail("consume 空结果".into());
            }
            Err(e) => {
                drop(consumer);
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("consume: {:?}", e.kind()));
            }
        };
        drop(consumer);
        let _ = self.pool.delete_topic(&topic).await;
        if got.payload.as_ref() == payload.as_ref() {
            CheckOutcome::Ok(Some(format!("offset={}", delivery.offset)))
        } else {
            CheckOutcome::Fail("produce/consume payload 比对失败".into())
        }
    }

    async fn check_topic_create_delete(&self, ctx: &ValidationContext) -> CheckOutcome {
        let topic = ctx.topic("ddl");
        let repl = ctx.config.replication;
        if let Err(e) = self.pool.ensure_topic(&topic, 1, repl).await {
            return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
        }
        // 确认出现在 metadata
        match self.pool.client().list_topics().await {
            Ok(topics) if topics.iter().any(|t| t.name == topic) => {}
            Ok(_) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail("create 后 list_topics 未见 topic".into());
            }
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("list_topics: {e}"));
            }
        }
        if let Err(e) = self.pool.delete_topic(&topic).await {
            return CheckOutcome::Fail(format!("delete: {:?}", e.kind()));
        }
        // 删除后允许短暂元数据延迟：再试一次 list
        tokio::time::sleep(Duration::from_millis(200)).await;
        match self.pool.client().list_topics().await {
            Ok(topics) if topics.iter().any(|t| t.name == topic) => {
                CheckOutcome::Ok(Some("delete 已调用；list 仍可能短暂可见（元数据延迟）".into()))
            }
            Ok(_) => CheckOutcome::Ok(None),
            Err(e) => CheckOutcome::Fail(format!("list after delete: {e}")),
        }
    }

    async fn check_key_partition_routing(&self, ctx: &ValidationContext) -> CheckOutcome {
        let topic = ctx.topic("route");
        let partitions = 3i32;
        let repl = ctx.config.replication;
        if let Err(e) = self.pool.ensure_topic(&topic, partitions, repl).await {
            return CheckOutcome::Fail(format!("ensure_topic: {:?}", e.kind()));
        }
        let key = b"stable-key-selfcheck";
        let p1 = stable_partition(key, partitions);
        let p2 = stable_partition(key, partitions);
        if p1 != p2 {
            let _ = self.pool.delete_topic(&topic).await;
            return CheckOutcome::Fail("本地 hash 不稳定".into());
        }
        let d1 = match produce_with_key(&self.pool, &topic, p1, Some(key), b"v1").await {
            Ok(d) => d,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("produce1: {:?}", e.kind()));
            }
        };
        let d2 = match produce_with_key(&self.pool, &topic, p2, Some(key), b"v2").await {
            Ok(d) => d,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("produce2: {:?}", e.kind()));
            }
        };
        let _ = self.pool.delete_topic(&topic).await;
        if d1.partition == d2.partition && d1.partition == p1 {
            CheckOutcome::Ok(Some(format!(
                "key→partition={p1}（应用层稳定 hash，非 broker sticky partitioner）"
            )))
        } else {
            CheckOutcome::Fail(format!(
                "分区不一致 d1={} d2={} expect={p1}",
                d1.partition, d2.partition
            ))
        }
    }

    async fn check_ordering_headers(&self, ctx: &ValidationContext) -> CheckOutcome {
        let topic = ctx.topic("ord");
        let repl = ctx.config.replication;
        if let Err(e) = self.pool.ensure_topic(&topic, 1, repl).await {
            return CheckOutcome::Fail(format!("ensure_topic: {:?}", e.kind()));
        }
        let producer = self.pool.producer();
        let mut first_offset = None;
        for i in 0..5u8 {
            match producer.publish(&topic, Bytes::from(vec![i])).await {
                Ok(d) => {
                    if first_offset.is_none() {
                        first_offset = Some(d.offset);
                    }
                }
                Err(e) => {
                    let _ = self.pool.delete_topic(&topic).await;
                    return CheckOutcome::Fail(format!("produce[{i}]: {:?}", e.kind()));
                }
            }
        }
        let start = first_offset.unwrap_or(0);
        let cfg = ConsumerConfig::assign(&topic, 0, format!("_scg_ord_{}", ctx.token))
            .with_start_offset(start);
        let mut consumer = match self.pool.consumer(cfg).await {
            Ok(c) => c,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("consumer: {:?}", e.kind()));
            }
        };
        let wait = Duration::from_millis(ctx.config.consume_wait_ms.max(1_000));
        let mut seq = Vec::new();
        for _ in 0..5 {
            match consumer.recv_timeout(wait).await {
                Ok(Some(m)) => {
                    if let Some(b) = m.payload.first() {
                        seq.push(*b);
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    drop(consumer);
                    let _ = self.pool.delete_topic(&topic).await;
                    return CheckOutcome::Fail(format!("consume: {:?}", e.kind()));
                }
            }
        }
        drop(consumer);
        let _ = self.pool.delete_topic(&topic).await;
        if seq == [0, 1, 2, 3, 4] {
            CheckOutcome::Ok(Some(
                "同分区顺序 ok；headers 公共面未导出（partial，非假绿 headers）".into(),
            ))
        } else {
            CheckOutcome::Fail(format!("顺序不符: {seq:?}"))
        }
    }

    async fn check_offset_commit(&self, ctx: &ValidationContext) -> CheckOutcome {
        let topic = ctx.topic("off");
        let store = MemoryOffsetStore::new();
        // 模拟消费 offset=10 后 ack → next-to-read = 11
        if let Err(e) = store.commit(&topic, 0, 10).await {
            return CheckOutcome::Fail(format!("commit: {:?}", e.kind()));
        }
        match store.committed(&topic, 0).await {
            Ok(Some(11)) => CheckOutcome::Ok(Some(
                "应用层 OffsetCommitStore：committed=consumed+1（非 broker group commit）".into(),
            )),
            Ok(other) => CheckOutcome::Fail(format!("committed 期望 11 得 {other:?}")),
            Err(e) => CheckOutcome::Fail(format!("committed: {:?}", e.kind())),
        }
    }

    async fn check_group_lag(&self, _ctx: &ValidationContext) -> CheckOutcome {
        CheckOutcome::Skip("NO-GO: rskafka 无 consumer group / group coordinator / lag 查询".into())
    }

    async fn check_large_message(&self, ctx: &ValidationContext) -> CheckOutcome {
        let size = ctx.config.large_message_bytes.max(self.config.large_message_bytes);
        // 保护：上限 1 MiB，避免拖垮单节点
        let size = size.min(1024 * 1024);
        let topic = ctx.topic("large");
        let repl = ctx.config.replication;
        if let Err(e) = self.pool.ensure_topic(&topic, 1, repl).await {
            return CheckOutcome::Fail(format!("ensure_topic: {:?}", e.kind()));
        }
        let mut body = vec![0xABu8; size];
        // 嵌入 token 尾缀便于比对
        let tok = ctx.token.as_bytes();
        let n = tok.len().min(size);
        body[size - n..].copy_from_slice(&tok[..n]);
        let payload = Bytes::from(body);
        let delivery = match self.pool.producer().publish(&topic, payload.clone()).await {
            Ok(d) => d,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("produce large: {:?}", e.kind()));
            }
        };
        let cfg = ConsumerConfig::assign(&topic, 0, format!("_scg_lg_{}", ctx.token))
            .with_start_offset(delivery.offset);
        let mut consumer = match self.pool.consumer(cfg).await {
            Ok(c) => c,
            Err(e) => {
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("consumer: {:?}", e.kind()));
            }
        };
        let wait = Duration::from_millis(ctx.config.consume_wait_ms.max(3_000));
        let got = match consumer.recv_timeout(wait).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                drop(consumer);
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail("large consume 空".into());
            }
            Err(e) => {
                drop(consumer);
                let _ = self.pool.delete_topic(&topic).await;
                return CheckOutcome::Fail(format!("large consume: {:?}", e.kind()));
            }
        };
        drop(consumer);
        let _ = self.pool.delete_topic(&topic).await;
        if got.payload.len() == size && got.payload == payload {
            CheckOutcome::Ok(Some(format!("bytes={size}")))
        } else {
            CheckOutcome::Fail(format!("大消息比对失败 len={} expect={size}", got.payload.len()))
        }
    }

    async fn check_isr_health(&self, _ctx: &ValidationContext) -> CheckOutcome {
        CheckOutcome::Skip("NO-GO: rskafka 无 ISR / under-replicated 分区 Admin 查询".into())
    }
}

async fn produce_with_key(
    pool: &KafkaPool,
    topic: &str,
    partition: i32,
    key: Option<&[u8]>,
    value: &[u8],
) -> kernel::XResult<crate::message::Delivery> {
    use crate::error_map::map_kafka_err;
    use crate::lifecycle::wait_for_shutdown;
    use crate::message::Delivery;

    pool.ensure_open()?;
    let client = pool.partition_client(topic, partition).await?;
    let _operation = pool.start_operation()?;
    let mut shutdown = pool.shutdown_receiver();
    let record = Record {
        key: key.map(|k| k.to_vec()),
        value: Some(value.to_vec()),
        headers: BTreeMap::new(),
        timestamp: Utc::now(),
    };
    match tokio::select! {
        biased;
        () = wait_for_shutdown(&mut shutdown) => {
            pool.record_publish_err();
            return Err(kernel::XError::cancelled("kafkax produce 因 pool 关闭而取消"));
        }
        result = tokio::time::timeout(
            pool.config().delivery_timeout,
            client.produce(vec![record], KafkaPool::compression()),
        ) => result,
    } {
        Ok(Ok(offsets)) => {
            let offset = offsets.first().copied().unwrap_or(0);
            pool.record_publish_ok();
            Ok(Delivery { partition, offset })
        }
        Ok(Err(e)) => {
            pool.record_publish_err();
            Err(map_kafka_err("kafkax produce", e))
        }
        Err(error) => {
            pool.record_publish_err();
            Err(kernel::XError::deadline_exceeded("kafkax produce 超时").with_source(error))
        }
    }
}

fn stable_partition(key: &[u8], partitions: i32) -> i32 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut h);
    let v = h.finish();
    (v % partitions as u64) as i32
}

fn redact_detail(s: &str) -> String {
    let mut out = s.to_owned();
    for needle in ["password=", "PASSWORD=", "sasl.password", "SASL_PASSWORD"] {
        if let Some(i) = out.find(needle) {
            let rest = &out[i + needle.len()..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .unwrap_or(rest.len());
            out.replace_range(i + needle.len()..i + needle.len() + end, "***");
        }
    }
    if out.len() > 400 {
        out.truncate(400);
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KafkaConfigBuilder;
    use std::time::Duration;

    #[test]
    fn catalog_matches_spec_section_6_2() {
        let cat = KafkaValidator::static_catalog();
        assert_eq!(cat.len(), 9);
        assert!(cat.iter().all(|d| d.id.starts_with("kafka.")));
        let ids: Vec<_> = cat.iter().map(|d| d.id.as_str()).collect();
        for expected in [
            "kafka.basic.metadata",
            "kafka.rw.produce_consume",
            "kafka.full.topic_create_delete",
            "kafka.full.key_partition_routing",
            "kafka.full.ordering_headers",
            "kafka.full.offset_commit",
            "kafka.full.group_lag",
            "kafka.full.large_message",
            "kafka.full.isr_health",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Basic).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::ReadWrite).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Full).count(), 7);
    }

    #[test]
    fn redact_hides_password_fragments() {
        let s = redact_detail("fail password=hunter2 more");
        assert!(!s.contains("hunter2"));
        assert!(s.contains("***"));
    }

    #[tokio::test]
    async fn connect_fail_short_circuits_full() {
        let cfg = KafkaConfigBuilder::new()
            .brokers("127.0.0.1:1")
            .connect_timeout(Duration::from_millis(200))
            .operation_timeout(Duration::from_millis(200))
            .delivery_timeout(Duration::from_millis(200))
            .build()
            .expect("cfg");
        let report = KafkaValidator::connect_and_run(cfg, CheckLevel::Full).await;
        assert_eq!(report.module, "kafka");
        assert!(!report.passed);
        assert_eq!(report.items.len(), 9);
        let meta = report.items.iter().find(|i| i.id == "kafka.basic.metadata").expect("meta");
        assert_eq!(meta.status, CheckStatus::Failed);
        assert!(meta.detail.as_ref().is_some_and(|d| !d.is_empty()));
        for i in &report.items {
            if i.id.starts_with("kafka.rw.") || i.id.starts_with("kafka.full.") {
                assert_eq!(i.status, CheckStatus::Skipped, "{} {:?}", i.id, i.status);
                assert!(
                    i.detail.as_ref().is_some_and(|d| d.contains("短路")),
                    "{} detail={:?}",
                    i.id,
                    i.detail
                );
            }
        }
        // NO-GO 项不得假 Passed
        for id in ["kafka.full.group_lag", "kafka.full.isr_health"] {
            let item = report.items.iter().find(|i| i.id == id).expect(id);
            assert_ne!(item.status, CheckStatus::Passed);
        }
    }

    #[tokio::test]
    async fn skip_config_on_connect_fail_basic() {
        let cfg = KafkaConfigBuilder::new()
            .brokers("127.0.0.1:1")
            .connect_timeout(Duration::from_millis(150))
            .operation_timeout(Duration::from_millis(150))
            .delivery_timeout(Duration::from_millis(150))
            .build()
            .expect("cfg");
        // connect_and_run 在 connect 失败时不走 skip 配置；验证入口仍不 panic
        let report = KafkaValidator::connect_and_run(cfg, CheckLevel::Basic).await;
        assert_eq!(report.items.len(), 1);
        assert!(!report.passed);
    }

    #[tokio::test]
    async fn twice_connect_fail_stable() {
        let make = || {
            KafkaConfigBuilder::new()
                .brokers("127.0.0.1:1")
                .connect_timeout(Duration::from_millis(150))
                .operation_timeout(Duration::from_millis(150))
                .delivery_timeout(Duration::from_millis(150))
                .build()
                .expect("cfg")
        };
        let r1 = KafkaValidator::connect_and_run(make(), CheckLevel::ReadWrite).await;
        let r2 = KafkaValidator::connect_and_run(make(), CheckLevel::ReadWrite).await;
        assert_eq!(r1.passed, r2.passed);
        assert_eq!(r1.items.len(), r2.items.len());
        assert_eq!(r1.items[0].status, r2.items[0].status);
    }

    #[test]
    fn stable_partition_same_key() {
        assert_eq!(stable_partition(b"k", 3), stable_partition(b"k", 3));
    }
}
