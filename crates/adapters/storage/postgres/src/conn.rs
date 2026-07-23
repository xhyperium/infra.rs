//! 池化连接句柄 [`PgConnection`]。

use bytes::Bytes;
use deadpool_postgres::Object;
use futures_util::{SinkExt, StreamExt};
use kernel::{XError, XResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;

use crate::error::map_tokio_error;
use crate::tx::PgTransaction;

/// 单次 `COPY IN` 默认最大载荷（16 MiB）。
pub const DEFAULT_COPY_IN_MAX_BYTES: usize = 16 * 1024 * 1024;

/// 单次 `COPY OUT` 默认最大载荷（16 MiB）。
pub const DEFAULT_COPY_OUT_MAX_BYTES: usize = 16 * 1024 * 1024;

/// 从连接池借出的连接（归还由 drop 完成）。
pub struct PgConnection {
    pub(crate) client: Option<Object>,
    pub(crate) operation_timeout: Duration,
    raw_exposed: AtomicBool,
}

/// 池对象取消守卫。
///
/// 只要异步操作未明确调用 release，Drop 就把连接从 deadpool 分离。
/// 因此外层 timeout、任务 abort 或 future drop 都不会把未知状态连接归池。
pub(crate) struct PooledObjectGuard {
    object: Option<Object>,
}

impl PooledObjectGuard {
    pub(crate) fn new(object: Object) -> Self {
        Self { object: Some(object) }
    }

    pub(crate) fn object(&self) -> XResult<&Object> {
        self.object.as_ref().ok_or_else(|| XError::invariant("Postgres 连接守卫为空"))
    }

    pub(crate) fn release(mut self) -> XResult<Object> {
        self.object.take().ok_or_else(|| XError::invariant("Postgres 连接守卫重复释放"))
    }
}

impl Drop for PooledObjectGuard {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            drop(Object::take(object));
        }
    }
}

impl PgConnection {
    /// 包装 deadpool 对象。
    pub(crate) fn new(client: Object, operation_timeout: Duration) -> Self {
        Self { client: Some(client), operation_timeout, raw_exposed: AtomicBool::new(false) }
    }

    fn take_guard(&mut self) -> XResult<PooledObjectGuard> {
        self.client
            .take()
            .map(PooledObjectGuard::new)
            .ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }

    /// 参数化 `EXECUTE`，返回影响行数。
    ///
    /// # 安全
    /// 调用方必须使用 `$1..$N` 占位符；禁止字符串拼接用户输入。
    pub async fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<u64> {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "postgresx", param_count = params.len(), "conn.execute");

        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.execute(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres execute 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，期望恰好一行。
    pub async fn query_one(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Row> {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "postgresx", param_count = params.len(), "conn.query_one");

        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_one(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query_one 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 参数化查询，返回 0..N 行。
    pub async fn query(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> XResult<Vec<Row>> {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "postgresx", param_count = params.len(), "conn.query");

        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query(sql, params)).await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query 超时；连接已丢弃").with_source(error))
            }
        }
    }

    /// 可选单行：0 行 → `Ok(None)`，>1 行 → 错误。
    pub async fn query_opt(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> XResult<Option<Row>> {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "postgresx", param_count = params.len(), "conn.query_opt");

        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.query_opt(sql, params))
            .await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres query_opt 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 执行多语句 SQL（simple query / `batch_execute`）。
    ///
    /// 仅用于**受信任**脚本（如 migration）；禁止拼接用户输入。
    pub async fn batch_execute(&mut self, sql: &str) -> XResult<()> {
        #[cfg(feature = "tracing")]
        tracing::trace!(target: "postgresx", sql_len = sql.len(), "conn.batch_execute");

        if sql.trim().is_empty() {
            return Err(XError::invalid("batch_execute sql 不能为空"));
        }
        let guard = self.take_guard()?;
        match tokio::time::timeout(self.operation_timeout, guard.object()?.batch_execute(sql)).await
        {
            Ok(result) => {
                self.client = Some(guard.release()?);
                result.map_err(map_tokio_error)
            }
            Err(error) => Err(XError::deadline_exceeded("Postgres batch_execute 超时；连接已丢弃")
                .with_source(error)),
        }
    }

