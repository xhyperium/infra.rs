//! postgres [`Validatable`] 实现与 §6.1 检查项执行。

use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use futures_util::FutureExt;
use kernel::ErrorKind;
use tokio::time::timeout;

use crate::config::{PostgresConfig, SslMode};
use crate::pool::PostgresPool;

use super::config::PostgresSelfCheckConfig;
use super::context::ValidationContext;
use super::types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};

/// 模块名（规范 3.1；crate 为 `postgresx`）。
pub const MODULE: &str = "postgres";

/// 检查返回：`Ok` 通过；`Skip` 拓扑/配置不适用；`Fail` 失败。
enum CheckOutcome {
    Ok,
    Skip(String),
    Fail(String),
}

/// postgres 验证器：绑定 [`PostgresPool`] + 自检配置。
///
/// `pg_config` 可选：用于 LISTEN/NOTIFY 专用连接（deadpool 不暴露 async message 流）。
#[derive(Clone, Debug)]
pub struct PostgresValidator {
    pool: PostgresPool,
    config: PostgresSelfCheckConfig,
    pg_config: Option<PostgresConfig>,
}

/// 验证器契约（规范 §5.2 子集，模块内聚）。
#[async_trait::async_trait]
pub trait Validatable: Send + Sync {
    fn module_name(&self) -> &'static str;
    fn catalog(&self) -> Vec<CheckDescriptor>;
    async fn validate(&self, ctx: &ValidationContext, level: CheckLevel) -> ValidationReport;
}

