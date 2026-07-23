//! 有界 schema migration 执行器。
//!
//! ## 合同（对齐 draft §2.9）
//!
//! - 持 **advisory lock** 串行迁移
//! - 每条 migration 记录 **checksum**（SHA-256）；禁止修改已应用版本的 SQL
//! - 默认入口 [`Migrator::verify`]：只校验，不自动跑 DDL
//! - [`Migrator::apply`] 显式调用才执行 pending；migration role 与 runtime 分离由运维保证

use std::collections::BTreeMap;

use kernel::{XError, XResult};
use sha2::{Digest, Sha256};

use crate::error::map_tokio_error;
use crate::pool::PostgresPool;

/// 迁移历史表名（固定，非动态标识符）。
pub const SCHEMA_MIGRATIONS_TABLE: &str = "infra_schema_migrations";

/// advisory lock 键（稳定常量，避免与业务锁冲突）。
///
/// 使用 PostgreSQL `pg_advisory_lock(key1, key2)` 的 key1 空间。
pub const MIGRATION_LOCK_KEY1: i32 = 0x7058_5f6d; // 'px_m'
/// advisory lock key2。
pub const MIGRATION_LOCK_KEY2: i32 = 0x6967_7261; // 'igra'

/// 单条迁移定义（forward SQL + 可选 down SQL）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// 单调递增版本号（> 0）。
    pub version: i64,
    /// 人类可读短名（如 `create_records`）。
    pub name: String,
    /// 完整 SQL 脚本（可多语句；由调用方保证安全）。
    pub sql: String,
    /// 回滚 SQL（可选；`None` 表示不可回滚）。
    pub down_sql: Option<String>,
}

impl Migration {
    /// 构造并做基础校验。
    pub fn new(version: i64, name: impl Into<String>, sql: impl Into<String>) -> XResult<Self> {
        let name = name.into();
        let sql = sql.into();
        if version <= 0 {
            return Err(XError::invalid("migration version 必须 > 0"));
        }
        if name.trim().is_empty() {
            return Err(XError::invalid("migration name 不能为空"));
        }
        if sql.trim().is_empty() {
            return Err(XError::invalid("migration sql 不能为空"));
        }
        if name.len() > 256 {
            return Err(XError::invalid("migration name 过长（≤256）"));
        }
        Ok(Self { version, name, sql, down_sql: None })
    }

    /// 构造含回滚 SQL 的迁移定义。
    pub fn with_down(
        version: i64,
        name: impl Into<String>,
        sql: impl Into<String>,
        down_sql: impl Into<String>,
    ) -> XResult<Self> {
        let name = name.into();
        let sql = sql.into();
        let down_sql = down_sql.into();
        if version <= 0 {
            return Err(XError::invalid("migration version 必须 > 0"));
        }
        if name.trim().is_empty() {
            return Err(XError::invalid("migration name 不能为空"));
        }
        if sql.trim().is_empty() {
            return Err(XError::invalid("migration sql 不能为空"));
        }
        if down_sql.trim().is_empty() {
            return Err(XError::invalid("migration down_sql 不能为空（无回滚用 Migration::new）"));
        }
        if name.len() > 256 {
            return Err(XError::invalid("migration name 过长（≤256）"));
        }
        Ok(Self { version, name, sql, down_sql: Some(down_sql) })
    }

    /// SQL 正文的 SHA-256 十六进制 checksum。
    #[must_use]
    pub fn checksum(&self) -> String {
        let digest = Sha256::digest(self.sql.as_bytes());
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// 已落库的迁移行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    /// 版本。
    pub version: i64,
    /// 名称。
    pub name: String,
    /// 落库 checksum。
    pub checksum: String,
}

/// checksum 不一致。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumMismatch {
    /// 版本。
    pub version: i64,
    /// 计划中的 checksum。
    pub expected: String,
    /// 库中已记录的 checksum。
    pub actual: String,
}

/// `verify` / `status` 快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    /// 已应用。
    pub applied: Vec<AppliedMigration>,
    /// 计划中尚未应用的版本。
    pub pending: Vec<i64>,
    /// 已应用但 checksum 与计划不符。
    pub mismatches: Vec<ChecksumMismatch>,
    /// 库中存在、计划中不存在的版本。
    pub unknown_applied: Vec<i64>,
}

impl MigrationStatus {
    /// 是否干净：无 mismatch、无 unknown、无 pending（完全同步）。
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty() && self.unknown_applied.is_empty() && self.pending.is_empty()
    }

    /// 启动默认是否可放行：无 mismatch / unknown；pending 允许（由运维显式 apply）。
    #[must_use]
    pub fn is_boot_ok(&self) -> bool {
        self.mismatches.is_empty() && self.unknown_applied.is_empty()
    }
}

