//! evidence —— L1 审计证据追加面（bootstrap 注入；非完整 monorepo wire 协议）。
//!
//! | 类型 | 说明 |
//! |------|------|
//! | [`EvidenceError`] | 追加失败（Durability / Unavailable） |
//! | [`EvidenceAppender`] | 对象安全追加 trait |
//! | [`InMemoryEvidenceAppender`] | 进程内实现（测试 / 开发默认；**非**合规审计） |
//! | [`FileEvidenceAppender`] | 本地文件追加（最小持久化合同，infra-s9t.7） |
//! | [`AppendReceipt`] | 成功回执（序号 + 名称） |
//!
//! **非目标**：远程签名链、跨进程证据总线、完整 AppendRequest wire。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 证据追加错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// 持久化失败。
    DurabilityFailure,
    /// 存储/后端不可用。
    Unavailable,
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DurabilityFailure => write!(f, "evidence durability failure"),
            Self::Unavailable => write!(f, "evidence backend unavailable"),
        }
    }
}

impl std::error::Error for EvidenceError {}

/// 成功追加回执。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendReceipt {
    /// 逻辑事件名。
    pub name: String,
    /// 单调序号（自 1 起，按成功追加递增）。
    pub seq: u64,
}

/// 审计证据追加器（对象安全）。
pub trait EvidenceAppender: Send + Sync {
    /// 按逻辑名追加一条审计事件。
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError>;
}

/// 进程内证据追加器（默认可用实现）。
///
/// - 线程安全（`Mutex`）
/// - 成功路径返回递增 `seq`
/// - [`Self::fail_next`] 可注入一次 `DurabilityFailure`（测试用）
#[derive(Debug, Default)]
pub struct InMemoryEvidenceAppender {
    inner: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    next_seq: u64,
    names: Vec<String>,
    fail_next: bool,
    closed: bool,
}

impl InMemoryEvidenceAppender {
    /// 构造空追加器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 下一次 `append_named` 返回 [`EvidenceError::DurabilityFailure`]。
    pub fn fail_next(&self) {
        self.inner.lock().expect("evidence lock").fail_next = true;
    }

    /// 关闭后端：后续追加返回 [`EvidenceError::Unavailable`]。
    pub fn close(&self) {
        self.inner.lock().expect("evidence lock").closed = true;
    }

    /// 已成功追加的事件名快照（顺序 = 追加顺序）。
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.inner.lock().expect("evidence lock").names.clone()
    }

    /// 成功追加条数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().expect("evidence lock").names.len()
    }

    /// 是否尚无成功追加。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 本地文件证据追加器（最小持久化合同）。
///
/// 每行：`{seq}<TAB>{name}` UTF-8。进程崩溃前已 `flush` 的行可恢复。
/// **不是**签名链 / 远程总线；生产合规仍须上层策略。
pub struct FileEvidenceAppender {
    path: PathBuf,
    inner: Mutex<FileState>,
}

struct FileState {
    next_seq: u64,
    file: std::fs::File,
}

impl FileEvidenceAppender {
    /// 打开或创建 `path` 并追加写入。
    ///
    /// # Errors
    ///
    /// 打开/创建失败 → [`EvidenceError::Unavailable`]。
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, EvidenceError> {
        let path = path.into();
        match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                std::fs::create_dir_all(parent).map_err(|_| EvidenceError::Unavailable)?;
            }
            _ => {}
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .map_err(|_| EvidenceError::Unavailable)?;
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let mut next_seq = 0u64;
        for line in existing.lines() {
            if let Some((seq_s, _)) = line.split_once('\t') {
                if let Ok(s) = seq_s.parse::<u64>() {
                    next_seq = next_seq.max(s);
                }
            }
        }
        Ok(Self { path, inner: Mutex::new(FileState { next_seq, file }) })
    }

    /// 日志文件路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl EvidenceAppender for FileEvidenceAppender {
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError> {
        let mut g = self.inner.lock().map_err(|_| EvidenceError::Unavailable)?;
        g.next_seq = g.next_seq.saturating_add(1);
        let seq = g.next_seq;
        if name.contains('\n') || name.contains('\r') {
            return Err(EvidenceError::DurabilityFailure);
        }
        writeln!(g.file, "{seq}\t{name}").map_err(|_| EvidenceError::DurabilityFailure)?;
        g.file.flush().map_err(|_| EvidenceError::DurabilityFailure)?;
        Ok(AppendReceipt { name: name.to_string(), seq })
    }
}

