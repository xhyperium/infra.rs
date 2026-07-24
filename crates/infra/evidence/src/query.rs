//! 证据查询面。

use crate::{AppendReceipt, EvidenceError, InMemoryEvidenceAppender};

/// 只读查询 trait（对象安全）。
pub trait EvidenceQuery: Send + Sync {
    /// 按事件名过滤（精确匹配）。
    fn query_by_name(&self, name: &str) -> Result<Vec<AppendReceipt>, EvidenceError>;
    /// 按序号闭区间 `[seq_start, seq_end]` 过滤（含端点）。
    fn query_range(
        &self,
        seq_start: u64,
        seq_end: u64,
    ) -> Result<Vec<AppendReceipt>, EvidenceError>;
    /// 列出全部（按 seq 升序）。
    fn list_all(&self) -> Result<Vec<AppendReceipt>, EvidenceError>;
}

impl EvidenceQuery for InMemoryEvidenceAppender {
    fn query_by_name(&self, name: &str) -> Result<Vec<AppendReceipt>, EvidenceError> {
        Ok(self.list_all()?.into_iter().filter(|r| r.name == name).collect())
    }

    fn query_range(
        &self,
        seq_start: u64,
        seq_end: u64,
    ) -> Result<Vec<AppendReceipt>, EvidenceError> {
        Ok(self
            .list_all()?
            .into_iter()
            .filter(|r| r.seq >= seq_start && r.seq <= seq_end)
            .collect())
    }

    fn list_all(&self) -> Result<Vec<AppendReceipt>, EvidenceError> {
        let g = self.inner_lock()?;
        let mut out = Vec::with_capacity(g.names.len());
        for (i, name) in g.names.iter().enumerate() {
            let seq = (i as u64).saturating_add(1);
            out.push(AppendReceipt { name: name.clone(), seq });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvidenceAppender;

    #[test]
    fn query_filters() {
        let a = InMemoryEvidenceAppender::new();
        a.append_named("a").unwrap();
        a.append_named("b").unwrap();
        a.append_named("a").unwrap();
        assert_eq!(a.list_all().unwrap().len(), 3);
        assert_eq!(a.query_by_name("a").unwrap().len(), 2);
        let range = a.query_range(2, 3).unwrap();
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].seq, 2);
    }
}
