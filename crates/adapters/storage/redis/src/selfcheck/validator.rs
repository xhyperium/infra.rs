//! redisx [`Validatable`] 实现与 §6.5 检查项执行。

use std::time::{Duration, Instant};

#[cfg(feature = "pubsub")]
use futures_util::StreamExt;
use kernel::{ErrorKind, XError};
use redis::AsyncCommands;
use tokio::time::{sleep, timeout};

use crate::client::RedisClient;
use crate::error_map::map_redis_result;
use crate::pool::RedisBackend;

use super::config::RedisSelfCheckConfig;
use super::context::ValidationContext;
use super::types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};

/// 模块名（规范 3.1）。
pub const MODULE: &str = "redisx";

/// 检查返回：`Ok` 通过；`Err(Skip)` 拓扑/feature 不适用；`Err(Fail)` 失败。
enum CheckOutcome {
    Ok,
    Skip(String),
    Fail(String),
}

/// redisx 验证器：绑定 [`RedisClient`] + 配置。
#[derive(Clone, Debug)]
pub struct RedisValidator {
    client: RedisClient,
    config: RedisSelfCheckConfig,
}

/// 验证器契约（规范 §5.2 子集，redisx 内聚）。
#[async_trait::async_trait]
pub trait Validatable: Send + Sync {
    fn module_name(&self) -> &'static str;
    fn catalog(&self) -> Vec<CheckDescriptor>;
    async fn validate(&self, ctx: &ValidationContext, level: CheckLevel) -> ValidationReport;
}

impl RedisValidator {
    #[must_use]
    pub fn new(client: RedisClient) -> Self {
        Self { client, config: RedisSelfCheckConfig::default() }
    }

