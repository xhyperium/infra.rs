//! taos [`Validatable`] 实现与 §6.7 检查项执行。

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use canonical::Tick;
use contracts::TimeSeriesStore;
use decimalx::{Decimal, Price};
use tokio::time::timeout;

use crate::client::TaosPool;
use crate::config::TsPrecision;

use super::config::TaosSelfCheckConfig;
use super::context::ValidationContext;
use super::types::{
    CheckDescriptor, CheckItem, CheckLevel, CheckStatus, ValidationReport, now_rfc3339,
};

/// 模块名（规范 3.1；crate 为 `taosx`）。
pub const MODULE: &str = "taos";

/// 检查返回：`Ok` 通过；`Skip` 拓扑/能力不适用；`Fail` 失败。
enum CheckOutcome {
    Ok,
    Skip(String),
    Fail(String),
}

/// taos 验证器：绑定 [`TaosPool`] + 自检配置。
#[derive(Clone)]
pub struct TaosValidator {
    pool: TaosPool,
    config: TaosSelfCheckConfig,
}

impl std::fmt::Debug for TaosValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaosValidator")
            .field("config", &self.config)
            .field("pool", &"TaosPool{..}")
            .finish()
    }
}

/// 验证器契约（规范 §5.2 子集，模块内聚）。
#[async_trait::async_trait]
pub trait Validatable: Send + Sync {
    fn module_name(&self) -> &'static str;
    fn catalog(&self) -> Vec<CheckDescriptor>;
    async fn validate(&self, ctx: &ValidationContext, level: CheckLevel) -> ValidationReport;
}

impl TaosValidator {
    #[must_use]
    pub fn new(pool: TaosPool) -> Self {
        Self { pool, config: TaosSelfCheckConfig::default() }
    }

    #[must_use]
    pub fn with_config(mut self, config: TaosSelfCheckConfig) -> Self {
        self.config = config;
        self
    }

    #[must_use]
    pub fn config(&self) -> &TaosSelfCheckConfig {
        &self.config
    }

    #[must_use]
    pub fn pool(&self) -> &TaosPool {
        &self.pool
    }

    /// 使用默认上下文执行验证（生成 run_id/token）。
    pub async fn run(&self, level: CheckLevel) -> ValidationReport {
        let ctx = ValidationContext::new(self.config.clone());
        self.validate(&ctx, level).await
    }

