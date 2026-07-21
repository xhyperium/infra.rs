//! evidence —— L1 审计证据追加面（bootstrap 注入；非完整 monorepo wire 协议）。
//!
//! | 类型 | 说明 |
//! |------|------|
//! | [`EvidenceError`] | 追加失败（Durability / Unavailable） |
//! | [`EvidenceAppender`] | 对象安全追加 trait |
//! | [`InMemoryEvidenceAppender`] | 进程内实现（测试 / 开发默认） |
//! | [`AppendReceipt`] | 成功回执（序号 + 名称） |
//!
//! **非目标**：远程持久化、签名链、跨进程证据总线、完整 AppendRequest wire。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

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
}
