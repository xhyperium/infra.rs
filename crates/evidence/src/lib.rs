//! evidence —— L1 审计证据追加面。
//!
//! | 类型 | 说明 |
//! |------|------|
//! | [`EvidenceAppender`] | 对象安全追加 trait |
//! | [`InMemoryEvidenceAppender`] | 进程内（**非**合规审计） |
//! | [`FileEvidenceAppender`] | 本地文件最小持久化 |
//!
//! **非目标**：远程签名链、跨进程总线。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

mod format;
mod inspect;
mod policy;
pub use format::{format_line, max_seq, parse_line, render_log};
pub use inspect::{event_count, seq_is_monotonic, validate_log_text};
pub use policy::{
    BackendClass, allows_as_sole_compliance_store, allows_in_memory_for_compliance, classify_file,
    classify_in_memory, classify_remote, file_appender_is_min_durable, policy_summary,
};

/// 证据错误。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceError {
    /// 持久化失败。
    DurabilityFailure,
    /// 不可用。
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

/// 成功回执。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendReceipt {
    /// 名称。
    pub name: String,
    /// 序号（自 1）。
    pub seq: u64,
}

/// 追加器。
pub trait EvidenceAppender: Send + Sync {
    /// 按名追加。
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError>;
}

/// 名称校验。
pub fn validate_event_name(name: &str) -> Result<(), EvidenceError> {
    if name.is_empty() || name.contains('\n') || name.contains('\r') {
        return Err(EvidenceError::DurabilityFailure);
    }
    Ok(())
}

/// 解析日志。
#[must_use]
pub fn parse_evidence_log(text: &str) -> Vec<(u64, String)> {
    text.lines().filter_map(parse_line).collect()
}

/// 校验后追加。
pub fn append_checked(
    app: &dyn EvidenceAppender,
    name: &str,
) -> Result<AppendReceipt, EvidenceError> {
    validate_event_name(name)?;
    app.append_named(name)
}

/// 批量追加。
pub fn append_batch(
    app: &dyn EvidenceAppender,
    names: &[&str],
) -> Result<Vec<AppendReceipt>, EvidenceError> {
    let mut out = Vec::with_capacity(names.len());
    for n in names {
        out.push(append_checked(app, n)?);
    }
    Ok(out)
}

/// 内存实现。
#[derive(Debug, Default)]
pub struct InMemoryEvidenceAppender {
    inner: Mutex<MemState>,
}

#[derive(Debug, Default)]
struct MemState {
    next_seq: u64,
    names: Vec<String>,
    fail_next: bool,
    closed: bool,
}

impl InMemoryEvidenceAppender {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// 下次失败。
    pub fn fail_next(&self) {
        self.inner.lock().expect("lock").fail_next = true;
    }
    /// 关闭。
    pub fn close(&self) {
        self.inner.lock().expect("lock").closed = true;
    }
    /// 名称快照。
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.inner.lock().expect("lock").names.clone()
    }
    /// 条数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().expect("lock").names.len()
    }
    /// 是否空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

/// 文件追加器。
pub struct FileEvidenceAppender {
    path: PathBuf,
    inner: Mutex<FileState>,
}

struct FileState {
    next_seq: u64,
    file: std::fs::File,
}

impl FileEvidenceAppender {
    /// 打开/创建。
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
        for (s, _) in parse_evidence_log(&existing) {
            next_seq = next_seq.max(s);
        }
        Ok(Self { path, inner: Mutex::new(FileState { next_seq, file }) })
    }
    /// 路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// 读全部条目。
    pub fn read_entries(&self) -> Result<Vec<(u64, String)>, EvidenceError> {
        let text = std::fs::read_to_string(&self.path).map_err(|_| EvidenceError::Unavailable)?;
        Ok(parse_evidence_log(&text))
    }
}