    /// 静态 catalog（§6.7，共 9 项）。
    #[must_use]
    pub fn static_catalog() -> Vec<CheckDescriptor> {
        vec![
            desc("taos.basic.ping", CheckLevel::Basic, Some(100), "SERVER_VERSION()", false),
            desc(
                "taos.rw.insert_query",
                CheckLevel::ReadWrite,
                Some(1_000),
                "子表写入查询闭环",
                false,
            ),
            desc(
                "taos.full.stable_ddl",
                CheckLevel::Full,
                Some(1_000),
                "超级表（含 TAGS）建删",
                true,
            ),
            desc(
                "taos.full.auto_subtable",
                CheckLevel::Full,
                Some(1_000),
                "USING 自动建子表 + 批量行写入",
                true,
            ),
            desc(
                "taos.full.tag_filter",
                CheckLevel::Full,
                Some(500),
                "TAG 过滤计数精确比对",
                false,
            ),
            desc(
                "taos.full.interval_window",
                CheckLevel::Full,
                Some(1_000),
                "INTERVAL 窗口聚合",
                false,
            ),
            desc("taos.full.last_row", CheckLevel::Full, None, "LAST_ROW 值正确", false),
            desc("taos.full.tmq_subscribe", CheckLevel::Full, Some(5_000), "TMQ 订阅闭环", true),
            desc("taos.full.db_config", CheckLevel::Full, None, "precision 与预期一致", false),
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
impl Validatable for TaosValidator {
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

impl TaosValidator {
    async fn run_checks(
        &self,
        ctx: &ValidationContext,
        level: CheckLevel,
        items: &mut Vec<CheckItem>,
    ) {
        if level.includes(CheckLevel::Basic) {
            items.push(
                self.exec_check(ctx, "taos.basic.ping", Some(100), self.check_ping(ctx)).await,
            );
        }

        let basic_failed = items
            .iter()
            .any(|i| i.id.starts_with("taos.basic.") && i.status == CheckStatus::Failed);

        if level.includes(CheckLevel::ReadWrite) {
            if basic_failed {
                items.push(CheckItem::skipped(
                    "taos.rw.insert_query",
                    "短路：basic 失败",
                    now_rfc3339(),
                ));
            } else {
                items.push(
                    self.exec_check(
                        ctx,
                        "taos.rw.insert_query",
                        Some(1_000),
                        self.check_insert_query(ctx),
                    )
                    .await,
                );
            }
        }

        if !level.includes(CheckLevel::Full) {
            return;
        }

        let full_ids = [
            "taos.full.stable_ddl",
            "taos.full.auto_subtable",
            "taos.full.tag_filter",
            "taos.full.interval_window",
            "taos.full.last_row",
            "taos.full.tmq_subscribe",
            "taos.full.db_config",
        ];

        if basic_failed {
            for id in full_ids {
                items.push(CheckItem::skipped(id, "短路：basic 失败", now_rfc3339()));
            }
            return;
        }

        items.push(
            self.exec_check(ctx, "taos.full.stable_ddl", Some(1_000), self.check_stable_ddl(ctx))
                .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "taos.full.auto_subtable",
                Some(1_000),
                self.check_auto_subtable(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "taos.full.tag_filter", Some(500), self.check_tag_filter(ctx))
                .await,
        );
        items.push(
            self.exec_check(
                ctx,
                "taos.full.interval_window",
                Some(1_000),
                self.check_interval_window(ctx),
            )
            .await,
        );
        items
            .push(self.exec_check(ctx, "taos.full.last_row", None, self.check_last_row(ctx)).await);
        items.push(
            self.exec_check(
                ctx,
                "taos.full.tmq_subscribe",
                Some(5_000),
                self.check_tmq_subscribe(ctx),
            )
            .await,
        );
        items.push(
            self.exec_check(ctx, "taos.full.db_config", None, self.check_db_config(ctx)).await,
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
        if !self.pool.liveness() {
            return CheckOutcome::Fail("池已关闭".into());
        }
        match self.pool.ping().await {
            Ok(()) => CheckOutcome::Ok,
            Err(e) => CheckOutcome::Fail(format!("ping 失败: {:?}", e.kind())),
        }
    }

    async fn check_insert_query(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("rw");
        let now_ns = now_ns();
        let prec = self.pool.precision();
        // 对齐精度边界，避免 ms 库丢纳秒导致查询 miss
        let ts = prec.to_nanos(prec.from_nanos(now_ns));
        let tick = sample_tick("SC_RW", ts, 10_050, 10_060);
        if let Err(e) = self.pool.write_series(&table, vec![tick.clone()]).await {
            let _ = self.drop_stable(&table).await;
            return CheckOutcome::Fail(format!("write: {:?}", e.kind()));
        }
        let rows = match self.pool.query_series(&table, ts, ts).await {
            Ok(r) => r,
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("query: {:?}", e.kind()));
            }
        };
        let _ = self.drop_stable(&table).await;
        if rows.is_empty() {
            return CheckOutcome::Fail("query 空集".into());
        }
        let got = &rows[0];
        if got.symbol != "SC_RW" {
            return CheckOutcome::Fail(format!("symbol 不一致: {}", got.symbol));
        }
        if got.ts != ts {
            return CheckOutcome::Fail(format!("ts 不一致: {} != {ts}", got.ts));
        }
        CheckOutcome::Ok
    }

    async fn check_stable_ddl(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("ddl");
        let create = format!(
            "CREATE STABLE IF NOT EXISTS `{table}` (\
               ts TIMESTAMP, v DOUBLE\
             ) TAGS (t NCHAR(32))"
        );
        match self.pool.exec_sql(&create).await {
            Ok(r) if r.code == 0 => {}
            Ok(r) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("CREATE STABLE code={}", r.code));
            }
            Err(e) => return CheckOutcome::Fail(format!("CREATE: {:?}", e.kind())),
        }
        // DESCRIBE 确认存在
        match self.pool.exec_sql(&format!("DESCRIBE `{table}`")).await {
            Ok(r) if r.code == 0 && !r.rows.is_empty() => {}
            Ok(r) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!(
                    "DESCRIBE code={} rows={}",
                    r.code,
                    r.rows.len()
                ));
            }
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("DESCRIBE: {:?}", e.kind()));
            }
        }
        match self.drop_stable(&table).await {
            Ok(()) => CheckOutcome::Ok,
            Err(msg) => CheckOutcome::Fail(msg),
        }
    }

    async fn check_auto_subtable(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("auto");
        let n = self.config.auto_subtable_rows.clamp(1, 500);
        let now_ns = now_ns();
        let prec = self.pool.precision();
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let raw = now_ns.saturating_add((i as i64).saturating_mul(1_000_000));
            let ts = prec.to_nanos(prec.from_nanos(raw));
            // 3 个 symbol → 自动 3 个子表
            let sym = match i % 3 {
                0 => "A",
                1 => "B",
                _ => "C",
            };
            points.push(sample_tick(sym, ts, 100 + i as i128, 200 + i as i128));
        }
        match self.pool.write_batch(&table, &points).await {
            Ok(()) => {}
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("batch write: {:?}", e.kind()));
            }
        }
        let start = points.first().map(|t| t.ts).unwrap_or(now_ns);
        let end = points.last().map(|t| t.ts).unwrap_or(now_ns);
        let rows = match self.pool.query_series(&table, start, end).await {
            Ok(r) => r,
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("query batch: {:?}", e.kind()));
            }
        };
        let _ = self.drop_stable(&table).await;
        if rows.len() < n {
            return CheckOutcome::Fail(format!("期望 ≥{n} 行，实际 {}", rows.len()));
        }
        CheckOutcome::Ok
    }

    async fn check_tag_filter(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("tag");
        let now_ns = now_ns();
        let prec = self.pool.precision();
        let t0 = prec.to_nanos(prec.from_nanos(now_ns));
        let t1 = prec.to_nanos(prec.from_nanos(now_ns.saturating_add(2_000_000)));
        let points = vec![
            sample_tick("TAG_A", t0, 1, 2),
            sample_tick("TAG_A", t1, 3, 4),
            sample_tick("TAG_B", t0, 5, 6),
        ];
        if let Err(e) = self.pool.write_batch(&table, &points).await {
            let _ = self.drop_stable(&table).await;
            return CheckOutcome::Fail(format!("write: {:?}", e.kind()));
        }
        let sql = format!("SELECT COUNT(*) FROM `{table}` WHERE symbol = 'TAG_A'");
        let count = match self.pool.exec_sql(&sql).await {
            Ok(r) if r.code == 0 => parse_count_cell(r.rows.first()),
            Ok(r) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("COUNT code={}", r.code));
            }
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("COUNT: {:?}", e.kind()));
            }
        };
        let _ = self.drop_stable(&table).await;
        match count {
            Some(2) => CheckOutcome::Ok,
            Some(n) => CheckOutcome::Fail(format!("TAG_A 计数期望 2 实际 {n}")),
            None => CheckOutcome::Fail("COUNT 无结果".into()),
        }
    }

    async fn check_interval_window(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("ivl");
        let now_ns = now_ns();
        let prec = self.pool.precision();
        // 两个间隔 ≥1s 的点，便于 1s INTERVAL
        let t0 = prec.to_nanos(prec.from_nanos(now_ns));
        let t1 = prec.to_nanos(prec.from_nanos(now_ns.saturating_add(1_500_000_000)));
        let points = vec![sample_tick("IVL", t0, 10, 11), sample_tick("IVL", t1, 20, 21)];
        if let Err(e) = self.pool.write_batch(&table, &points).await {
            let _ = self.drop_stable(&table).await;
            return CheckOutcome::Fail(format!("write: {:?}", e.kind()));
        }
        let start_db = prec.from_nanos(t0);
        let end_db = prec.from_nanos(t1);
        let sql = format!(
            "SELECT COUNT(*) FROM `{table}` WHERE ts >= {start_db} AND ts <= {end_db} INTERVAL(1s)"
        );
        let windows = match self.pool.exec_sql(&sql).await {
            Ok(r) if r.code == 0 => r.rows.len(),
            Ok(r) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("INTERVAL code={}", r.code));
            }
            Err(e) => {
                let _ = self.drop_stable(&table).await;
                return CheckOutcome::Fail(format!("INTERVAL: {:?}", e.kind()));
            }
        };
        let _ = self.drop_stable(&table).await;
        if windows == 0 {
            return CheckOutcome::Fail("INTERVAL 无窗口行".into());
        }
        CheckOutcome::Ok
    }

    async fn check_last_row(&self, ctx: &ValidationContext) -> CheckOutcome {
        let table = ctx.stable("last");
        let now_ns = now_ns();
        let prec = self.pool.precision();
        let t0 = prec.to_nanos(prec.from_nanos(now_ns));
        let t1 = prec.to_nanos(prec.from_nanos(now_ns.saturating_add(5_000_000)));
        // scale=2 → 123.45 / 678.90，便于 LAST_ROW 文本比对
        let points =
            vec![sample_tick("LR", t0, 10_000, 10_100), sample_tick("LR", t1, 12_345, 67_890)];
        if let Err(e) = self.pool.write_batch(&table, &points).await {
            let _ = self.drop_stable(&table).await;
            return CheckOutcome::Fail(format!("write: {:?}", e.kind()));
        }
        // LAST_ROW 返回最后一行；NCHAR 文本应含末行 decimal 表示
        let sql = format!("SELECT LAST_ROW(bid), LAST_ROW(ask) FROM `{table}`");
        let outcome = match self.pool.exec_sql(&sql).await {
            Ok(r) if r.code == 0 && !r.rows.is_empty() => {
                let row = &r.rows[0];
                let bid = row.first().map(String::as_str).unwrap_or("");
                let ask = row.get(1).map(String::as_str).unwrap_or("");
                // NCHAR 文本可能裁掉尾零（678.90 → 678.9）；按 f64 容差比对
                let bid_ok = decimal_text_eq(bid, 123.45);
                let ask_ok = decimal_text_eq(ask, 678.90);
                if bid_ok && ask_ok {
                    CheckOutcome::Ok
                } else {
                    CheckOutcome::Fail(format!("LAST_ROW 值不符: bid={bid:?} ask={ask:?}"))
                }
            }
            Ok(r) => CheckOutcome::Fail(format!("LAST_ROW code={} rows={}", r.code, r.rows.len())),
            Err(e) => CheckOutcome::Fail(format!("LAST_ROW: {:?}", e.kind())),
        };
        let _ = self.drop_stable(&table).await;
        outcome
    }

    async fn check_tmq_subscribe(&self, _ctx: &ValidationContext) -> CheckOutcome {
        // 本 crate 生产面为 REST TimeSeriesStore，未实现 TMQ 客户端。
        CheckOutcome::Skip("taosx 未实现 TMQ 订阅客户端（NO-GO / 诚实边界）".into())
    }

    async fn check_db_config(&self, ctx: &ValidationContext) -> CheckOutcome {
        let current = self.pool.precision();
        let current_s = precision_str(current);
        match ctx.config.expected_precision.as_deref().or(self.config.expected_precision.as_deref())
        {
            None => {
                // 无期望值：仅确认 precision API 可读
                let _ = current_s;
                CheckOutcome::Ok
            }
            Some(exp) => match TsPrecision::parse(exp) {
                Some(want) if want == current => CheckOutcome::Ok,
                Some(want) => CheckOutcome::Fail(format!(
                    "precision 期望 {} 实际 {current_s}",
                    precision_str(want)
                )),
                None => CheckOutcome::Fail(format!("expected_precision 非法: {exp}")),
            },
        }
    }

    async fn drop_stable(&self, table: &str) -> Result<(), String> {
        match self.pool.exec_sql(&format!("DROP STABLE IF EXISTS `{table}`")).await {
            Ok(r) if r.code == 0 => Ok(()),
            Ok(r) => Err(format!("DROP STABLE code={}", r.code)),
            Err(e) => Err(format!("DROP STABLE: {:?}", e.kind())),
        }
    }
}

