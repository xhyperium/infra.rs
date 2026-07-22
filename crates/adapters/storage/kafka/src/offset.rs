//! 应用层 offset commit 存储。
//!
//! `rskafka` 无 consumer group coordinator，因此 offset 由应用显式持久化。
//!
//! - [`OffsetCommitStore::commit`]：将 **next-to-read** 写为 `delivered_offset + 1`
//! - [`OffsetCommitStore::committed`]：读取 next-to-read（若无则 `None`）

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use kernel::{XError, XResult};
use tokio::sync::Mutex;

/// Offset 提交存储抽象。
///
/// 存储的是 **下一次应读取的 offset**（Kafka group 语义中的 committed offset）。
#[async_trait]
pub trait OffsetCommitStore: Send + Sync {
    /// 读取 `(topic, partition)` 已提交的 next-to-read offset。
    async fn committed(&self, topic: &str, partition: i32) -> XResult<Option<i64>>;

    /// 提交已成功处理的消息 offset：内部写入 `next = offset + 1`。
    async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()>;
}

/// 内存 offset 表（进程内；测试与单实例草稿默认）。
#[derive(Debug, Default)]
pub struct MemoryOffsetStore {
    inner: Mutex<HashMap<(String, i32), i64>>,
}

impl MemoryOffsetStore {
    /// 新建空表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 包装为 `Arc`。
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// 单调写入 next-to-read（测试辅助）。
    ///
    /// # Errors
    ///
    /// `next_offset` 为负时返回 `Invalid`。
    pub async fn put_next(&self, topic: &str, partition: i32, next_offset: i64) -> XResult<()> {
        if next_offset < 0 {
            return Err(XError::invalid("kafkax: next offset 不能为负"));
        }
        let mut g = self.inner.lock().await;
        let entry = g.entry((topic.to_string(), partition)).or_insert(next_offset);
        *entry = (*entry).max(next_offset);
        Ok(())
    }
}

#[async_trait]
impl OffsetCommitStore for MemoryOffsetStore {
    async fn committed(&self, topic: &str, partition: i32) -> XResult<Option<i64>> {
        let g = self.inner.lock().await;
        Ok(g.get(&(topic.to_string(), partition)).copied())
    }

    async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()> {
        if offset < 0 {
            return Err(XError::invalid("kafkax: commit offset 不能为负"));
        }
        // next-to-read = delivered + 1；溢出必须显式失败，不能饱和后伪报成功。
        let next =
            offset.checked_add(1).ok_or_else(|| XError::invalid("kafkax: commit offset 溢出"))?;
        let mut g = self.inner.lock().await;
        let entry = g.entry((topic.to_string(), partition)).or_insert(next);
        *entry = (*entry).max(next);
        Ok(())
    }
}

/// 可选文件持久化：每行 `topic\tpartition\tnext_offset`。
#[derive(Debug)]
pub struct FileOffsetStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl FileOffsetStore {
    /// 绑定文件路径（首次 commit 时创建父目录）。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), lock: Arc::new(Mutex::new(())) }
    }

    /// 路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load_map(path: &Path) -> XResult<HashMap<(String, i32), i64>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|error| XError::unavailable("kafkax offset 读取失败").with_source(error))?;
        let mut map = HashMap::new();
        for (lineno, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split('\t');
            let topic = parts.next().ok_or_else(|| {
                XError::invalid(format!("kafkax offset 行 {}: 缺 topic", lineno + 1))
            })?;
            let partition: i32 = parts
                .next()
                .ok_or_else(|| {
                    XError::invalid(format!("kafkax offset 行 {}: 缺 partition", lineno + 1))
                })?
                .parse()
                .map_err(|error| {
                    XError::invalid(format!("kafkax offset 行 {}: partition 非法", lineno + 1))
                        .with_source(error)
                })?;
            let next: i64 = parts
                .next()
                .ok_or_else(|| {
                    XError::invalid(format!("kafkax offset 行 {}: 缺 next_offset", lineno + 1))
                })?
                .parse()
                .map_err(|error| {
                    XError::invalid(format!("kafkax offset 行 {}: next_offset 非法", lineno + 1))
                        .with_source(error)
                })?;
            map.insert((topic.to_string(), partition), next);
        }
        Ok(map)
    }

    fn save_map(path: &Path, map: &HashMap<(String, i32), i64>) -> XResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    XError::unavailable("kafkax offset 创建目录失败").with_source(error)
                })?;
            }
        }
        let mut lines: Vec<String> =
            map.iter().map(|((t, p), n)| format!("{t}\t{p}\t{n}")).collect();
        lines.sort();
        let body = lines.join("\n");
        let tmp = path.with_extension("tmp");
        let mut file =
            OpenOptions::new().create(true).truncate(true).write(true).open(&tmp).map_err(
                |error| XError::unavailable("kafkax offset 打开临时文件失败").with_source(error),
            )?;
        file.write_all(body.as_bytes()).map_err(|error| {
            XError::unavailable("kafkax offset 写临时文件失败").with_source(error)
        })?;
        file.sync_all().map_err(|error| {
            XError::unavailable("kafkax offset sync 临时文件失败").with_source(error)
        })?;
        std::fs::rename(&tmp, path)
            .map_err(|error| XError::unavailable("kafkax offset rename 失败").with_source(error))?;
        let parent =
            path.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
        OpenOptions::new()
            .read(true)
            .open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| {
                XError::unavailable("kafkax offset sync 父目录失败").with_source(error)
            })?;
        Ok(())
    }
}

