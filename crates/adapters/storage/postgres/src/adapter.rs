//! Postgres 内存 scaffold：`Repository` + `TxRunner`。
//!
//! 注意：scaffold 的 `begin_tx` 使用本模块本地 [`ScaffoldTxContext`]，
//! **不**依赖 test-support 的 `contract-testkit`（禁止 production graph 泄漏）。

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use contracts::{Repository, TxContext, TxRunner};
use kernel::{XError, XResult};

/// 简单可持久化记录（scaffold entity）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub id: String,
    pub data: Vec<u8>,
}

/// Scaffold 事务上下文：仅形状可测，**不**与 rows 绑定真实事务边界。
#[derive(Debug, Default)]
struct ScaffoldTxContext {
    committed: bool,
    rolled_back: bool,
}

#[async_trait]
impl TxContext for ScaffoldTxContext {
    async fn commit(&mut self) -> XResult<()> {
        self.committed = true;
        self.rolled_back = false;
        Ok(())
    }

    async fn rollback(&mut self) -> XResult<()> {
        self.rolled_back = true;
        self.committed = false;
        Ok(())
    }
}

/// Postgres 适配器（进程内；非真实客户端）。
pub struct PostgresAdapter {
    name: String,
    endpoint: String,
    rows: Mutex<HashMap<String, Record>>,
}

impl PostgresAdapter {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self { name: name.into(), endpoint: endpoint.into(), rows: Mutex::new(HashMap::new()) }
    }

    pub fn local() -> Self {
        Self::new("postgres-local", "postgres://127.0.0.1:5432/postgres")
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn lock(&self) -> XResult<std::sync::MutexGuard<'_, HashMap<String, Record>>> {
        self.rows.lock().map_err(|e| XError::internal(format!("rows lock poisoned: {e}")))
    }
}

#[async_trait]
impl Repository<Record, String> for PostgresAdapter {
    async fn find(&self, id: String) -> XResult<Option<Record>> {
        Ok(self.lock()?.get(&id).cloned())
    }

    async fn save(&self, entity: &Record) -> XResult<()> {
        self.lock()?.insert(entity.id.clone(), entity.clone());
        Ok(())
    }
}

#[async_trait]
impl TxRunner for PostgresAdapter {
    async fn begin_tx(&self) -> XResult<Box<dyn TxContext>> {
        Ok(Box::new(ScaffoldTxContext::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contracts::run_tx_lifecycle;

    #[tokio::test]
    async fn repository_roundtrip() {
        let a = PostgresAdapter::local();
        let r = Record { id: "1".into(), data: b"x".to_vec() };
        a.save(&r).await.expect("save");
        assert_eq!(a.find("1".into()).await.expect("find"), Some(r));
    }

    #[tokio::test]
    async fn tx_runner_commit_path() {
        let a = PostgresAdapter::local();
        let v = run_tx_lifecycle(&a, || async move { Ok::<_, kernel::XError>(42) })
            .await
            .expect("事务成功");
        assert_eq!(v, 42);
    }

    #[test]
    fn name_endpoint() {
        let a = PostgresAdapter::local();
        assert_eq!(a.name(), "postgres-local");
    }
}
