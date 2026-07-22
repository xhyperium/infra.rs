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
    lock: Mutex<()>,
}

impl FileOffsetStore {
    /// 绑定文件路径（首次 commit 时创建父目录）。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), lock: Mutex::new(()) }
    }

    /// 路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load_map(&self) -> XResult<HashMap<(String, i32), i64>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let text = std::fs::read_to_string(&self.path)
            .map_err(|e| XError::unavailable(format!("kafkax offset 读取失败: {e}")))?;
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
                .map_err(|_| {
                    XError::invalid(format!("kafkax offset 行 {}: partition 非法", lineno + 1))
                })?;
            let next: i64 = parts
                .next()
                .ok_or_else(|| {
                    XError::invalid(format!("kafkax offset 行 {}: 缺 next_offset", lineno + 1))
                })?
                .parse()
                .map_err(|_| {
                    XError::invalid(format!("kafkax offset 行 {}: next_offset 非法", lineno + 1))
                })?;
            map.insert((topic.to_string(), partition), next);
        }
        Ok(map)
    }

    fn save_map(&self, map: &HashMap<(String, i32), i64>) -> XResult<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| XError::unavailable(format!("kafkax offset 创建目录失败: {e}")))?;
            }
        }
        let mut lines: Vec<String> =
            map.iter().map(|((t, p), n)| format!("{t}\t{p}\t{n}")).collect();
        lines.sort();
        let body = lines.join("\n");
        let tmp = self.path.with_extension("tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| XError::unavailable(format!("kafkax offset 打开临时文件失败: {e}")))?;
        file.write_all(body.as_bytes())
            .map_err(|e| XError::unavailable(format!("kafkax offset 写临时文件失败: {e}")))?;
        file.sync_all()
            .map_err(|e| XError::unavailable(format!("kafkax offset sync 临时文件失败: {e}")))?;
        std::fs::rename(&tmp, &self.path)
            .map_err(|e| XError::unavailable(format!("kafkax offset rename 失败: {e}")))?;
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        OpenOptions::new()
            .read(true)
            .open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| XError::unavailable(format!("kafkax offset sync 父目录失败: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl OffsetCommitStore for FileOffsetStore {
    async fn committed(&self, topic: &str, partition: i32) -> XResult<Option<i64>> {
        let _g = self.lock.lock().await;
        let map = self.load_map()?;
        Ok(map.get(&(topic.to_string(), partition)).copied())
    }

    async fn commit(&self, topic: &str, partition: i32, offset: i64) -> XResult<()> {
        if offset < 0 {
            return Err(XError::invalid("kafkax: commit offset 不能为负"));
        }
        let next =
            offset.checked_add(1).ok_or_else(|| XError::invalid("kafkax: commit offset 溢出"))?;
        let _g = self.lock.lock().await;
        let mut map = self.load_map()?;
        let entry = map.entry((topic.to_string(), partition)).or_insert(next);
        *entry = (*entry).max(next);
        self.save_map(&map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_put_get_commit_roundtrip() {
        let store = MemoryOffsetStore::new();
        assert!(store.committed("t", 0).await.expect("c").is_none());

        store.commit("t", 0, 5).await.expect("commit");
        // next-to-read = 5 + 1
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(6));

        store.commit("t", 0, 10).await.expect("commit2");
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(11));

        // 不同分区独立
        store.commit("t", 1, 0).await.expect("p1");
        assert_eq!(store.committed("t", 1).await.expect("c"), Some(1));
        assert_eq!(store.committed("t", 0).await.expect("c"), Some(11));
    }

    #[tokio::test]
    async fn memory_reject_negative_offset() {
        let store = MemoryOffsetStore::new();
        let err = store.commit("t", 0, -1).await.expect_err("neg");
        assert!(err.to_string().contains("负") || err.context().contains("负"));
    }

    #[tokio::test]
    async fn memory_commit_is_monotonic_and_rejects_overflow() {
        let store = MemoryOffsetStore::new();
        store.commit("t", 0, 10).await.expect("forward");
        store.commit("t", 0, 3).await.expect("stale commit is idempotent");
        assert_eq!(store.committed("t", 0).await.expect("read"), Some(11));
        let err = store.commit("t", 0, i64::MAX).await.expect_err("overflow");
        assert!(err.context().contains("溢出"));
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
            task.await.expect("join").expect("commit");
        }
        assert_eq!(store.committed("t", 0).await.expect("read"), Some(101));
    }

    #[tokio::test]
    async fn file_store_roundtrip() {
        let dir = std::env::temp_dir().join(format!("kafkax-offset-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("offsets.tsv");
        let store = FileOffsetStore::new(&path);
        store.commit("orders", 2, 99).await.expect("commit");
        assert_eq!(store.committed("orders", 2).await.expect("c"), Some(100));

        // 重新打开应能读到
        let store2 = FileOffsetStore::new(&path);
        assert_eq!(store2.committed("orders", 2).await.expect("c"), Some(100));
        store2.commit("orders", 2, 10).await.expect("stale");
        assert_eq!(store2.committed("orders", 2).await.expect("c"), Some(100));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