    #[must_use]
    pub fn with_config(mut self, config: RedisSelfCheckConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn config(&self) -> &RedisSelfCheckConfig {
        &self.config
    }

    #[must_use]
    pub fn client(&self) -> &RedisClient {
        &self.client
    }

    /// 使用默认上下文执行验证（生成 run_id/token）。
    pub async fn run(&self, level: CheckLevel) -> ValidationReport {
        let ctx = ValidationContext::new(self.config.clone());
        self.validate(&ctx, level).await
    }

    /// 静态 catalog（§6.5）。
    #[must_use]
    pub fn static_catalog() -> Vec<CheckDescriptor> {
        vec![
            desc("redisx.basic.ping", CheckLevel::Basic, Some(20), "PING → PONG", false),
            desc(
                "redisx.rw.set_get_del",
                CheckLevel::ReadWrite,
                Some(50),
                "带 TTL 的 KV 闭环 set/get/del",
                false,
            ),
            desc("redisx.full.ttl_semantics", CheckLevel::Full, None, "PX 过期后 GET 为空", false),
            desc(
                "redisx.full.data_structures",
                CheckLevel::Full,
                Some(100),
                "hash/list/set/zset 基本操作",
                false,
            ),
            desc(
                "redisx.full.pipeline",
                CheckLevel::Full,
                Some(100),
                "管道批量 SET 结果顺序正确",
                false,
            ),
            desc(
                "redisx.full.multi_exec",
                CheckLevel::Full,
                None,
                "MULTI/EXEC 事务原子读回",
                false,
            ),
            desc("redisx.full.lua_cas", CheckLevel::Full, None, "EVAL CAS 脚本语义", false),
            desc("redisx.full.pubsub", CheckLevel::Full, Some(1_000), "发布订阅闭环", false),
            desc("redisx.full.dist_lock", CheckLevel::Full, None, "加锁/互斥/释放/误删保护", false),
            desc(
                "redisx.full.memory_pressure",
                CheckLevel::Full,
                None,
                "used/maxmemory 比例阈值",
                false,
            ),
            desc(
                "redisx.full.cluster_slots",
                CheckLevel::Full,
                None,
                "16384 槽全覆盖（仅 Cluster）",
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

#[async_trait::async_trait]
impl Validatable for RedisValidator {
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

impl RedisValidator {
    async fn run_checks(
        &self,
        ctx: &ValidationContext,
        level: CheckLevel,
        items: &mut Vec<CheckItem>,
    ) {
        if level.includes(CheckLevel::Basic) {
            items.push(
                self.exec_check(ctx, "redisx.basic.ping", Some(20), self.check_ping(ctx)).await,
            );
        }

        let basic_failed = items
            .iter()
            .any(|i| i.id.starts_with("redisx.basic.") && i.status == CheckStatus::Failed);

        if level.includes(CheckLevel::ReadWrite) {
            if basic_failed {
                items.push(CheckItem::skipped(
                    "redisx.rw.set_get_del",
                    "短路：basic 失败",
                    now_rfc3339(),
                ));
            } else {
                items.push(
                    self.exec_check(
                        ctx,
                        "redisx.rw.set_get_del",
                        Some(50),
                        self.check_set_get_del(ctx),
                    )
                    .await,
                );
            }
        }

        if !level.includes(CheckLevel::Full) {
            return;
        }

        let full_ids = [
            "redisx.full.ttl_semantics",
            "redisx.full.data_structures",
            "redisx.full.pipeline",
            "redisx.full.multi_exec",
            "redisx.full.lua_cas",
            "redisx.full.pubsub",
            "redisx.full.dist_lock",
            "redisx.full.memory_pressure",
            "redisx.full.cluster_slots",
        ];

        if basic_failed {
            for id in full_ids {
                items.push(CheckItem::skipped(id, "短路：basic 失败", now_rfc3339()));
            }
            return;
        }

        items.push(
            self.exec_check(ctx, "redisx.full.ttl_semantics", None, self.check_ttl_semantics(ctx))
                .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "redisx.full.data_structures",
                Some(100),
                self.check_data_structures(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "redisx.full.pipeline", Some(100), self.check_pipeline(ctx)).await,
        );
        items.push(
            self.exec_check(ctx, "redisx.full.multi_exec", None, self.check_multi_exec(ctx)).await,
        );
        items
            .push(self.exec_check(ctx, "redisx.full.lua_cas", None, self.check_lua_cas(ctx)).await);
        items.push(
            self.exec_check(ctx, "redisx.full.pubsub", Some(1_000), self.check_pubsub(ctx)).await,
        );
        items.push(
            self.exec_check(ctx, "redisx.full.dist_lock", None, self.check_dist_lock(ctx)).await,
        );
        items.push(
            self.exec_check(
                ctx,
                "redisx.full.memory_pressure",
                None,
                self.check_memory_pressure(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "redisx.full.cluster_slots", None, self.check_cluster_slots(ctx))
                .await,
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
            CheckOutcome::Ok => {
                CheckItem::finish(id, CheckStatus::Passed, latency, baseline, None, started_at)
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

    async fn check_ping(&self, _ctx: &ValidationContext) -> CheckOutcome {
        match self.client.pool().ping().await {
            Ok(_) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(format!("ping 失败: {:?}", e.kind())),
        }
    }

    async fn check_set_get_del(&self, ctx: &ValidationContext) -> CheckOutcome {
        let key = ctx.key("kv");
        let val = format!("v-{}", ctx.token).into_bytes();
        let ttl = Duration::from_secs(60);
        if let Err(e) = self.client.set(&key, val.clone(), Some(ttl)).await {
            return CheckOutcome::Fail(format!("set: {:?}", e.kind()));
        }
        match self.client.get(&key).await {
            Ok(Some(g)) if g == val => {}
            Ok(_) => return CheckOutcome::Fail("set/get 比对失败".into()),
            Err(e) => return CheckOutcome::Fail(format!("get: {:?}", e.kind())),
        }
        match self.client.delete(&key).await {
            Ok(true) => {}
            Ok(false) => return CheckOutcome::Fail("del 未删除 key".into()),
            Err(e) => return CheckOutcome::Fail(format!("del: {:?}", e.kind())),
        }
        match self.client.get(&key).await {
            Ok(None) => CheckOutcome::Ok,
            Ok(Some(_)) => CheckOutcome::Fail("del 后仍可读".into()),
            Err(e) => CheckOutcome::Fail(format!("get2: {:?}", e.kind())),
        }
    }

    async fn check_ttl_semantics(&self, ctx: &ValidationContext) -> CheckOutcome {
        let key = ctx.key("ttl");
        let expire_ms = ctx.config.ttl_expire_ms.max(50);
        let wait_ms = ctx.config.ttl_wait_ms.max(expire_ms + 50);
        if let Err(e) =
            self.client.set(&key, b"ttl-val".to_vec(), Some(Duration::from_millis(expire_ms))).await
        {
            return CheckOutcome::Fail(format!("set ttl: {:?}", e.kind()));
        }
        sleep(Duration::from_millis(wait_ms)).await;
        match self.client.get(&key).await {
            Ok(None) => CheckOutcome::Ok,
            Ok(Some(_)) => {
                let _ = self.client.delete(&key).await;
                CheckOutcome::Fail("TTL 到期后仍可读".into())
            }
            Err(e) => CheckOutcome::Fail(format!("get: {:?}", e.kind())),
        }
    }

    async fn check_data_structures(&self, ctx: &ValidationContext) -> CheckOutcome {
        let h = ctx.key("h");
        let l = ctx.key("l");
        let s = ctx.key("s");
        let z = ctx.key("z");
        let r = self
            .client
            .with_pool_conn(|mut conn: RedisBackend| async move {
                let _: i32 = map_redis_result(conn.hset(&h, "f", b"hv").await)?;
                let hv: Option<Vec<u8>> = map_redis_result(conn.hget(&h, "f").await)?;
                if hv.as_deref() != Some(b"hv") {
                    return Err(XError::internal("hash 读回不一致"));
                }
                let _: i32 = map_redis_result(conn.lpush(&l, b"lv").await)?;
                let lv: Option<Vec<u8>> = map_redis_result(conn.lpop(&l, None).await)?;
                if lv.as_deref() != Some(b"lv") {
                    return Err(XError::internal("list 读回不一致"));
                }
                let _: i32 = map_redis_result(conn.sadd(&s, b"sv").await)?;
                let ok: bool = map_redis_result(conn.sismember(&s, b"sv").await)?;
                if !ok {
                    return Err(XError::internal("set 成员缺失"));
                }
                let _: i32 = map_redis_result(conn.zadd(&z, b"zv", 1.5).await)?;
                let sc: Option<f64> = map_redis_result(conn.zscore(&z, b"zv").await)?;
                if sc.map(|x| (x - 1.5).abs() > 1e-9).unwrap_or(true) {
                    return Err(XError::internal("zset score 不一致"));
                }
                for k in [&h, &l, &s, &z] {
                    let _: bool = map_redis_result(conn.expire(k.as_str(), 300).await)?;
                }
                Ok(())
            })
            .await;
        match r {
            Ok(()) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(format!("data_structures: {:?}", e.kind())),
        }
    }

    async fn check_pipeline(&self, ctx: &ValidationContext) -> CheckOutcome {
        let k1 = ctx.key("p1");
        let k2 = ctx.key("p2");
        let k3 = ctx.key("p3");
        let items = [
            (k1.as_str(), b"a".to_vec()),
            (k2.as_str(), b"b".to_vec()),
            (k3.as_str(), b"c".to_vec()),
        ];
        if let Err(e) = self.client.pipeline_set(&items, Some(Duration::from_secs(60))).await {
            return CheckOutcome::Fail(format!("pipeline_set: {:?}", e.kind()));
        }
        match self.client.mget(&[k1.as_str(), k2.as_str(), k3.as_str()]).await {
            Ok(got)
                if got.len() == 3
                    && got[0].as_deref() == Some(b"a")
                    && got[1].as_deref() == Some(b"b")
                    && got[2].as_deref() == Some(b"c") =>
            {
                let _ = self.client.delete(&k1).await;
                let _ = self.client.delete(&k2).await;
                let _ = self.client.delete(&k3).await;
                CheckOutcome::Ok
            }
            Ok(_) => CheckOutcome::Fail("pipeline 顺序/内容不正确".into()),
            Err(e) => CheckOutcome::Fail(format!("mget: {:?}", e.kind())),
        }
    }

    async fn check_multi_exec(&self, ctx: &ValidationContext) -> CheckOutcome {
        let key = ctx.key("tx");
        let val = b"tx-ok".to_vec();
        let r = self
            .client
            .with_pool_conn({
                let key = key.clone();
                let val = val.clone();
                move |mut conn: RedisBackend| async move {
                    let mut pipe = redis::pipe();
                    pipe.atomic();
                    pipe.cmd("SET").arg(&key).arg(&val).ignore();
                    pipe.cmd("GET").arg(&key);
                    pipe.cmd("PEXPIRE").arg(&key).arg(60_000u64).ignore();
                    let results: Vec<redis::Value> =
                        map_redis_result(pipe.query_async(&mut conn).await)?;
                    let ok = results.iter().any(|v| match v {
                        redis::Value::BulkString(b) => b.as_slice() == val.as_slice(),
                        redis::Value::SimpleString(s) => s.as_bytes() == val.as_slice(),
                        _ => false,
                    });
                    if !ok {
                        let got: Option<Vec<u8>> = map_redis_result(conn.get(&key).await)?;
                        if got.as_ref() != Some(&val) {
                            return Err(XError::internal("multi/exec 读回不一致"));
                        }
                    }
                    Ok(())
                }
            })
            .await;
        let _ = self.client.delete(&key).await;
        match r {
            Ok(()) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(format!("multi_exec: {:?}", e.kind())),
        }
    }

    async fn check_lua_cas(&self, ctx: &ValidationContext) -> CheckOutcome {
        let key = ctx.key("cas");
        const CAS: &str = r#"
local cur = redis.call('GET', KEYS[1])
if (not cur) or cur == ARGV[1] then
  redis.call('SET', KEYS[1], ARGV[2], 'PX', ARGV[3])
  return 1
end
return 0
"#;
        if let Err(e) = self.client.set(&key, b"old".to_vec(), Some(Duration::from_secs(60))).await
        {
            return CheckOutcome::Fail(format!("seed: {:?}", e.kind()));
        }
        match self.client.eval_script(CAS, &[&key], &[b"old", b"new", b"60000"]).await {
            Ok(v) if value_truthy(&v) => {}
            Ok(_) => return CheckOutcome::Fail("CAS 期望成功".into()),
            Err(e) => return CheckOutcome::Fail(format!("cas ok: {:?}", e.kind())),
        }
        match self.client.eval_script(CAS, &[&key], &[b"wrong", b"x", b"60000"]).await {
            Ok(v) if !value_truthy(&v) => {}
            Ok(_) => return CheckOutcome::Fail("CAS 期望失败".into()),
            Err(e) => return CheckOutcome::Fail(format!("cas fail: {:?}", e.kind())),
        }
        let outcome = match self.client.get(&key).await {
            Ok(Some(g)) if g == b"new" => CheckOutcome::Ok,
            Ok(_) => CheckOutcome::Fail("CAS 后值不正确".into()),
            Err(e) => CheckOutcome::Fail(format!("get: {:?}", e.kind())),
        };
        let _ = self.client.delete(&key).await;
        outcome
    }

    async fn check_pubsub(&self, ctx: &ValidationContext) -> CheckOutcome {
        #[cfg(not(feature = "pubsub"))]
        {
            let _ = ctx;
            return CheckOutcome::Skip("feature `pubsub` 未启用".into());
        }
        #[cfg(feature = "pubsub")]
        {
            let channel = format!("_sc.{}:ch", ctx.token);
            let wait = Duration::from_millis(ctx.config.pubsub_wait_ms.max(500));
            let session = match self.client.pool().subscribe([channel.clone()]).await {
                Ok(s) => s,
                Err(e) => {
                    return CheckOutcome::Fail(format!("subscribe: {:?}", e.kind()));
                }
            };
            let mut stream = match session.into_message_stream() {
                Ok(s) => s,
                Err(e) => return CheckOutcome::Fail(format!("stream: {:?}", e.kind())),
            };
            let payload = format!("msg-{}", ctx.token);
            let ch = channel.clone();
            let pl = payload.clone();
            if let Err(e) = self
                .client
                .with_pool_conn(move |mut conn: RedisBackend| async move {
                    let _: i64 = map_redis_result(
                        redis::cmd("PUBLISH")
                            .arg(&ch)
                            .arg(pl.as_bytes())
                            .query_async(&mut conn)
                            .await,
                    )?;
                    Ok(())
                })
                .await
            {
                return CheckOutcome::Fail(format!("publish: {:?}", e.kind()));
            }

            let recv = timeout(wait, async {
                while let Some(msg) = stream.next().await {
                    if msg.payload.as_ref() == payload.as_bytes() {
                        return true;
                    }
                }
                false
            })
            .await;

            match recv {
                Ok(true) => CheckOutcome::Ok,
                Ok(false) => CheckOutcome::Fail("未收到期望消息".into()),
                Err(_) => CheckOutcome::Fail("pubsub 等待超时".into()),
            }
        }
    }

    async fn check_dist_lock(&self, ctx: &ValidationContext) -> CheckOutcome {
        let key = ctx.key("lock");
        let ttl = Duration::from_secs(30);
        let lock = match self.client.lock_acquire(&key, ttl).await {
            Ok(l) => l,
            Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
        };
        match self.client.lock_acquire(&key, ttl).await {
            Err(e) if e.kind() == ErrorKind::Conflict => {}
            other => {
                let _ = self.client.lock_release(&lock).await;
                return CheckOutcome::Fail(format!("二次加锁应 Conflict，得到 {other:?}"));
            }
        }
        // 误删保护：错误 token 的 compare-and-delete
        match self
                .client
                .eval_script(
                    r#"if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) else return 0 end"#,
                    &[&key],
                    &[b"not-owner"],
                )
                .await
            {
                Ok(v) if !value_truthy(&v) => {}
                Ok(_) => {
                    let _ = self.client.lock_release(&lock).await;
                    return CheckOutcome::Fail("非 owner 不应删除锁".into());
                }
                Err(e) => {
                    let _ = self.client.lock_release(&lock).await;
                    return CheckOutcome::Fail(format!("wrong release: {:?}", e.kind()));
                }
            }
        match self.client.lock_release(&lock).await {
            Ok(true) => {}
            Ok(false) => return CheckOutcome::Fail("owner 释放失败".into()),
            Err(e) => return CheckOutcome::Fail(format!("release: {:?}", e.kind())),
        }
        match self.client.lock_acquire(&key, ttl).await {
            Ok(lock2) => {
                let _ = self.client.lock_release(&lock2).await;
                CheckOutcome::Ok
            }
            Err(e) => CheckOutcome::Fail(format!("reacquire: {:?}", e.kind())),
        }
    }

    async fn check_memory_pressure(&self, ctx: &ValidationContext) -> CheckOutcome {
        let ratio_limit = ctx.config.max_memory_ratio;
        let info = match self
            .client
            .with_pool_conn(|mut conn: RedisBackend| async move {
                map_redis_result::<String>(
                    redis::cmd("INFO").arg("memory").query_async(&mut conn).await,
                )
            })
            .await
        {
            Ok(s) => s,
            Err(e) => return CheckOutcome::Fail(format!("INFO memory: {:?}", e.kind())),
        };
        let Some(used) = parse_info_u64(&info, "used_memory") else {
            return CheckOutcome::Fail("缺少 used_memory".into());
        };
        let max = parse_info_u64(&info, "maxmemory").unwrap_or(0);
        if max == 0 {
            return CheckOutcome::Ok;
        }
        let ratio = used as f64 / max as f64;
        if ratio >= ratio_limit {
            CheckOutcome::Fail(format!(
                "内存压力 used/max={ratio:.3} ≥ 阈值 {ratio_limit}（used={used} max={max}）"
            ))
        } else {
            CheckOutcome::Ok
        }
    }

    async fn check_cluster_slots(&self, ctx: &ValidationContext) -> CheckOutcome {
        let force_cluster = ctx.config.cluster_mode || self.config.cluster_mode;
        if !force_cluster {
            let probe = self
                .client
                .with_pool_conn(|mut conn: RedisBackend| async move {
                    map_redis_result::<String>(
                        redis::cmd("CLUSTER").arg("INFO").query_async(&mut conn).await,
                    )
                })
                .await;
            match probe {
                Err(_) => return CheckOutcome::Skip("非 Cluster 拓扑".into()),
                Ok(s) if !s.contains("cluster_state:") => {
                    return CheckOutcome::Skip("非 Cluster 拓扑".into());
                }
                Ok(_) => {}
            }
        }

        match self
            .client
            .with_pool_conn(|mut conn: RedisBackend| async move {
                map_redis_result::<redis::Value>(
                    redis::cmd("CLUSTER").arg("SLOTS").query_async(&mut conn).await,
                )
            })
            .await
        {
            Ok(slots) => {
                let covered = count_cluster_slots(&slots);
                if covered == 16_384 {
                    CheckOutcome::Ok
                } else {
                    CheckOutcome::Fail(format!("槽覆盖 {covered} != 16384"))
                }
            }
            Err(e) => CheckOutcome::Fail(format!("CLUSTER SLOTS: {:?}", e.kind())),
        }
    }
}

fn value_truthy(v: &redis::Value) -> bool {
    match v {
        redis::Value::Int(n) => *n != 0,
        redis::Value::Okay => true,
        redis::Value::BulkString(b) => b != b"0" && !b.is_empty(),
        redis::Value::SimpleString(s) => s != "0" && !s.is_empty(),
        redis::Value::Nil => false,
        _ => false,
    }
}

fn parse_info_u64(info: &str, key: &str) -> Option<u64> {
    for line in info.lines() {
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.strip_prefix(':').unwrap_or(rest);
        if let Ok(n) = rest.trim().parse::<u64>() {
            return Some(n);
        }
    }
    None
}

fn count_cluster_slots(v: &redis::Value) -> u32 {
    let redis::Value::Array(entries) = v else {
        return 0;
    };
    let mut total = 0u32;
    for e in entries {
        let redis::Value::Array(parts) = e else {
            continue;
        };
        if parts.len() < 2 {
            continue;
        }
        let start = match &parts[0] {
            redis::Value::Int(n) => *n,
            _ => continue,
        };
        let end = match &parts[1] {
            redis::Value::Int(n) => *n,
            _ => continue,
        };
        if end >= start {
            total = total.saturating_add((end - start + 1) as u32);
        }
    }
    total
}

fn redact_detail(s: &str) -> String {
    let mut out = s.to_owned();
    if let Some(i) = out.find("redis://") {
        if let Some(at) = out[i..].find('@') {
            out.replace_range(i..i + at + 1, "redis://***@");
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
    use crate::pool::RedisPool;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn catalog_matches_spec_count() {
        let cat = RedisValidator::static_catalog();
        assert_eq!(cat.len(), 11);
        assert!(cat.iter().all(|d| d.id.starts_with("redisx.")));
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Basic).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::ReadWrite).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Full).count(), 9);
    }

    #[tokio::test]
    async fn probe_basic_fails_and_short_circuits_full() {
        let pool = RedisPool::test_probe(Arc::new(AtomicUsize::new(0)));
        let v = RedisValidator::new(pool.client());
        let report = v.run(CheckLevel::Full).await;
        assert_eq!(report.module, "redisx");
        assert!(!report.passed);
        let ping = report.items.iter().find(|i| i.id == "redisx.basic.ping").expect("ping");
        assert_eq!(ping.status, CheckStatus::Failed);
        let rw = report.items.iter().find(|i| i.id == "redisx.rw.set_get_del").expect("rw");
        assert_eq!(rw.status, CheckStatus::Skipped);
        let full_skipped = report
            .items
            .iter()
            .filter(|i| i.id.starts_with("redisx.full.") && i.status == CheckStatus::Skipped)
            .count();
        assert_eq!(full_skipped, 9);
    }

    #[tokio::test]
    async fn skip_config_marks_item() {
        let pool = RedisPool::test_probe(Arc::new(AtomicUsize::new(0)));
        let mut cfg = RedisSelfCheckConfig::default();
        cfg.skip.insert("redisx.basic.ping".into());
        let v = RedisValidator::new(pool.client()).with_config(cfg);
        let report = v.run(CheckLevel::Basic).await;
        // skip 后 basic 无 Failed → passed
        let ping = report.items.iter().find(|i| i.id == "redisx.basic.ping").expect("ping");
        assert_eq!(ping.status, CheckStatus::Skipped);
        assert!(report.passed);
    }
}