fn sample_tick(symbol: &str, ts_ns: i64, bid: i128, ask: i128) -> Tick {
    Tick {
        symbol: symbol.into(),
        bid: Price::new(
            Decimal::try_new(bid, 2).unwrap_or_else(|_| Decimal::try_new(0, 2).expect("0")),
        ),
        ask: Price::new(
            Decimal::try_new(ask, 2).unwrap_or_else(|_| Decimal::try_new(0, 2).expect("0")),
        ),
        ts: ts_ns,
    }
}

fn now_ns() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as i64).unwrap_or(0)
}

fn precision_str(p: TsPrecision) -> &'static str {
    match p {
        TsPrecision::Ms => "ms",
        TsPrecision::Us => "us",
        TsPrecision::Ns => "ns",
    }
}

fn parse_count_cell(row: Option<&Vec<String>>) -> Option<i64> {
    let cell = row?.first()?;
    // TDengine 可能返回 "2" 或 "2.0"
    let trimmed = cell.trim();
    if let Ok(n) = trimmed.parse::<i64>() {
        return Some(n);
    }
    trimmed.parse::<f64>().ok().map(|f| f as i64)
}

/// 比较 NCHAR 小数文本与期望值（允许尾零裁剪与极小浮点误差）。
fn decimal_text_eq(text: &str, expected: f64) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    match t.parse::<f64>() {
        Ok(v) => (v - expected).abs() < 1e-9,
        Err(_) => false,
    }
}