#[async_trait]
impl OffsetCommitStore for FileOffsetStore {
    async fn committed(&self, topic: &str, partition: i32) -> XResult<Option<i64>> {
        let path = self.path.clone();
        let guard = Arc::clone(&self.lock).lock_owned().await;
        let topic = topic.to_string();
        tokio::task::spawn_blocking(move || {
            let _guard = guard;
            let map = Self::load_map(&path)?;
            Ok(map.get(&(topic, partition)).copied())
        })
        .await
        .map_err(|error| XError::internal("kafkax offset 读取任务失败").with_source(error))?
    }

    async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()> {
        if offset < 0 {
            return Err(XError::invalid("kafkax: commit offset 不能为负"));
        }
        let next =
            offset.checked_add(1).ok_or_else(|| XError::invalid("kafkax: commit offset 溢出"))?;
        let path = self.path.clone();
        let guard = Arc::clone(&self.lock).lock_owned().await;
        let topic = topic.to_string();
        tokio::task::spawn_blocking(move || {
            let _guard = guard;
            let mut map = Self::load_map(&path)?;
            let entry = map.entry((topic, partition)).or_insert(next);
            *entry = (*entry).max(next);
            Self::save_map(&path, &map)
        })
        .await
        .map_err(|error| XError::internal("kafkax offset 写入任务失败").with_source(error))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_put_get_commit_roundtrip() {
        let store = MemoryOffsetStore::new();
        assert!(store.committed("t", 0).await.expect("读取位点").is_none());

        store.commit("t", 0, 5).await.expect("提交位点");
        // next-to-read = 5 + 1
        assert_eq!(store.committed("t", 0).await.expect("读取位点"), Some(6));

        store.commit("t", 0, 10).await.expect("再次提交位点");
        assert_eq!(store.committed("t", 0).await.expect("读取位点"), Some(11));

        // 不同分区独立
        store.commit("t", 1, 0).await.expect("提交分区一位点");
        assert_eq!(store.committed("t", 1).await.expect("读取位点"), Some(1));
        assert_eq!(store.committed("t", 0).await.expect("读取位点"), Some(11));
    }

    #[tokio::test]
    async fn memory_reject_negative_offset() {
        let store = MemoryOffsetStore::new();
        let err = store.commit("t", 0, -1).await.expect_err("负 offset 必须失败");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn memory_commit_is_monotonic_and_rejects_overflow() {
        let store = MemoryOffsetStore::new();
        store.put_next("t", 0, 11).await.expect("写入初始下一位点");
        store.put_next("t", 0, 4).await.expect("旧位点应幂等");
        store.commit("t", 0, 10).await.expect("向前提交");
        store.commit("t", 0, 3).await.expect("旧提交应幂等");
        assert_eq!(store.committed("t", 0).await.expect("读取位点"), Some(11));
        let err = store.commit("t", 0, i64::MAX).await.expect_err("溢出必须失败");
        assert_eq!(err.kind(), kernel::ErrorKind::Invalid);
        assert!(err.context().contains("溢出"));
        let negative_next = store.put_next("t", 0, -1).await.expect_err("负 next 必须失败");
        assert_eq!(negative_next.kind(), kernel::ErrorKind::Invalid);
    }

    #[tokio::test]
    async fn concurrent_memory_commits_keep_high_watermark() {
        let store = Arc::new(MemoryOffsetStore::new());
        let mut tasks = Vec::new();
        for offset in [100, 3, 50, 99, 1] {
            let store = Arc::clone(&store);
            tasks.push(tokio::spawn(async move { store.commit("t", 0, offset).await }));
        }
        for task in tasks {
            task.await.expect("等待并发任务").expect("并发提交");
        }
        assert_eq!(store.committed("t", 0).await.expect("读取位点"), Some(101));
    }

    #[tokio::test]
    async fn file_store_roundtrip() {
        let dir = std::env::temp_dir().join(format!("kafkax-offset-{}", std::process::id()));
        let create_dir = dir.clone();
        tokio::task::spawn_blocking(move || {
            let _ = std::fs::remove_dir_all(&create_dir);
            std::fs::create_dir_all(create_dir)
        })
        .await
        .expect("等待创建临时目录")
        .expect("创建临时目录");
        let path = dir.join("offsets.tsv");
        let store = FileOffsetStore::new(&path);
        store.commit("orders", 2, 99).await.expect("提交文件位点");
        assert_eq!(store.committed("orders", 2).await.expect("读取文件位点"), Some(100));

        // 重新打开应能读到
        let store2 = FileOffsetStore::new(&path);
        assert_eq!(store2.committed("orders", 2).await.expect("重新读取文件位点"), Some(100));
        store2.commit("orders", 2, 10).await.expect("旧文件位点应幂等");
        assert_eq!(store2.committed("orders", 2).await.expect("读取文件位点"), Some(100));
        tokio::task::spawn_blocking(move || std::fs::remove_dir_all(dir))
            .await
            .expect("等待清理临时目录")
            .expect("清理临时目录");
    }
}