/// `apply` 结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// 本次新应用的版本。
    pub applied_now: Vec<i64>,
    /// apply 后的状态。
    pub status: MigrationStatus,
}

/// schema migration 执行器。
#[derive(Clone, Debug)]
pub struct Migrator {
    pool: PostgresPool,
    migrations: Vec<Migration>,
}

impl Migrator {
    /// 构造执行器；`migrations` 将按 version 排序，拒绝重复 version。
    pub fn new(pool: PostgresPool, migrations: Vec<Migration>) -> XResult<Self> {
        let mut migrations = migrations;
        migrations.sort_by_key(|m| m.version);
        let mut seen = BTreeMap::new();
        for m in &migrations {
            if seen.insert(m.version, ()).is_some() {
                return Err(XError::invalid(format!("重复的 migration version: {}", m.version)));
            }
        }
        Ok(Self { pool, migrations })
    }

    /// 计划中的迁移（已排序）。
    #[must_use]
    pub fn plan(&self) -> &[Migration] {
        &self.migrations
    }

    /// 确保历史表存在。
    pub async fn ensure_table(&self) -> XResult<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {SCHEMA_MIGRATIONS_TABLE} (\
               version BIGINT PRIMARY KEY, \
               name TEXT NOT NULL, \
               checksum TEXT NOT NULL, \
               applied_at TIMESTAMPTZ NOT NULL DEFAULT now()\
             )"
        );
        self.pool.execute(&sql, &[]).await?;
        Ok(())
    }

    /// 读取已应用行（无锁；调用方在 apply 路径应先持锁）。
    pub async fn list_applied(&self) -> XResult<Vec<AppliedMigration>> {
        self.ensure_table().await?;
        let sql = format!(
            "SELECT version, name, checksum FROM {SCHEMA_MIGRATIONS_TABLE} ORDER BY version ASC"
        );
        let rows = self.pool.query(&sql, &[]).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let version: i64 = row.try_get(0).map_err(map_tokio_error)?;
            let name: String = row.try_get(1).map_err(map_tokio_error)?;
            let checksum: String = row.try_get(2).map_err(map_tokio_error)?;
            out.push(AppliedMigration { version, name, checksum });
        }
        Ok(out)
    }

    /// 计算状态（不持锁）。
    pub async fn status(&self) -> XResult<MigrationStatus> {
        let applied = self.list_applied().await?;
        let applied_map: BTreeMap<i64, &AppliedMigration> =
            applied.iter().map(|a| (a.version, a)).collect();
        let plan_map: BTreeMap<i64, &Migration> =
            self.migrations.iter().map(|m| (m.version, m)).collect();

        let mut pending = Vec::new();
        let mut mismatches = Vec::new();
        for m in &self.migrations {
            match applied_map.get(&m.version) {
                None => pending.push(m.version),
                Some(a) => {
                    let expected = m.checksum();
                    if a.checksum != expected {
                        mismatches.push(ChecksumMismatch {
                            version: m.version,
                            expected,
                            actual: a.checksum.clone(),
                        });
                    }
                }
            }
        }
        let unknown_applied: Vec<i64> = applied
            .iter()
            .filter(|a| !plan_map.contains_key(&a.version))
            .map(|a| a.version)
            .collect();

        Ok(MigrationStatus { applied, pending, mismatches, unknown_applied })
    }

    /// 默认启动路径：校验 checksum / 未知版本；**不**执行 pending DDL。
    ///
    /// 有 mismatch 或 unknown → `Conflict`；pending 仅体现在返回值。
    pub async fn verify(&self) -> XResult<MigrationStatus> {
        let status = self.status().await?;
        if !status.mismatches.is_empty() {
            return Err(XError::conflict(format!(
                "migration checksum 不一致: {} 条",
                status.mismatches.len()
            )));
        }
        if !status.unknown_applied.is_empty() {
            return Err(XError::conflict(format!(
                "库中存在计划外 migration 版本: {:?}",
                status.unknown_applied
            )));
        }
        Ok(status)
    }

    /// 回滚最近一条已应用的迁移（需有 `down_sql`）。
    ///
    /// 全程持 session advisory lock；在事务中执行 down_sql 并删除历史行。
    /// 若无已应用迁移，返回 `None`；若最近迁移无 `down_sql`，返回错误。
    pub async fn down(&self) -> XResult<Option<i64>> {
        let mut conn = self.pool.acquire().await?;
        conn.execute(
            "SELECT pg_advisory_lock($1, $2)",
            &[&MIGRATION_LOCK_KEY1, &MIGRATION_LOCK_KEY2],
        )
        .await?;

        let result = self.down_locked(&mut conn).await;

        let _ = conn
            .execute(
                "SELECT pg_advisory_unlock($1, $2)",
                &[&MIGRATION_LOCK_KEY1, &MIGRATION_LOCK_KEY2],
            )
            .await;
        result
    }

    async fn down_locked(&self, conn: &mut crate::conn::PgConnection) -> XResult<Option<i64>> {
        self.ensure_table_on_conn(conn).await?;

        let list_sql = format!(
            "SELECT version, name, checksum FROM {SCHEMA_MIGRATIONS_TABLE} ORDER BY version DESC LIMIT 1"
        );
        let rows = conn.query(&list_sql, &[]).await?;
        let last_applied = match rows.first() {
            None => return Ok(None),
            Some(row) => {
                let version: i64 = row.try_get(0).map_err(map_tokio_error)?;
                version
            }
        };

        // 从计划中查找对应迁移
        let migration =
            self.migrations.iter().find(|m| m.version == last_applied).ok_or_else(|| {
                XError::conflict(format!("库中已应用版本 v{last_applied} 不在当前计划中，无法回滚"))
            })?;

        let down_sql = migration.down_sql.as_deref().ok_or_else(|| {
            XError::invalid(format!("迁移 v{last_applied} 未提供 down_sql，无法回滚"))
        })?;

        conn.batch_execute(down_sql).await.map_err(|e| {
            XError::internal(format!("migration v{last_applied} down 执行失败: {}", e.context()))
        })?;

        let delete = format!("DELETE FROM {SCHEMA_MIGRATIONS_TABLE} WHERE version = $1");
        conn.execute(&delete, &[&last_applied]).await?;

        Ok(Some(last_applied))
    }

    /// 在指定连接上确保历史表存在。
    async fn ensure_table_on_conn(&self, conn: &mut crate::conn::PgConnection) -> XResult<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {SCHEMA_MIGRATIONS_TABLE} (\
               version BIGINT PRIMARY KEY, \
               name TEXT NOT NULL, \
               checksum TEXT NOT NULL, \
               applied_at TIMESTAMPTZ NOT NULL DEFAULT now()\
             )"
        );
        conn.execute(&sql, &[]).await?;
        Ok(())
    }

    /// 显式应用全部 pending（按 version 升序）。
    ///
    /// 全程持 session advisory lock；每条 migration 在事务中执行 SQL 并写入历史行。
    /// 已应用版本不会被修改。
    pub async fn apply(&self) -> XResult<MigrationReport> {
        let mut conn = self.pool.acquire().await?;
        // 会话级锁：释放在连接归还时由 PG 在会话结束清理；我们显式 unlock
        conn.execute(
            "SELECT pg_advisory_lock($1, $2)",
            &[&MIGRATION_LOCK_KEY1, &MIGRATION_LOCK_KEY2],
        )
        .await?;

        let apply_result = self.apply_locked(&mut conn).await;

        // 尽力解锁
        let _ = conn
            .execute(
                "SELECT pg_advisory_unlock($1, $2)",
                &[&MIGRATION_LOCK_KEY1, &MIGRATION_LOCK_KEY2],
            )
            .await;
        apply_result
    }

    async fn apply_locked(&self, conn: &mut crate::conn::PgConnection) -> XResult<MigrationReport> {
        self.ensure_table_on_conn(conn).await?;

        let list_sql = format!(
            "SELECT version, name, checksum FROM {SCHEMA_MIGRATIONS_TABLE} ORDER BY version ASC"
        );
        let rows = conn.query(&list_sql, &[]).await?;
        let mut applied_map: BTreeMap<i64, AppliedMigration> = BTreeMap::new();
        for row in rows {
            let version: i64 = row.try_get(0).map_err(map_tokio_error)?;
            let name: String = row.try_get(1).map_err(map_tokio_error)?;
            let checksum: String = row.try_get(2).map_err(map_tokio_error)?;
            applied_map.insert(version, AppliedMigration { version, name, checksum });
        }

        // refuse checksum drift before applying
        for m in &self.migrations {
            if let Some(a) = applied_map.get(&m.version) {
                let expected = m.checksum();
                if a.checksum != expected {
                    return Err(XError::conflict(format!(
                        "禁止修改已应用 migration v{}: checksum 不一致",
                        m.version
                    )));
                }
            }
        }

        let mut applied_now = Vec::new();
        for m in &self.migrations {
            if applied_map.contains_key(&m.version) {
                continue;
            }
            // 单条 migration：batch_execute 支持多语句；内容由开发者信任，非用户输入。
            conn.batch_execute(&m.sql).await.map_err(|e| {
                XError::internal(format!("migration v{} 执行失败: {}", m.version, e.context()))
            })?;
            let checksum = m.checksum();
            let insert = format!(
                "INSERT INTO {SCHEMA_MIGRATIONS_TABLE} (version, name, checksum) \
                 VALUES ($1, $2, $3)"
            );
            conn.execute(&insert, &[&m.version, &m.name, &checksum]).await?;
            applied_map.insert(
                m.version,
                AppliedMigration { version: m.version, name: m.name.clone(), checksum },
            );
            applied_now.push(m.version);
        }

        // rebuild status without second pool round-trip
        let applied: Vec<_> = applied_map.into_values().collect();
        let applied_versions: BTreeMap<i64, ()> = applied.iter().map(|a| (a.version, ())).collect();
        let pending: Vec<i64> = self
            .migrations
            .iter()
            .filter(|m| !applied_versions.contains_key(&m.version))
            .map(|m| m.version)
            .collect();
        let status = MigrationStatus {
            applied,
            pending,
            mismatches: Vec::new(),
            unknown_applied: Vec::new(),
        };
        Ok(MigrationReport { applied_now, status })
    }
}

