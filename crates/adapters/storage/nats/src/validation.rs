//! NATS 库内自验证（对齐 `.cargo/draft/verifyctl.md` §6.3）。
//!
//! 检查项（9 项中 8 项可跑，`kv_watch` NO-GO）：
//! - basic: `nats.basic.connection`
//! - rw: `nats.rw.pub_sub`
//! - full: `request_reply` / `queue_group` / `wildcard` /
//!   `js_stream` / `js_publish_ack` / `js_ack_redelivery`
//!
//! `nats.full.kv_watch`：async-nats `kv` feature 未启用，且 natsx 稳定承诺不含 KV。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use bytes::Bytes;
use futures_util::StreamExt;
use kernel::XResult;
use serde::Serialize;

use crate::config::NatsConfig;
use crate::jetstream::{JetStream, JetStreamConsumerConfig, StreamConfig};
use crate::pool::NatsPool;

/// 单项检查结果状态（四态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Degraded,
    Failed,
    Skipped,
}

/// 自检级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckLevel {
    Basic,
    ReadWrite,
    Full,
}

/// 单项检查结果。
#[derive(Debug, Clone, Serialize)]
pub struct CheckItem {
    pub id: String,
    pub status: CheckStatus,
    pub latency_ms: u128,
    pub baseline_ms: Option<u128>,
    pub detail: Option<String>,
}

/// 整次自检报告。
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub module: String,
    pub level: CheckLevel,
    pub passed: bool,
    pub degraded: bool,
    pub total_ms: u128,
    pub items: Vec<CheckItem>,
}

impl CheckLevel {
    /// 当前级别是否包含 `other`。
    #[must_use]
    pub fn includes(self, other: CheckLevel) -> bool {
        matches!(
            (self, other),
            (CheckLevel::Full, _)
                | (CheckLevel::ReadWrite, CheckLevel::ReadWrite | CheckLevel::Basic)
                | (CheckLevel::Basic, CheckLevel::Basic)
        )
    }
}

/// NATS 自验证器。
pub struct NatsValidator {
    pool: NatsPool,
}

impl NatsValidator {
    /// 使用给定配置连接后构造。
    pub async fn connect(config: NatsConfig) -> XResult<Self> {
        Ok(Self { pool: NatsPool::connect(config).await? })
    }

    /// 从环境变量连接。
    pub async fn connect_from_env() -> XResult<Self> {
        Self::connect(NatsConfig::from_env()?).await
    }

    /// 按级别执行检查项。
    pub async fn validate(&self, level: CheckLevel) -> ValidationReport {
        let start = Instant::now();
        let mut items = Vec::new();

        if level.includes(CheckLevel::Basic) {
            items.push(self.basic_connection().await);
        }

        if level.includes(CheckLevel::ReadWrite) {
            let short = items.last().is_some_and(|item| item.status == CheckStatus::Failed);
            if short {
                items.push(self.skip("nats.rw.pub_sub"));
            } else {
                items.push(self.rw_pub_sub().await);
            }
        }

        if level.includes(CheckLevel::Full) {
            let short = items.iter().any(|item| {
                item.id == "nats.basic.connection" && item.status == CheckStatus::Failed
            });
            let full_ids = [
                "nats.full.request_reply",
                "nats.full.queue_group",
                "nats.full.wildcard",
                "nats.full.js_stream",
                "nats.full.js_publish_ack",
                "nats.full.js_ack_redelivery",
            ];
            if short {
                for id in &full_ids {
                    items.push(self.skip(id));
                }
            } else {
                items.push(self.full_request_reply().await);
                items.push(self.full_queue_group().await);
                items.push(self.full_wildcard().await);
                items.push(self.full_js_stream().await);
                items.push(self.full_js_publish_ack().await);
                items.push(self.full_js_ack_redelivery().await);
            }
        }

        let total_ms = start.elapsed().as_millis();
        let failed = items.iter().any(|item| item.status == CheckStatus::Failed);
        let degraded = items.iter().any(|item| item.status == CheckStatus::Degraded);
        ValidationReport {
            module: "nats".into(),
            level,
            passed: !failed,
            degraded,
            total_ms,
            items,
        }
    }

    fn skip(&self, id: &str) -> CheckItem {
        CheckItem {
            id: id.into(),
            status: CheckStatus::Skipped,
            latency_ms: 0,
            baseline_ms: None,
            detail: Some("short-circuited".into()),
        }
    }