    /// `COPY ... FROM STDIN`：将 `data` 作为单块写入，返回影响行数。
    ///
    /// - `statement` 必须是完整 `COPY ... FROM STDIN ...` SQL（**禁止**拼接不可信标识符）
    /// - 载荷上限 [`DEFAULT_COPY_IN_MAX_BYTES`]；超时或错误时连接脱池
    pub async fn copy_in_bytes(&mut self, statement: &str, data: &[u8]) -> XResult<u64> {
        if statement.trim().is_empty() {
            return Err(XError::invalid("COPY IN statement 不能为空"));
        }
        if data.len() > DEFAULT_COPY_IN_MAX_BYTES {
            return Err(XError::invalid(format!(
                "COPY IN 载荷 {} 字节超过上限 {}",
                data.len(),
                DEFAULT_COPY_IN_MAX_BYTES
            )));
        }
        let guard = self.take_guard()?;
        let statement = statement.to_owned();
        let payload = Bytes::copy_from_slice(data);
        let fut = async {
            let sink = guard.object()?.copy_in(&statement).await.map_err(map_tokio_error)?;
            let mut sink = std::pin::pin!(sink);
            sink.send(payload).await.map_err(map_tokio_error)?;
            sink.finish().await.map_err(map_tokio_error)
        };
        match tokio::time::timeout(self.operation_timeout, fut).await {
            Ok(Ok(rows)) => {
                self.client = Some(guard.release()?);
                Ok(rows)
            }
            Ok(Err(error)) => Err(error),
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres COPY IN 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// `COPY ... TO STDOUT`：聚合数据块，受 `max_bytes` 上限约束。
    ///
    /// - `max_bytes == 0` 时使用 [`DEFAULT_COPY_OUT_MAX_BYTES`]
    /// - 超过上限返回 `Invalid` 并脱池（流可能未读完，连接不可复用）
    pub async fn copy_out_bytes(&mut self, statement: &str, max_bytes: usize) -> XResult<Vec<u8>> {
        if statement.trim().is_empty() {
            return Err(XError::invalid("COPY OUT statement 不能为空"));
        }
        let limit = if max_bytes == 0 { DEFAULT_COPY_OUT_MAX_BYTES } else { max_bytes };
        let guard = self.take_guard()?;
        let statement = statement.to_owned();
        let fut = async {
            let stream = guard.object()?.copy_out(&statement).await.map_err(map_tokio_error)?;
            let mut stream = std::pin::pin!(stream);
            let mut out = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(map_tokio_error)?;
                let next = out.len().saturating_add(chunk.len());
                if next > limit {
                    return Err(XError::invalid(format!(
                        "COPY OUT 聚合大小将超过上限 {limit} 字节"
                    )));
                }
                out.extend_from_slice(&chunk);
            }
            Ok(out)
        };
        match tokio::time::timeout(self.operation_timeout, fut).await {
            Ok(Ok(bytes)) => {
                self.client = Some(guard.release()?);
                Ok(bytes)
            }
            Ok(Err(error)) => Err(error),
            Err(error) => {
                Err(XError::deadline_exceeded("Postgres COPY OUT 超时；连接已丢弃")
                    .with_source(error))
            }
        }
    }

    /// 开启事务，消费本连接。
    pub async fn begin(mut self) -> XResult<PgTransaction> {
        if self.raw_exposed.load(Ordering::Acquire) {
            return Err(XError::unavailable(
                "连接曾通过 deprecated 原始 client API 暴露；已隔离，禁止开启事务",
            ));
        }
        let client =
            self.client.take().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))?;
        PgTransaction::begin(client, self.operation_timeout).await
    }

    /// 访问底层 client 的迁移兼容面。
    ///
    /// 调用后连接会被标记为污染并在 Drop 时永久移出池；调用方仍须自行约束原始操作
    /// deadline。请迁移到本类型的参数化 SQL API。
    #[deprecated(note = "请使用 PgConnection 的参数化 SQL API；原始 client 会强制脱池")]
    pub fn client(&self) -> XResult<&Object> {
        self.raw_exposed.store(true, Ordering::Release);
        self.client.as_ref().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }

    /// 可变访问底层 client 的迁移兼容面。
    ///
    /// 调用后连接会被标记为污染并在 Drop 时永久移出池。
    #[deprecated(note = "请使用 PgConnection 的参数化 SQL API；原始 client 会强制脱池")]
    pub fn client_mut(&mut self) -> XResult<&mut Object> {
        self.raw_exposed.store(true, Ordering::Release);
        self.client.as_mut().ok_or_else(|| XError::unavailable("Postgres 连接已丢弃"))
    }
}

impl Drop for PgConnection {
    fn drop(&mut self) {
        if self.raw_exposed.load(Ordering::Acquire)
            && let Some(client) = self.client.take()
        {
            drop(Object::take(client));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_limits_are_documented_and_stable() {
        // 上限作为公开合同；live 覆盖超限路径。
        const _: () = assert!(DEFAULT_COPY_IN_MAX_BYTES >= 1024 * 1024);
        const _: () = assert!(DEFAULT_COPY_OUT_MAX_BYTES >= 1024 * 1024);
        assert_eq!(DEFAULT_COPY_IN_MAX_BYTES, DEFAULT_COPY_OUT_MAX_BYTES);
    }
}