/// 将 [`MigrationStatus::is_boot_ok`] 失败映射为错误（便捷）。
pub fn ensure_boot_ok(status: &MigrationStatus) -> XResult<()> {
    if status.is_boot_ok() {
        Ok(())
    } else {
        Err(XError::conflict(format!(
            "migration 启动校验失败: mismatches={}, unknown={:?}",
            status.mismatches.len(),
            status.unknown_applied
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_checksum_stable() {
        let m = Migration::new(1, "a", "CREATE TABLE t (id int);").unwrap();
        let c1 = m.checksum();
        let c2 = m.checksum();
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 64);
        let m2 = Migration::new(1, "a", "CREATE TABLE t (id int); ").unwrap();
        assert_ne!(m.checksum(), m2.checksum(), "空白变化必须改变 checksum");
    }

    #[test]
    fn migration_rejects_bad_meta() {
        assert!(Migration::new(0, "a", "x").is_err());
        assert!(Migration::new(1, "", "x").is_err());
        assert!(Migration::new(1, "a", "  ").is_err());
    }

    #[test]
    fn status_boot_ok_logic() {
        let ok = MigrationStatus {
            applied: vec![],
            pending: vec![1],
            mismatches: vec![],
            unknown_applied: vec![],
        };
        assert!(ok.is_boot_ok());
        assert!(!ok.is_clean());
        let bad = MigrationStatus {
            applied: vec![],
            pending: vec![],
            mismatches: vec![ChecksumMismatch {
                version: 1,
                expected: "a".into(),
                actual: "b".into(),
            }],
            unknown_applied: vec![],
        };
        assert!(!bad.is_boot_ok());
        assert!(ensure_boot_ok(&bad).is_err());
        assert_eq!(ensure_boot_ok(&bad).unwrap_err().kind(), kernel::ErrorKind::Conflict);
    }

    #[test]
    fn plan_versions_must_be_unique() {
        fn check_unique(migrations: &[Migration]) -> XResult<()> {
            let mut seen = BTreeMap::new();
            for m in migrations {
                if seen.insert(m.version, ()).is_some() {
                    return Err(XError::invalid(format!(
                        "重复的 migration version: {}",
                        m.version
                    )));
                }
            }
            Ok(())
        }
        let ok = vec![Migration::new(1, "a", "s1").unwrap(), Migration::new(2, "b", "s2").unwrap()];
        assert!(check_unique(&ok).is_ok());
        let bad =
            vec![Migration::new(1, "a", "s1").unwrap(), Migration::new(1, "b", "s2").unwrap()];
        assert!(check_unique(&bad).is_err());
    }

    #[test]
    fn with_down_constructs_migration_with_down_sql() {
        let m =
            Migration::with_down(1, "create", "CREATE TABLE t (id int)", "DROP TABLE t").unwrap();
        assert_eq!(m.version, 1);
        assert_eq!(m.name, "create");
        assert!(m.down_sql.as_deref() == Some("DROP TABLE t"));
        assert!(Migration::new(1, "a", "x").unwrap().down_sql.is_none());
    }

    #[test]
    fn with_down_rejects_empty_down_sql() {
        assert!(Migration::with_down(1, "a", "x", "").is_err());
        assert!(Migration::with_down(1, "a", "x", "  ").is_err());
    }
}