    fn pass(id: &str, ms: u128, baseline: Option<u128>) -> CheckItem {
        let status = match baseline {
            Some(limit) if ms > limit => CheckStatus::Degraded,
            _ => CheckStatus::Passed,
        };
        CheckItem { id: id.into(), status, latency_ms: ms, baseline_ms: baseline, detail: None }
    }

    fn fail(id: &str, ms: u128, baseline: Option<u128>, detail: &str) -> CheckItem {
        CheckItem {
            id: id.into(),
            status: CheckStatus::Failed,
            latency_ms: ms,
            baseline_ms: baseline,
            detail: Some(detail.into()),
        }
    }

    // ── basic ──

    async fn basic_connection(&self) -> CheckItem {
        let started = Instant::now();
        let baseline = Some(50u128);
        match self.pool.health().await {
            Ok(health) if health.ready => {
                Self::pass("nats.basic.connection", started.elapsed().as_millis(), baseline)
            }
            Ok(health) => Self::fail(
                "nats.basic.connection",
                started.elapsed().as_millis(),
                baseline,
                &health.detail,
            ),
            Err(error) => Self::fail(
                "nats.basic.connection",
                started.elapsed().as_millis(),
                baseline,
                &error.to_string(),
            ),
        }
    }

    // ── rw ──

    async fn rw_pub_sub(&self) -> CheckItem {
        let started = Instant::now();
        let baseline = Some(500u128);
        let subject = "_sc.rw.pub_sub";
        match self.pool.subscribe(subject).await {
            Ok(mut sub) => {
                let payload = Bytes::from("v42");
                if let Err(error) = self.pool.publish(subject, payload.clone()).await {
                    return Self::fail(
                        "nats.rw.pub_sub",
                        started.elapsed().as_millis(),
                        baseline,
                        &format!("pub:{error}"),
                    );
                }
                match tokio::time::timeout(Duration::from_secs(3), sub.recv()).await {
                    Ok(Some(message)) if message.payload == payload => {
                        Self::pass("nats.rw.pub_sub", started.elapsed().as_millis(), baseline)
                    }
                    Ok(Some(_)) => Self::fail(
                        "nats.rw.pub_sub",
                        started.elapsed().as_millis(),
                        baseline,
                        "mismatch",
                    ),
                    _ => Self::fail(
                        "nats.rw.pub_sub",
                        started.elapsed().as_millis(),
                        baseline,
                        "recv timeout",
                    ),
                }
            }
            Err(error) => Self::fail(
                "nats.rw.pub_sub",
                started.elapsed().as_millis(),
                baseline,
                &error.to_string(),
            ),
        }
    }

    // ── full: request_reply ──