impl EvidenceAppender for FileEvidenceAppender {
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError> {
        let mut g = self.inner.lock().map_err(|_| EvidenceError::Unavailable)?;
        if name.contains('\n') || name.contains('\r') {
            return Err(EvidenceError::DurabilityFailure);
        }
        g.next_seq = g.next_seq.saturating_add(1);
        let seq = g.next_seq;
        writeln!(g.file, "{seq}\t{name}").map_err(|_| EvidenceError::DurabilityFailure)?;
        g.file.flush().map_err(|_| EvidenceError::DurabilityFailure)?;
        Ok(AppendReceipt { name: name.to_string(), seq })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn memory_roundtrip_fail_close() {
        let a = InMemoryEvidenceAppender::new();
        assert!(a.is_empty());
        let r = a.append_named("e1").unwrap();
        assert_eq!(r.seq, 1);
        assert_eq!(a.names(), vec!["e1".to_string()]);
        a.fail_next();
        assert_eq!(a.append_named("e2"), Err(EvidenceError::DurabilityFailure));
        a.append_named("e3").unwrap();
        assert_eq!(a.len(), 2);
        a.close();
        assert_eq!(a.append_named("e4"), Err(EvidenceError::Unavailable));
        let arc = Arc::new(InMemoryEvidenceAppender::default());
        arc.append_named("x").unwrap();
        assert_eq!(arc.len(), 1);
    }

    #[test]
    fn helpers_and_file() {
        assert!(validate_event_name("").is_err());
        assert!(validate_event_name("a\nb").is_err());
        let mem = InMemoryEvidenceAppender::new();
        let batch = append_batch(&mem, &["a", "b"]).unwrap();
        assert_eq!(batch[1].seq, 2);
        assert!(format!("{}", EvidenceError::Unavailable).contains("unavailable"));
        let dir = std::env::temp_dir().join(format!("ev-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("e.log");
        let f = FileEvidenceAppender::open(&path).unwrap();
        append_checked(&f, "one").unwrap();
        assert_eq!(f.read_entries().unwrap().len(), 1);
        assert!(append_checked(&f, "x\ny").is_err());
        let f2 = FileEvidenceAppender::open(&path).unwrap();
        assert_eq!(f2.append_named("two").unwrap().seq, 2);
        assert_eq!(f2.path(), path.as_path());
        let body = render_log(&[(1, "z".into())]).unwrap();
        assert_eq!(max_seq(&body), 1);
        assert_eq!(parse_line("1\tz").unwrap().1, "z");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_with_no_parent_relative() {
        // relative path without parent dirs → parent is empty or "."
        let name = format!("evidence-rel-{}.log", std::process::id());
        let a = FileEvidenceAppender::open(&name).unwrap();
        a.append_named("r").unwrap();
        assert_eq!(a.read_entries().unwrap().len(), 1);
        let _ = std::fs::remove_file(&name);
    }

    #[test]
    fn parse_line_empty_name_skipped() {
        assert!(parse_line("1\t").is_none());
        assert!(parse_line("\tname").is_none());
        assert_eq!(parse_evidence_log("1\t\n2\tok\n").len(), 1);
    }

    #[test]
    fn policy_is_honest() {
        assert!(!allows_in_memory_for_compliance());
        assert!(file_appender_is_min_durable());
        assert!(policy_summary().contains("dev-only"));
    }

    #[test]
    fn file_parent_create_and_invalid() {
        let base = std::env::temp_dir().join(format!("ev-bad-{}", std::process::id()));
        let _ = std::fs::remove_file(&base);
        let _ = std::fs::remove_dir_all(&base);
        std::fs::write(&base, b"not-dir").unwrap();
        let child = base.join("c.log");
        assert_eq!(FileEvidenceAppender::open(&child).err(), Some(EvidenceError::Unavailable));
        let _ = std::fs::remove_file(&base);
    }

    #[test]
    fn format_empty_name_and_file_newline() {
        assert!(parse_line("9\t").is_none());
        let dir = std::env::temp_dir().join(format!("ev-nl-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("n.log");
        let f = FileEvidenceAppender::open(&path).unwrap();
        assert_eq!(f.append_named("bad\nname"), Err(EvidenceError::DurabilityFailure));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