/// 细节脱敏：去掉密码/完整 URL 凭据痕迹。
fn redact_detail(s: &str) -> String {
    let mut out = s.to_owned();
    // 粗粒度：password= / ://user:pass@
    if let Some(at) = out.find("://") {
        if let Some(at_sign) = out[at..].find('@') {
            let end = at + at_sign + 1;
            out.replace_range(at..end, "://***@");
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
    use crate::config::TaosConfig;
    use std::time::Duration;

    fn offline_pool() -> TaosPool {
        let cfg = TaosConfig {
            host: "127.0.0.1".into(),
            port: 1, // 不可达
            timeout: Duration::from_millis(80),
            acquire_timeout: Duration::from_millis(80),
            ..TaosConfig::default()
        };
        TaosPool::connect_without_ping(cfg).expect("offline pool")
    }

    #[test]
    fn catalog_matches_spec_count() {
        let cat = TaosValidator::static_catalog();
        assert_eq!(cat.len(), 9);
        assert!(cat.iter().all(|d| d.id.starts_with("taos.")));
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Basic).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::ReadWrite).count(), 1);
        assert_eq!(cat.iter().filter(|d| d.level == CheckLevel::Full).count(), 7);
        assert!(cat.iter().any(|d| d.id == "taos.full.tmq_subscribe" && d.destructive));
    }

    #[tokio::test]
    async fn probe_basic_fails_and_short_circuits_full() {
        let v = TaosValidator::new(offline_pool());
        let report = v.run(CheckLevel::Full).await;
        assert_eq!(report.module, "taos");
        assert!(!report.passed);
        let ping = report.items.iter().find(|i| i.id == "taos.basic.ping").expect("ping");
        assert_eq!(ping.status, CheckStatus::Failed);
        let rw = report.items.iter().find(|i| i.id == "taos.rw.insert_query").expect("rw");
        assert_eq!(rw.status, CheckStatus::Skipped);
        let full_skipped = report
            .items
            .iter()
            .filter(|i| i.id.starts_with("taos.full.") && i.status == CheckStatus::Skipped)
            .count();
        assert_eq!(full_skipped, 7);
    }

    #[tokio::test]
    async fn skip_config_marks_item() {
        let mut cfg = TaosSelfCheckConfig::default();
        cfg.skip.insert("taos.basic.ping".into());
        let v = TaosValidator::new(offline_pool()).with_config(cfg);
        let report = v.run(CheckLevel::Basic).await;
        let ping = report.items.iter().find(|i| i.id == "taos.basic.ping").expect("ping");
        assert_eq!(ping.status, CheckStatus::Skipped);
        assert!(report.passed);
    }

    #[tokio::test]
    async fn tmq_is_skipped_when_network_items_skipped() {
        // 不可达时 skip 掉 basic/rw 与会访问网络的 full 项，仅跑 tmq → Skip（诚实边界）
        let mut cfg = TaosSelfCheckConfig::default();
        for id in [
            "taos.basic.ping",
            "taos.rw.insert_query",
            "taos.full.stable_ddl",
            "taos.full.auto_subtable",
            "taos.full.tag_filter",
            "taos.full.interval_window",
            "taos.full.last_row",
            "taos.full.db_config",
        ] {
            cfg.skip.insert(id.into());
        }
        let v = TaosValidator::new(offline_pool()).with_config(cfg);
        let report = v.run(CheckLevel::Full).await;
        let tmq = report.items.iter().find(|i| i.id == "taos.full.tmq_subscribe").expect("tmq");
        assert_eq!(tmq.status, CheckStatus::Skipped, "{tmq:?}");
        assert!(tmq.detail.as_ref().is_some_and(|d| d.contains("TMQ") || d.contains("skip")));
        assert!(report.passed);
    }
}