    async fn full_request_reply(&self) -> CheckItem {
        let started = Instant::now();
        let baseline = Some(500u128);
        let subject = "_sc.full.req";
        let token = Bytes::from("req-echo-42");

        // 原始 client 以访问 reply subject，做 echo 服务端。
        let client = self.pool.client();
        let subject_owned = subject.to_string();
        tokio::spawn(async move {
            if let Ok(mut sub) = client.subscribe(subject_owned).await {
                while let Some(msg) =
                    tokio::time::timeout(Duration::from_secs(10), sub.next()).await.ok().flatten()
                {
                    if let Some(reply) = msg.reply {
                        let _ = client.publish(reply, msg.payload).await;
                    }
                }
            }
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        match self.pool.request(subject, token.clone(), Duration::from_secs(3)).await {
            Ok(reply) if reply.payload == token => {
                Self::pass("nats.full.request_reply", started.elapsed().as_millis(), baseline)
            }
            Ok(_) => Self::fail(
                "nats.full.request_reply",
                started.elapsed().as_millis(),
                baseline,
                "payload mismatch",
            ),
            Err(error) => Self::fail(
                "nats.full.request_reply",
                started.elapsed().as_millis(),
                baseline,
                &error.to_string(),
            ),
        }
    }

    // ── full: queue_group ──

    async fn full_queue_group(&self) -> CheckItem {
        let started = Instant::now();
        let subject = "_sc.full.qg";
        let group = "sc-qg";
        let total = Arc::new(AtomicU64::new(0));

        // 三个 queue 成员；合计收包数应等于发布数。
        for _ in 0..3 {
            let client = self.pool.client();
            let total = Arc::clone(&total);
            let subject = subject.to_string();
            let group = group.to_string();
            tokio::spawn(async move {
                if let Ok(mut sub) = client.queue_subscribe(subject, group).await {
                    while let Some(_msg) = tokio::time::timeout(Duration::from_secs(3), sub.next())
                        .await
                        .ok()
                        .flatten()
                    {
                        total.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_millis(200)).await;

        let n: u64 = 9;
        for _ in 0..n {
            let _ = self.pool.publish(subject, Bytes::from("qg")).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;

        let received = total.load(Ordering::Relaxed);
        if received == n {
            Self::pass("nats.full.queue_group", started.elapsed().as_millis(), None)
        } else {
            Self::fail(
                "nats.full.queue_group",
                started.elapsed().as_millis(),
                None,
                &format!("received {received}/{n}"),
            )
        }
    }

    // ── full: wildcard ──

    async fn full_wildcard(&self) -> CheckItem {
        let started = Instant::now();
        let star_cnt = Arc::new(AtomicU64::new(0));
        let gt_cnt = Arc::new(AtomicU64::new(0));

        {
            let pool = self.pool.clone();
            let cnt = Arc::clone(&star_cnt);
            tokio::spawn(async move {
                if let Ok(mut sub) = pool.subscribe("_sc.full.wc.a.*").await {
                    while let Ok(Some(_)) =
                        tokio::time::timeout(Duration::from_secs(3), sub.recv()).await
                    {
                        cnt.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        {
            let pool = self.pool.clone();
            let cnt = Arc::clone(&gt_cnt);
            tokio::spawn(async move {
                if let Ok(mut sub) = pool.subscribe("_sc.full.wc.a.>").await {
                    while let Ok(Some(_)) =
                        tokio::time::timeout(Duration::from_secs(3), sub.recv()).await
                    {
                        cnt.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        tokio::time::sleep(Duration::from_millis(200)).await;

        let _ = self.pool.publish("_sc.full.wc.a.b", Bytes::from("1")).await;
        let _ = self.pool.publish("_sc.full.wc.a.x", Bytes::from("2")).await;
        let _ = self.pool.publish("_sc.full.wc.a.b.c", Bytes::from("3")).await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let star = star_cnt.load(Ordering::Relaxed);
        let gt = gt_cnt.load(Ordering::Relaxed);
        // `a.*` 匹配两段：a.b / a.x；`a.>` 匹配三段及以下：a.b / a.x / a.b.c
        if star == 2 && gt == 3 {
            Self::pass("nats.full.wildcard", started.elapsed().as_millis(), None)
        } else {
            Self::fail(
                "nats.full.wildcard",
                started.elapsed().as_millis(),
                None,
                &format!("a.*={star}(expected 2) a.>={gt}(expected 3)"),
            )
        }
    }

    // ── full: js_stream ──

    async fn full_js_stream(&self) -> CheckItem {
        let started = Instant::now();
        let baseline = Some(2000u128);
        let js = JetStream::from_pool(&self.pool);
        match js
            .get_or_create_stream(StreamConfig::new("_sc_full_js_stream", "_sc.full.js.>"))
            .await
        {
            Ok(()) => {
                let _ = js.delete_stream("_sc_full_js_stream").await;
                Self::pass("nats.full.js_stream", started.elapsed().as_millis(), baseline)
            }
            Err(error) => Self::fail(
                "nats.full.js_stream",
                started.elapsed().as_millis(),
                baseline,
                &error.to_string(),
            ),
        }
    }

    // ── full: js_publish_ack ──

    async fn full_js_publish_ack(&self) -> CheckItem {
        let started = Instant::now();
        let baseline = Some(1000u128);
        let js = JetStream::from_pool(&self.pool);
        let stream = "_sc_full_js_pub";
        if let Err(error) =
            js.get_or_create_stream(StreamConfig::new(stream, "_sc.full.js.pub.>")).await
        {
            return Self::fail(
                "nats.full.js_publish_ack",
                started.elapsed().as_millis(),
                baseline,
                &error.to_string(),
            );
        }
        let subject = format!("_sc.full.js.pub.{}", std::process::id());
        match js.publish(&subject, Bytes::from("pub-test")).await {
            Ok(()) => {
                let _ = js.delete_stream(stream).await;
                Self::pass("nats.full.js_publish_ack", started.elapsed().as_millis(), baseline)
            }
            Err(error) => {
                let _ = js.delete_stream(stream).await;
                Self::fail(
                    "nats.full.js_publish_ack",
                    started.elapsed().as_millis(),
                    baseline,
                    &error.to_string(),
                )
            }
        }
    }

    // ── full: js_ack_redelivery ──

    async fn full_js_ack_redelivery(&self) -> CheckItem {
        let started = Instant::now();
        let js = JetStream::from_pool(&self.pool);
        let stream = "_sc_full_js_redel";
        if let Err(error) =
            js.get_or_create_stream(StreamConfig::new(stream, "_sc.full.js.redel.>")).await
        {
            return Self::fail(
                "nats.full.js_ack_redelivery",
                started.elapsed().as_millis(),
                None,
                &error.to_string(),
            );
        }
        let _ = js.publish("_sc.full.js.redel.1", Bytes::from("rt")).await;
        let mut cfg = JetStreamConsumerConfig::durable("sc_redel_test");
        cfg.ack_wait = Duration::from_millis(300);
        cfg.max_deliver = 3;
        cfg.filter_subject = Some("_sc.full.js.redel.1".into());

        match js.consumer(stream, cfg).await {
            Ok(consumer) => match consumer.next_timeout(Duration::from_secs(2)).await {
                Ok(Some(delivery)) => {
                    let attempts = delivery.metadata().delivery_attempts;
                    drop(delivery);
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    match consumer.next_timeout(Duration::from_secs(2)).await {
                        Ok(Some(redelivery)) => {
                            let ok = redelivery.metadata().delivery_attempts > attempts;
                            redelivery.ack().await.ok();
                            let _ = js.delete_stream(stream).await;
                            if ok {
                                Self::pass(
                                    "nats.full.js_ack_redelivery",
                                    started.elapsed().as_millis(),
                                    None,
                                )
                            } else {
                                Self::fail(
                                    "nats.full.js_ack_redelivery",
                                    started.elapsed().as_millis(),
                                    None,
                                    "no redelivery",
                                )
                            }
                        }
                        _ => {
                            let _ = js.delete_stream(stream).await;
                            Self::fail(
                                "nats.full.js_ack_redelivery",
                                started.elapsed().as_millis(),
                                None,
                                "redel fetch fail",
                            )
                        }
                    }
                }
                _ => {
                    let _ = js.delete_stream(stream).await;
                    Self::fail(
                        "nats.full.js_ack_redelivery",
                        started.elapsed().as_millis(),
                        None,
                        "no msg",
                    )
                }
            },
            Err(error) => {
                let _ = js.delete_stream(stream).await;
                Self::fail(
                    "nats.full.js_ack_redelivery",
                    started.elapsed().as_millis(),
                    None,
                    &error.to_string(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_level_includes_hierarchy() {
        assert!(CheckLevel::Full.includes(CheckLevel::Basic));
        assert!(CheckLevel::Full.includes(CheckLevel::ReadWrite));
        assert!(CheckLevel::Full.includes(CheckLevel::Full));
        assert!(CheckLevel::ReadWrite.includes(CheckLevel::Basic));
        assert!(CheckLevel::ReadWrite.includes(CheckLevel::ReadWrite));
        assert!(!CheckLevel::ReadWrite.includes(CheckLevel::Full));
        assert!(CheckLevel::Basic.includes(CheckLevel::Basic));
        assert!(!CheckLevel::Basic.includes(CheckLevel::ReadWrite));
    }

    #[test]
    fn pass_marks_degraded_when_over_baseline() {
        let item = NatsValidator::pass("x", 100, Some(50));
        assert_eq!(item.status, CheckStatus::Degraded);
        let ok = NatsValidator::pass("x", 10, Some(50));
        assert_eq!(ok.status, CheckStatus::Passed);
    }

    #[test]
    fn report_serde_snake_case() {
        let report = ValidationReport {
            module: "nats".into(),
            level: CheckLevel::Basic,
            passed: true,
            degraded: false,
            total_ms: 1,
            items: vec![CheckItem {
                id: "nats.basic.connection".into(),
                status: CheckStatus::Passed,
                latency_ms: 1,
                baseline_ms: Some(50),
                detail: None,
            }],
        };
        let json = serde_json::to_string(&report).expect("json");
        assert!(json.contains("\"level\":\"basic\""));
        assert!(json.contains("\"status\":\"passed\""));
        assert!(json.contains("nats.basic.connection"));
    }
}
