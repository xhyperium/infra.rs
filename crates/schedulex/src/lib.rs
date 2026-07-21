//! schedulex —— L1 任务 ID 登记表（active SSOT：无真实定时器）。
//!
//! 长期定位是定时/异步任务调度；**当前**公开面仅为内存中的 ID 登记：
//! [`Scheduler::schedule`] / [`Scheduler::cancel`] / [`Scheduler::list`]。
//!
//! “登记一个任务 ID”不等于定时触发或执行任务。
//!
//! 权威规范：`.agents/ssot/infra/schedulex/spec/spec.md`

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

use std::collections::HashMap;

/// 内存任务 ID 登记表。
///
/// 内部为 `HashMap<String, ()>`；不持有时钟、Job、Run 或 async runtime。
#[derive(Debug, Clone)]
pub struct Scheduler {
    tasks: HashMap<String, ()>,
}

impl Scheduler {
    /// 创建空登记表。
    #[must_use]
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }

    /// 登记任务 ID。重复 ID 幂等覆盖（仍只保留一个条目）。
    pub fn schedule(&mut self, id: impl Into<String>) {
        self.tasks.insert(id.into(), ());
    }

    /// 取消任务 ID。
    ///
    /// 返回此前是否存在该 ID（`true` = 已删除；`false` = 本不存在）。
    pub fn cancel(&mut self, id: &str) -> bool {
        self.tasks.remove(id).is_some()
    }

    /// 返回当前所有已登记 ID。
    ///
    /// 顺序未承诺（与 `HashMap` 迭代顺序一致）。
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// SSOT：schedule + list — 登记后 list 含该 ID。
    #[test]
    fn schedule_then_list_contains_id() {
        let mut s = Scheduler::new();
        s.schedule("job-a");
        let ids: HashSet<_> = s.list().into_iter().collect();
        assert_eq!(ids, HashSet::from(["job-a".to_string()]));
    }

    /// SSOT：cancel 存在的 ID 返回 true，且 list 不再包含。
    #[test]
    fn cancel_present_returns_true_and_removes() {
        let mut s = Scheduler::new();
        s.schedule("job-b");
        assert!(s.cancel("job-b"));
        assert!(s.list().is_empty());
    }

    /// SSOT：cancel missing 返回 false。
    #[test]
    fn cancel_missing_returns_false() {
        let mut s = Scheduler::new();
        assert!(!s.cancel("never-scheduled"));
        assert!(s.list().is_empty());
    }

    /// SSOT：Default 为空登记表。
    #[test]
    fn default_is_empty() {
        let s = Scheduler::default();
        assert!(s.list().is_empty());
        // Debug/Clone 表面也走一遍，避免派生实现成为覆盖率盲区
        let _ = format!("{s:?}");
        let _ = s.clone();
    }

    /// SSOT：重复 schedule 幂等覆盖（不重复条目）。
    #[test]
    fn schedule_duplicate_is_idempotent() {
        let mut s = Scheduler::new();
        s.schedule("job-c");
        s.schedule("job-c");
        let ids = s.list();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "job-c");
    }
}