impl PostgresValidator {
    /// 仅持有池；`listen_notify` 在无 `pg_config` 时 Failed（诚实，非假绿）。
    #[must_use]
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool, config: PostgresSelfCheckConfig::default(), pg_config: None }
    }

    /// 从配置建池并保留连接参数（推荐 live / Full）。
    pub async fn connect(pg: PostgresConfig) -> kernel::XResult<Self> {
        let pool = PostgresPool::connect(&pg).await?;
        Ok(Self { pool, config: PostgresSelfCheckConfig::default(), pg_config: Some(pg) })
    }

    /// 从环境变量建池（`DATABASE_URL` / `FOUNDATIONX_POSTGRESX_*`）。
    pub async fn connect_from_env() -> kernel::XResult<Self> {
        let pg = PostgresConfig::from_env()?;
        Self::connect(pg).await
    }

    #[must_use]
    pub fn with_config(mut self, config: PostgresSelfCheckConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn with_pg_config(mut self, pg: PostgresConfig) -> Self {
        self.pg_config = Some(pg);
        self
    }

    #[must_use]
    pub fn config(&self) -> &PostgresSelfCheckConfig {
        &self.config
    }

    #[must_use]
    pub fn pool(&self) -> &PostgresPool {
        &self.pool
    }

    /// 使用默认上下文执行验证（生成 run_id/token）。
    pub async fn run(&self, level: CheckLevel) -> ValidationReport {
        let ctx = ValidationContext::new(self.config.clone());
        self.validate(&ctx, level).await
    }

    /// 静态 catalog（§6.1，共 11 项）。
    #[must_use]
    pub fn static_catalog() -> Vec<CheckDescriptor> {
        vec![
            desc("postgres.basic.ping", CheckLevel::Basic, Some(50), "SELECT 1", false),
            desc("postgres.basic.version", CheckLevel::Basic, None, "版本 ≥ 配置最低版本", false),
            desc(
                "postgres.rw.crud_roundtrip",
                CheckLevel::ReadWrite,
                Some(200),
                "UNLOGGED 表 CRUD + token 比对",
                false,
            ),
            desc("postgres.rw.tx_commit", CheckLevel::ReadWrite, None, "提交后可见", false),
            desc("postgres.rw.tx_rollback", CheckLevel::ReadWrite, None, "回滚后不可见", false),
            desc(
                "postgres.full.batch_insert_1k",
                CheckLevel::Full,
                Some(500),
                "UNNEST 批量写入 1000 行",
                true,
            ),
            desc(
                "postgres.full.jsonb_roundtrip",
                CheckLevel::Full,
                None,
                "JSONB 写入 + 路径查询",
                true,
            ),
            desc(
                "postgres.full.listen_notify",
                CheckLevel::Full,
                Some(2_000),
                "LISTEN/NOTIFY 闭环",
                false,
            ),
            desc(
                "postgres.full.pool_saturation",
                CheckLevel::Full,
                None,
                "池满时正确超时不死锁",
                false,
            ),
            desc("postgres.full.pool_recovery", CheckLevel::Full, None, "归还后池恢复", false),
            desc(
                "postgres.full.replication_lag",
                CheckLevel::Full,
                Some(1_000),
                "主从延迟 < 阈值（可选）",
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
impl Validatable for PostgresValidator {
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

impl PostgresValidator {
    async fn run_checks(
        &self,
        ctx: &ValidationContext,
        level: CheckLevel,
        items: &mut Vec<CheckItem>,
    ) {
        if level.includes(CheckLevel::Basic) {
            items.push(
                self.exec_check(ctx, "postgres.basic.ping", Some(50), self.check_ping(ctx)).await,
            );
            items.push(
                self.exec_check(ctx, "postgres.basic.version", None, self.check_version(ctx)).await,
            );
        }

        // C-13: basic.* 全部失败 → rw/full 短路 Skipped
        // 部分 basic 失败仍继续（仅「全部失败」短路）
        let basic_items: Vec<_> =
            items.iter().filter(|i| i.id.starts_with("postgres.basic.")).collect();
        let basic_all_failed =
            !basic_items.is_empty() && basic_items.iter().all(|i| i.status == CheckStatus::Failed);

        let rw_ids = [
            ("postgres.rw.crud_roundtrip", Some(200u64)),
            ("postgres.rw.tx_commit", None),
            ("postgres.rw.tx_rollback", None),
        ];

        if level.includes(CheckLevel::ReadWrite) {
            if basic_all_failed {
                for (id, _) in rw_ids {
                    items.push(CheckItem::skipped(id, "短路：basic 全部失败", now_rfc3339()));
                }
            } else {
                items.push(
                    self.exec_check(
                        ctx,
                        "postgres.rw.crud_roundtrip",
                        Some(200),
                        self.check_crud_roundtrip(ctx),
                    )
                    .await,
                );
                items.push(
                    self.exec_check(ctx, "postgres.rw.tx_commit", None, self.check_tx_commit(ctx))
                        .await,
                );
                items.push(
                    self.exec_check(
                        ctx,
                        "postgres.rw.tx_rollback",
                        None,
                        self.check_tx_rollback(ctx),
                    )
                    .await,
                );
            }
        }

        if !level.includes(CheckLevel::Full) {
            return;
        }

        let full_specs = [
            ("postgres.full.batch_insert_1k", Some(500u64)),
            ("postgres.full.jsonb_roundtrip", None),
            ("postgres.full.listen_notify", Some(2_000)),
            ("postgres.full.pool_saturation", None),
            ("postgres.full.pool_recovery", None),
            ("postgres.full.replication_lag", Some(1_000)),
        ];

        if basic_all_failed {
            for (id, _) in full_specs {
                items.push(CheckItem::skipped(id, "短路：basic 全部失败", now_rfc3339()));
            }
            return;
        }

        items.push(
            self.exec_check(
                ctx,
                "postgres.full.batch_insert_1k",
                Some(500),
                self.check_batch_insert_1k(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "postgres.full.jsonb_roundtrip",
                None,
                self.check_jsonb_roundtrip(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "postgres.full.listen_notify",
                Some(2_000),
                self.check_listen_notify(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "postgres.full.pool_saturation",
                None,
                self.check_pool_saturation(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "postgres.full.pool_recovery",
                None,
                self.check_pool_recovery(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "postgres.full.replication_lag",
                Some(1_000),
                self.check_replication_lag(ctx),
            )
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
        // 单检查项 panic 不得拖垮进程
        let outcome = match AssertUnwindSafe(outcome_fut).catch_unwind().await {
            Ok(o) => o,
            Err(_) => CheckOutcome::Fail("检查项 panic（已捕获）".into()),
        };
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
        match self.pool.health().await {
            Ok(()) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(format!("ping 失败: {:?}", e.kind())),
        }
    }

    async fn check_version(&self, ctx: &ValidationContext) -> CheckOutcome {
        let min = ctx.config.min_version.max(self.config.min_version);
        let min_num = i32::try_from(min.saturating_mul(10_000)).unwrap_or(i32::MAX);
        match self.pool.query_one("SELECT current_setting('server_version_num')::int4", &[]).await {
            Ok(row) => match row.try_get::<_, i32>(0) {
                Ok(num) if num >= min_num => CheckOutcome::Ok,
                Ok(num) => CheckOutcome::Fail(format!(
                    "server_version_num={num} < 最低要求 {min_num} (v{min})"
                )),
                Err(e) => CheckOutcome::Fail(format!("解析版本: {e}")),
            },
            Err(e) => CheckOutcome::Fail(format!("读版本: {:?}", e.kind())),
        }
    }

    async fn check_crud_roundtrip(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.table("crud");
        let token = ctx.token.clone();
        let mut conn = match self.pool.acquire().await {
            Ok(c) => c,
            Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
        };

        let create = format!(
            "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (k text PRIMARY KEY, v text NOT NULL)"
        );
        if let Err(e) = conn.batch_execute(&create).await {
            return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
        }

        let insert_sql = format!("INSERT INTO {table} (k, v) VALUES ($1, $2)");
        if let Err(e) = conn.execute(&insert_sql, &[&token, &"ok"]).await {
            let _ = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await;
            return CheckOutcome::Fail(format!("insert: {:?}", e.kind()));
        }

        let select_sql = format!("SELECT v FROM {table} WHERE k = $1");
        let got = match conn.query_one(&select_sql, &[&token]).await {
            Ok(row) => row.try_get::<_, String>(0).unwrap_or_default(),
            Err(e) => {
                let _ = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await;
                return CheckOutcome::Fail(format!("select: {:?}", e.kind()));
            }
        };
        if got != "ok" {
            let _ = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await;
            return CheckOutcome::Fail("crud 读回不一致".into());
        }

        let update_sql = format!("UPDATE {table} SET v = $1 WHERE k = $2");
        if let Err(e) = conn.execute(&update_sql, &[&"upd", &token]).await {
            let _ = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await;
            return CheckOutcome::Fail(format!("update: {:?}", e.kind()));
        }

        let del_sql = format!("DELETE FROM {table} WHERE k = $1");
        if let Err(e) = conn.execute(&del_sql, &[&token]).await {
            let _ = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await;
            return CheckOutcome::Fail(format!("delete: {:?}", e.kind()));
        }

        if let Err(e) = conn.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await {
            return CheckOutcome::Fail(format!("drop: {:?}", e.kind()));
        }
        CheckOutcome::Ok
    }

    async fn check_tx_commit(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.table("txc");
        let token = ctx.token.clone();
        {
            let mut c = match self.pool.acquire().await {
                Ok(c) => c,
                Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
            };
            if let Err(e) = c
                .batch_execute(&format!(
                    "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (k text PRIMARY KEY, v text)"
                ))
                .await
            {
                return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
            }
        }

        let insert_sql = format!("INSERT INTO {table} (k, v) VALUES ($1, $2)");
        let r = self
            .pool
            .with_transaction(async |tx| {
                tx.execute(&insert_sql, &[&token, &"committed"]).await?;
                Ok::<_, kernel::XError>(())
            })
            .await;
        if let Err(e) = r {
            let _ = self.drop_table(&table).await;
            return CheckOutcome::Fail(format!("tx commit path: {:?}", e.kind()));
        }

        let select_sql = format!("SELECT v FROM {table} WHERE k = $1");
        let outcome = match self.pool.query_one(&select_sql, &[&token]).await {
            Ok(row) => match row.try_get::<_, String>(0) {
                Ok(v) if v == "committed" => CheckOutcome::Ok,
                Ok(_) => CheckOutcome::Fail("提交后值不正确".into()),
                Err(e) => CheckOutcome::Fail(format!("try_get: {e}")),
            },
            Err(e) => CheckOutcome::Fail(format!("提交后不可见: {:?}", e.kind())),
        };
        let _ = self.drop_table(&table).await;
        outcome
    }

    async fn check_tx_rollback(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.table("txr");
        let token = ctx.token.clone();
        {
            let mut c = match self.pool.acquire().await {
                Ok(c) => c,
                Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
            };
            if let Err(e) = c
                .batch_execute(&format!(
                    "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (k text PRIMARY KEY, v text)"
                ))
                .await
            {
                return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
            }
        }

        let insert_sql = format!("INSERT INTO {table} (k, v) VALUES ($1, $2)");
        let r = self
            .pool
            .with_transaction(async |tx| {
                tx.execute(&insert_sql, &[&token, &"should_not_exist"]).await?;
                Err::<(), _>(kernel::XError::invalid("selfcheck 故意回滚"))
            })
            .await;
        if r.is_ok() {
            let _ = self.drop_table(&table).await;
            return CheckOutcome::Fail("期望业务错误触发 rollback".into());
        }

        let select_sql = format!("SELECT v FROM {table} WHERE k = $1");
        let outcome = match self.pool.query_opt(&select_sql, &[&token]).await {
            Ok(None) => CheckOutcome::Ok,
            Ok(Some(_)) => CheckOutcome::Fail("回滚后仍可见".into()),
            Err(e) => CheckOutcome::Fail(format!("query_opt: {:?}", e.kind())),
        };
        let _ = self.drop_table(&table).await;
        outcome
    }

    async fn check_batch_insert_1k(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.table("batch");
        {
            let mut c = match self.pool.acquire().await {
                Ok(c) => c,
                Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
            };
            if let Err(e) = c
                .batch_execute(&format!(
                    "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (id int PRIMARY KEY, v text)"
                ))
                .await
            {
                return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
            }
        }

        // generate_series 等价于 UNNEST 批量 1000 行（避免额外 array ToSql feature）
        let insert_r = self
            .pool
            .execute(
                &format!(
                    "INSERT INTO {table} (id, v) SELECT g, 'r' || g::text FROM generate_series(0, 999) AS g"
                ),
                &[],
            )
            .await;

        if let Err(e) = insert_r {
            let _ = self.drop_table(&table).await;
            return CheckOutcome::Fail(format!("batch insert: {:?}", e.kind()));
        }

        let count_sql = format!("SELECT count(*)::int4 FROM {table}");
        let outcome = match self.pool.query_one(&count_sql, &[]).await {
            Ok(row) => match row.try_get::<_, i32>(0) {
                Ok(1000) => CheckOutcome::Ok,
                Ok(n) => CheckOutcome::Fail(format!("期望 1000 行，实际 {n}")),
                Err(e) => CheckOutcome::Fail(format!("count: {e}")),
            },
            Err(e) => CheckOutcome::Fail(format!("count query: {:?}", e.kind())),
        };
        let _ = self.drop_table(&table).await;
        outcome
    }

    async fn check_jsonb_roundtrip(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.table("jsonb");
        let token = ctx.token.clone();
        {
            let mut c = match self.pool.acquire().await {
                Ok(c) => c,
                Err(e) => return CheckOutcome::Fail(format!("acquire: {:?}", e.kind())),
            };
            if let Err(e) = c
                .batch_execute(&format!(
                    "CREATE UNLOGGED TABLE IF NOT EXISTS {table} (k text PRIMARY KEY, payload jsonb)"
                ))
                .await
            {
                return CheckOutcome::Fail(format!("create: {:?}", e.kind()));
            }
        }

        let payload = format!(r#"{{"token":"{token}","n":42}}"#);
        // 绑定 text 再 cast jsonb，避免缺少 with-serde-json feature 时的类型绑定问题
        let insert_sql =
            format!("INSERT INTO {table} (k, payload) VALUES ($1, CAST($2 AS text)::jsonb)");
        if let Err(e) = self.pool.execute(&insert_sql, &[&token, &payload]).await {
            let _ = self.drop_table(&table).await;
            return CheckOutcome::Fail(format!(
                "jsonb insert: {:?} — {}",
                e.kind(),
                redact_detail(e.context())
            ));
        }

        let select_sql =
            format!("SELECT payload->>'token', (payload->>'n')::int4 FROM {table} WHERE k = $1");
        let outcome = match self.pool.query_one(&select_sql, &[&token]).await {
            Ok(row) => {
                let t: String = row.try_get(0).unwrap_or_default();
                let n: i32 = row.try_get(1).unwrap_or(-1);
                if t == token && n == 42 {
                    CheckOutcome::Ok
                } else {
                    CheckOutcome::Fail(format!("jsonb 路径读回不一致 token={t} n={n}"))
                }
            }
            Err(e) => CheckOutcome::Fail(format!("jsonb select: {:?}", e.kind())),
        };
        let _ = self.drop_table(&table).await;
        outcome
    }

    async fn check_listen_notify(&self, ctx: &ValidationContext) -> CheckOutcome {
        let Some(pg) = self.pg_config.as_ref() else {
            return CheckOutcome::Fail(
                "listen_notify 需要 PostgresConfig（请用 PostgresValidator::connect / with_pg_config）"
                    .into(),
            );
        };

        let channel = ctx.channel();
        let payload = format!("p-{}", ctx.token);
        let wait = Duration::from_millis(
            ctx.config.notify_wait_ms.max(self.config.notify_wait_ms).max(500),
        );

        match run_listen_notify_loop(pg, &channel, &payload, wait, self.pool.clone()).await {
            Ok(()) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(e),
        }
    }

    async fn check_pool_saturation(&self, ctx: &ValidationContext) -> CheckOutcome {
        let probe = Duration::from_millis(
            ctx.config.pool_acquire_probe_ms.max(self.config.pool_acquire_probe_ms).max(50),
        );
        let max = self.pool.stats().max_size.max(1);
        let mut held = Vec::with_capacity(max);
        for i in 0..max {
            match self.pool.acquire().await {
                Ok(c) => held.push(c),
                Err(e) => {
                    drop(held);
                    return CheckOutcome::Fail(format!("持有第 {i} 连接失败: {:?}", e.kind()));
                }
            }
        }
        // 池满后短超时 acquire 应失败（证明不死锁且有界等待）
        let extra = self.pool.acquire_with(probe).await;
        drop(held);
        match extra {
            Ok(_) => CheckOutcome::Fail("池满时仍能 acquire，饱和语义未生效".into()),
            Err(_) => CheckOutcome::Ok,
        }
    }

    async fn check_pool_recovery(&self, ctx: &ValidationContext) -> CheckOutcome {
        let _ = ctx;
        let probe = Duration::from_millis(2_000);
        // 归还后应能再借
        match self.pool.acquire_with(probe).await {
            Ok(_c) => {
                // drop 归还
                CheckOutcome::Ok
            }
            Err(e) => CheckOutcome::Fail(format!("归还后 acquire 失败: {:?}", e.kind())),
        }
    }

    async fn check_replication_lag(&self, ctx: &ValidationContext) -> CheckOutcome {
        let enabled = ctx.config.replica_check || self.config.replica_check;
        if !enabled {
            return CheckOutcome::Skip("replica_check=false：未启用主从延迟检查".into());
        }
        let max_lag = ctx.config.max_replication_lag_ms.max(self.config.max_replication_lag_ms)
            as f64
            / 1000.0;

        // 读 pg_stat_replication；无副本 → Skipped
        match self
            .pool
            .query(
                "SELECT COALESCE(EXTRACT(EPOCH FROM (now() - replay_lag))::float8, EXTRACT(EPOCH FROM replay_lag)::float8, 0) AS lag_s FROM pg_stat_replication",
                &[],
            )
            .await
        {
            Ok(rows) if rows.is_empty() => {
                CheckOutcome::Skip("无活跃副本（pg_stat_replication 为空）".into())
            }
            Ok(rows) => {
                let mut worst = 0.0_f64;
                for r in &rows {
                    let lag: f64 = r.try_get(0).unwrap_or(0.0);
                    if lag > worst {
                        worst = lag;
                    }
                }
                if worst <= max_lag {
                    CheckOutcome::Ok
                } else {
                    CheckOutcome::Fail(format!(
                        "副本延迟 {worst:.3}s > 阈值 {max_lag:.3}s"
                    ))
                }
            }
            Err(e) => {
                // 权限不足等：诚实 Failed 或 Skip
                if e.kind() == ErrorKind::Missing {
                    CheckOutcome::Skip(format!("无法读 pg_stat_replication: {:?}", e.kind()))
                } else {
                    CheckOutcome::Fail(format!("replication 查询: {:?}", e.kind()))
                }
            }
        }
    }

    async fn drop_table(&self, table: &str) -> Result<(), kernel::XError> {
        let mut c = self.pool.acquire().await?;
        c.batch_execute(&format!("DROP TABLE IF EXISTS {table}")).await
    }
}

// --- helpers ---------------------------------------------------------------

async fn run_listen_notify_loop(
    pg: &PostgresConfig,
    channel: &str,
    payload: &str,
    wait: Duration,
    pool: PostgresPool,
) -> Result<(), String> {
    use crate::tls::MakeRustlsConnect;
    use tokio_postgres::NoTls;

    let mut cfg = tokio_postgres::Config::new();
    cfg.host(&pg.host);
    cfg.port(pg.port);
    cfg.dbname(&pg.database);
    cfg.user(&pg.user);
    if !pg.password.is_empty() {
        cfg.password(&pg.password);
    }
    if let Some(t) = pg.connect_timeout {
        cfg.connect_timeout(t);
    }

    let ch = channel.to_owned();
    let pl = payload.to_owned();
    let expected = format!("{channel}|{payload}");

    async fn drive_and_listen<T>(
        client: tokio_postgres::Client,
        mut connection: tokio_postgres::Connection<tokio_postgres::Socket, T>,
        channel: String,
        payload: String,
        expected: String,
        wait: Duration,
        pool: PostgresPool,
    ) -> Result<(), String>
    where
        T: tokio_postgres::tls::TlsStream + Unpin + Send + 'static,
    {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let driver = tokio::spawn(async move {
            loop {
                match futures_util::future::poll_fn(|cx| connection.poll_message(cx)).await {
                    Some(Ok(tokio_postgres::AsyncMessage::Notification(n))) => {
                        let _ = tx.send(format!("{}|{}", n.channel(), n.payload()));
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break,
                }
            }
        });

        client
            .batch_execute(&format!("LISTEN {channel}"))
            .await
            .map_err(|e| format!("LISTEN: {e}"))?;

        tokio::time::sleep(Duration::from_millis(30)).await;
        {
            let mut n =
                pool.acquire().await.map_err(|e| format!("NOTIFY acquire: {:?}", e.kind()))?;
            n.execute("SELECT pg_notify($1, $2)", &[&channel, &payload])
                .await
                .map_err(|e| format!("NOTIFY: {:?}", e.kind()))?;
        }

        let got = timeout(wait, rx.recv())
            .await
            .map_err(|_| "LISTEN/NOTIFY 超时".to_string())?
            .ok_or_else(|| "通知通道关闭".to_string())?;
        if got != expected {
            driver.abort();
            return Err(format!("通知不匹配: {got}"));
        }

        let _ = client.batch_execute(&format!("UNLISTEN {channel}")).await;
        drop(client);
        driver.abort();
        Ok(())
    }

    match pg.sslmode {
        SslMode::Disable => {
            let (client, connection) =
                cfg.connect(NoTls).await.map_err(|e| format!("connect: {e}"))?;
            drive_and_listen(client, connection, ch, pl, expected, wait, pool).await
        }
        SslMode::Prefer | SslMode::Require => {
            let tls = MakeRustlsConnect::with_options(
                pg.tls_ca_file.as_deref(),
                pg.tls_client_cert.as_deref(),
                pg.tls_client_key.as_deref(),
            )
            .map_err(|e| format!("tls: {:?}", e.kind()))?;
            let (client, connection) =
                cfg.connect(tls).await.map_err(|e| format!("connect: {e}"))?;
            drive_and_listen(client, connection, ch, pl, expected, wait, pool).await
        }
    }
}

fn redact_detail(s: &str) -> String {
    let mut out = s.to_owned();
    for scheme in ["postgres://", "postgresql://"] {
        if let Some(i) = out.find(scheme) {
            if let Some(at) = out[i..].find('@') {
                out.replace_range(i..i + at + 1, &format!("{scheme}***@"));
            }
        }
    }
    // 常见 password= 形态
    if let Some(i) = out.find("password=") {
        let rest = &out[i + "password=".len()..];
        let end = rest.find([' ', '&', ';', '"']).unwrap_or(rest.len());
        out.replace_range(i..i + "password=".len() + end, "password=***");
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
    use std::time::Duration;

    #[test]
    fn catalog_matches_spec_section_6_1() {
        let cat = PostgresValidator::static_catalog();
        assert_eq!(cat.len(), 11);
        assert!(cat.iter().all(|d| d.id.starts_with("postgres.")));
        let ids: Vec<_> = cat.iter().map(|d| d.id.as_str()).collect();
        for expected in [
            "postgres.basic.ping",
            "postgres.basic.version",
            "postgres.rw.crud_roundtrip",
            "postgres.rw.tx_commit",
            "postgres.rw.tx_rollback",
            "postgres.full.batch_insert_1k",
            "postgres.full.jsonb_roundtrip",
            "postgres.full.listen_notify",
            "postgres.full.pool_saturation",
            "postgres.full.pool_recovery",
            "postgres.full.replication_lag",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Basic).count(), 2);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::ReadWrite).count(), 3);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Full).count(), 6);
    }

    #[test]
    fn redact_hides_password_and_url() {
        let s = redact_detail("fail postgres://user:secret@host/db password=hunter2 more");
        assert!(!s.contains("secret"));
        assert!(!s.contains("hunter2"));
        assert!(s.contains("***"));
    }

    async fn unreachable_validator() -> PostgresValidator {
        let cfg = PostgresConfig::builder()
            .host("127.0.0.1")
            .port(1)
            .database("db")
            .user("u")
            .password("p")
            .sslmode(SslMode::Disable)
            .max_pool_size(1)
            .acquire_timeout(Duration::from_millis(80))
            .operation_timeout(Duration::from_millis(80))
            .build()
            .expect("cfg");
        let pool = PostgresPool::connect_lazy(&cfg).await.expect("lazy pool");
        PostgresValidator::new(pool)
    }

    #[tokio::test]
    async fn basic_fail_short_circuits_rw_and_full() {
        let v = unreachable_validator().await;
        let report = v.run(CheckLevel::Full).await;
        assert_eq!(report.module, "postgres");
        assert!(!report.passed);
        assert_eq!(report.items.len(), 11);

        let basic_failed = report
            .items
            .iter()
            .filter(|i| i.id.starts_with("postgres.basic.") && i.status == CheckStatus::Failed)
            .count();
        assert_eq!(basic_failed, 2);

        for i in &report.items {
            if i.id.starts_with("postgres.rw.") || i.id.starts_with("postgres.full.") {
                assert_eq!(i.status, CheckStatus::Skipped, "{} should be skipped", i.id);
                assert!(i.detail.as_ref().is_some_and(|d| d.contains("短路")), "{} detail", i.id);
            }
        }
    }

    #[tokio::test]
    async fn skip_config_marks_item_without_fail() {
        let mut cfg = PostgresSelfCheckConfig::default();
        cfg.skip.insert("postgres.basic.ping".into());
        cfg.skip.insert("postgres.basic.version".into());
        let v = unreachable_validator().await.with_config(cfg);
        let report = v.run(CheckLevel::Basic).await;
        assert!(report.passed, "skip 后无 Failed → passed");
        assert_eq!(report.items.len(), 2);
        assert!(report.items.iter().all(|i| i.status == CheckStatus::Skipped));
    }

    #[tokio::test]
    async fn table_token_pattern_on_context() {
        let ctx = ValidationContext::with_ids(PostgresSelfCheckConfig::default(), "run-x", "tok99");
        assert_eq!(ctx.table(""), "_self_check_tok99");
        assert!(ctx.table("crud").starts_with("_self_check_tok99"));
    }
}
