//! 远程证据追加（可注入传输）。

use std::sync::Mutex;

use crate::{
    AppendReceipt, EvidenceAppender, EvidenceError, InMemoryEvidenceAppender, validate_event_name,
};

/// 远程传输抽象：将已格式化的一行证据送出。
pub trait EvidenceTransport: Send + Sync {
    /// 发送 wire 行（如 `seq\tname` 或签名 hex）。
    fn send_line(&self, line: &str) -> Result<(), EvidenceError>;
}

/// 闭包适配器。
pub struct FnTransport<F>(pub F);

impl<F> EvidenceTransport for FnTransport<F>
where
    F: Fn(&str) -> Result<(), EvidenceError> + Send + Sync,
{
    fn send_line(&self, line: &str) -> Result<(), EvidenceError> {
        (self.0)(line)
    }
}

/// 内存 mock 传输：记录所有行。
#[derive(Debug, Default)]
pub struct MockEvidenceTransport {
    lines: Mutex<Vec<String>>,
    fail_next: Mutex<bool>,
}

impl MockEvidenceTransport {
    /// 构造。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 下次 send 失败。
    pub fn fail_next(&self) {
        *self.fail_next.lock().expect("lock") = true;
    }

    /// 已发送行。
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().expect("lock").clone()
    }
}

impl EvidenceTransport for MockEvidenceTransport {
    fn send_line(&self, line: &str) -> Result<(), EvidenceError> {
        let mut fail = self.fail_next.lock().map_err(|_| EvidenceError::Unavailable)?;
        if *fail {
            *fail = false;
            return Err(EvidenceError::DurabilityFailure);
        }
        drop(fail);
        self.lines.lock().map_err(|_| EvidenceError::Unavailable)?.push(line.to_string());
        Ok(())
    }
}

/// 通过 [`EvidenceTransport`] 追加；本地仍维护序号。
pub struct RemoteEvidenceAppender<T: EvidenceTransport> {
    transport: T,
    local: InMemoryEvidenceAppender,
}

impl<T: EvidenceTransport> RemoteEvidenceAppender<T> {
    /// 构造。
    #[must_use]
    pub fn new(transport: T) -> Self {
        Self { transport, local: InMemoryEvidenceAppender::new() }
    }

    /// 底层传输。
    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// 本地序号侧（查询用）。
    #[must_use]
    pub fn local(&self) -> &InMemoryEvidenceAppender {
        &self.local
    }
}

impl<T: EvidenceTransport> EvidenceAppender for RemoteEvidenceAppender<T> {
    fn append_named(&self, name: &str) -> Result<AppendReceipt, EvidenceError> {
        validate_event_name(name)?;
        // 先占本地序号；传输失败时仍留下本地记录，调用方可见 seq
        let receipt = self.local.append_named(name)?;
        let line = format!("{}\t{}", receipt.seq, receipt.name);
        self.transport.send_line(&line)?;
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_append_path() {
        let mock = MockEvidenceTransport::new();
        let remote = RemoteEvidenceAppender::new(mock);
        let r = remote.append_named("evt").unwrap();
        assert_eq!(r.seq, 1);
        assert_eq!(remote.transport().lines(), vec!["1\tevt".to_string()]);
        remote.transport().fail_next();
        assert_eq!(remote.append_named("x"), Err(EvidenceError::DurabilityFailure));
    }

    #[test]
    fn fn_transport() {
        let t = FnTransport(|line: &str| {
            assert!(!line.is_empty());
            Ok(())
        });
        t.send_line("1\ta").unwrap();
    }
}
