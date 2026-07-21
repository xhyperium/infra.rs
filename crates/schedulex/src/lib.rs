//! schedulex —— L1 任务 ID 登记表（active SSOT：无真实定时器）。
//!
//! “登记一个任务 ID”不等于定时触发或执行任务。
//! 权威规范：`.agents/ssot/schedulex/spec/spec.md`

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

use std::collections::HashMap;

mod bulk;
mod id;
mod stats;
pub use bulk::{schedule_checked_many, schedule_filtering};
pub use id::{MAX_ID_LEN, debug_label, is_debug_label, normalize_task_id, validate_task_id};
pub use stats::{
    NO_HARD_CAPACITY, RegistryStats, is_busy, over_soft_threshold, stats, status_line, utilization,
};

/// 任务 ID 登记错误。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    /// 空或不合法 ID。
    EmptyId,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyId => write!(f, "task id must not be empty"),
        }
    }
}

impl std::error::Error for ScheduleError {}

/// 内存任务 ID 登记表（无时钟 / Job / runtime）。
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

    /// 登记任务 ID（重复幂等覆盖）。
    pub fn schedule(&mut self, id: impl Into<String>) {
        self.tasks.insert(id.into(), ());
    }

    /// 校验后登记。
    pub fn schedule_checked(&mut self, id: impl Into<String>) -> Result<(), ScheduleError> {
        let id = id.into();
        validate_task_id(&id)?;
        self.schedule(id);
        Ok(())
    }

    /// trim + 校验后登记。
    pub fn schedule_normalized(&mut self, id: &str) -> Result<(), ScheduleError> {
        let id = normalize_task_id(id)?;
        self.schedule(id);
        Ok(())
    }

    /// 仅当尚未登记时插入。
    pub fn try_schedule(&mut self, id: impl Into<String>) -> bool {
        let id = id.into();
        if self.tasks.contains_key(&id) {
            return false;
        }
        self.tasks.insert(id, ());
        true
    }

    /// 批量登记。
    pub fn schedule_many<I, S>(&mut self, ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for id in ids {
            self.schedule(id);
        }
    }

    /// 取消；返回是否曾存在。
    pub fn cancel(&mut self, id: &str) -> bool {
        self.tasks.remove(id).is_some()
    }

    /// 批量取消。
    pub fn cancel_many<'a, I>(&mut self, ids: I) -> usize
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut n = 0;
        for id in ids {
            if self.cancel(id) {
                n += 1;
            }
        }
        n
    }

    /// 是否已登记。
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.tasks.contains_key(id)
    }

    /// 条数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// 清空。
    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    /// 全部 ID（顺序未承诺）。
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// 交集。
    #[must_use]
    pub fn intersection_ids(&self, other: &Scheduler) -> Vec<String> {
        self.tasks.keys().filter(|k| other.contains(k)).cloned().collect()
    }

    /// 差集。
    #[must_use]
    pub fn difference_ids(&self, other: &Scheduler) -> Vec<String> {
        self.tasks.keys().filter(|k| !other.contains(k)).cloned().collect()
    }

    /// 并集。
    #[must_use]
    pub fn union_ids(&self, other: &Scheduler) -> Vec<String> {
        let mut m = self.tasks.clone();
        for k in other.tasks.keys() {
            m.insert(k.clone(), ());
        }
        m.into_keys().collect()
    }

    /// 保留满足谓词的 ID。
    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(&str) -> bool,
    {
        self.tasks.retain(|k, _| pred(k));
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

    #[test]
    fn schedule_then_list_contains_id() {
        let mut s = Scheduler::new();
        s.schedule("job-a");
        assert!(s.contains("job-a"));
        assert_eq!(s.len(), 1);
        let ids: HashSet<_> = s.list().into_iter().collect();
        assert_eq!(ids, HashSet::from(["job-a".to_string()]));
    }

    #[test]
    fn cancel_and_idempotent() {
        let mut s = Scheduler::new();
        s.schedule("job-b");
        assert!(s.cancel("job-b"));
        assert!(!s.cancel("job-b"));
        s.schedule("job-c");
        s.schedule("job-c");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn default_empty_debug_clone() {
        let s = Scheduler::default();
        assert!(s.is_empty());
        let _ = format!("{s:?}");
        let _ = s.clone();
    }

    #[test]
    fn checked_normalized_many() {
        let mut s = Scheduler::new();
        assert!(matches!(s.schedule_checked(""), Err(ScheduleError::EmptyId)));
        s.schedule_checked("ok").unwrap();
        s.schedule_normalized("  t1  ").unwrap();
        s.schedule_many(["a", "b"]);
        assert_eq!(s.cancel_many(["a", "x"]), 1);
        assert!(s.try_schedule("new"));
        assert!(!s.try_schedule("new"));
        let msg = format!("{}", ScheduleError::EmptyId);
        assert!(msg.contains("empty"));
    }

    #[test]
    fn set_ops_and_retain() {
        let mut a = Scheduler::new();
        a.schedule_many(["1", "2", "3"]);
        let mut b = Scheduler::new();
        b.schedule_many(["2", "4"]);
        assert_eq!(
            a.intersection_ids(&b).into_iter().collect::<HashSet<_>>(),
            HashSet::from(["2".to_string()])
        );
        assert!(a.difference_ids(&b).contains(&"1".to_string()));
        assert_eq!(a.union_ids(&b).len(), 4);
        a.retain(|id| id != "3");
        assert!(!a.contains("3"));
        a.clear();
        assert!(a.is_empty());
    }

    #[test]
    fn bulk_matrix() {
        let mut s = Scheduler::new();
        for i in 0..60 {
            s.schedule(format!("j{i}"));
        }
        assert_eq!(s.len(), 60);
        s.retain(|id| !id.ends_with('0'));
        assert!(s.len() < 60);
        let other = s.clone();
        assert_eq!(s.intersection_ids(&other).len(), s.len());
        let empty: [&str; 0] = [];
        s.schedule_many(empty);
        assert_eq!(s.cancel_many(empty), 0);
    }
}