impl EvidenceAppender for InMemoryEvidenceAppender {
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError> {
        let mut g = self.inner.lock().map_err(|_| EvidenceError::Unavailable)?;
        if g.closed {
            return Err(EvidenceError::Unavailable);
        }
        if g.fail_next {
            g.fail_next = false;
            return Err(EvidenceError::DurabilityFailure);
        }
        g.next_seq = g.next_seq.saturating_add(1);
        let seq = g.next_seq;
        g.names.push(name.to_string());
        Ok(AppendReceipt { name: name.to_string(), seq })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn in_memory_append_receipts_and_order() {
        let a = InMemoryEvidenceAppender::new();
        assert!(a.is_empty());
        let r1 = a.append_named("boot").expect("1");
        assert_eq!(r1.seq, 1);
        assert_eq!(r1.name, "boot");
        let r2 = a.append_named("ready").expect("2");
        assert_eq!(r2.seq, 2);
        assert_eq!(a.len(), 2);
        assert_eq!(a.names(), vec!["boot".to_string(), "ready".to_string()]);
    }

    #[test]
    fn fail_next_then_recover() {
        let a = InMemoryEvidenceAppender::new();
        a.fail_next();
        assert_eq!(a.append_named("x"), Err(EvidenceError::DurabilityFailure));
        let r = a.append_named("y").expect("after fail");
        assert_eq!(r.seq, 1);
        assert_eq!(a.names(), vec!["y".to_string()]);
    }

    #[test]
    fn close_returns_unavailable() {
        let a = InMemoryEvidenceAppender::new();
        a.close();
        assert_eq!(a.append_named("z"), Err(EvidenceError::Unavailable));
    }

    #[test]
    fn trait_object_and_error_display() {
        let a: Arc<dyn EvidenceAppender> = Arc::new(InMemoryEvidenceAppender::new());
        let _ = a.append_named("t").expect("ok");
        assert_eq!(EvidenceError::DurabilityFailure.to_string(), "evidence durability failure");
        assert_eq!(EvidenceError::Unavailable.to_string(), "evidence backend unavailable");
        let _ = format!("{:?}", EvidenceError::Unavailable);
        let _ = format!("{:?}", AppendReceipt { name: "n".into(), seq: 1 });
    }

    #[test]
    fn default_equals_new() {
        let a = InMemoryEvidenceAppender::default();
        assert!(a.is_empty());
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn file_appender_persists_across_open() {
        let dir = std::env::temp_dir().join(format!("evidence-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ev.log");
        {
            let a = FileEvidenceAppender::open(&path).expect("open");
            let r = a.append_named("boot").expect("append");
            assert_eq!(r.seq, 1);
            assert_eq!(a.path(), path.as_path());
        }
        let a2 = FileEvidenceAppender::open(&path).expect("reopen");
        let r2 = a2.append_named("ready").expect("append2");
        assert_eq!(r2.seq, 2);
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("boot"));
        assert!(body.contains("ready"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_appender_rejects_newline_name() {
        let path = std::env::temp_dir().join(format!("evidence-nl-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let a = FileEvidenceAppender::open(&path).expect("open");
        assert_eq!(a.append_named("bad\nname"), Err(EvidenceError::DurabilityFailure));
        assert_eq!(a.append_named("bad\rname"), Err(EvidenceError::DurabilityFailure));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn file_appender_parses_existing_and_skips_junk() {
        let path = std::env::temp_dir().join(format!("evidence-parse-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        std::fs::write(&path, "junk\n2\talready\n").unwrap();
        let a = FileEvidenceAppender::open(&path).expect("open");
        let r = a.append_named("next").expect("append");
        assert_eq!(r.seq, 3);
        assert!(
            a.path().ends_with("evidence-parse-".to_string() + &std::process::id().to_string())
                || a.path().exists()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn file_appender_open_relative_filename() {
        // parent 为空字符串分支：覆盖 if !parent.is_empty() 的 false 路径
        let name = format!("evidence-rel-{}.log", std::process::id());
        let _ = std::fs::remove_file(&name);
        let a = FileEvidenceAppender::open(&name).expect("open relative");
        let _ = a.append_named("x").expect("append");
        let _ = std::fs::remove_file(&name);
    }

    #[test]
    fn file_appender_open_invalid_path_errors() {
        // 父路径是已存在的文件 → create_dir_all 失败 → Unavailable
        let base = std::env::temp_dir().join(format!("evidence-as-file-{}", std::process::id()));
        let _ = std::fs::remove_file(&base);
        let _ = std::fs::remove_dir_all(&base);
        std::fs::write(&base, b"not-a-dir").unwrap();
        let child = base.join("child.log");
        let r = FileEvidenceAppender::open(&child);
        assert_eq!(r.err(), Some(EvidenceError::Unavailable));
        let _ = std::fs::remove_file(&base);
    }
}
