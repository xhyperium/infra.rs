//! 确定性 Job 运行器：`tick(now_ms)` 驱动，无墙钟依赖。

use std::collections::HashMap;

use crate::ScheduleError;
use crate::job::{Job, JobId, JobMeta};
use crate::schedule::{CronParsed, Schedule, cron_matches};

struct Entry {
    job: Job,
    schedule: Schedule,
    /// Once 是否已触发。
    fired_once: bool,
    /// FixedDelay / Cron EveryMs 上次触发时刻。
    last_fire_ms: Option<u64>,
    /// Cron MinuteMatch：上次触发的逻辑分钟索引。
    last_minute_index: Option<u64>,
    cancelled: bool,
}

/// 内存 Job 运行器。
///
/// 调用方注入 `now_ms`（可来自测试时钟或墙钟）；核心不读系统时间。
#[derive(Default)]
pub struct JobRunner {
    entries: HashMap<String, Entry>,
}

impl JobRunner {
    /// 空运行器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 job + schedule；重复 ID 覆盖。
    pub fn add(&mut self, job: Job, schedule: Schedule) -> Result<(), ScheduleError> {
        if let Schedule::FixedDelay { every_ms, .. } = &schedule {
            if *every_ms == 0 {
                return Err(ScheduleError::InvalidSchedule("every_ms must be > 0".into()));
            }
        }
        let id = job.id.as_str().to_string();
        self.entries.insert(
            id,
            Entry {
                job,
                schedule,
                fired_once: false,
                last_fire_ms: None,
                last_minute_index: None,
                cancelled: false,
            },
        );
        Ok(())
    }

    /// 取消；返回是否存在。
    pub fn cancel(&mut self, id: &str) -> bool {
        if let Some(e) = self.entries.get_mut(id) {
            e.cancelled = true;
            true
        } else {
            false
        }
    }

    /// 移除已取消或任意条目。
    pub fn remove(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// 是否包含（含已取消未移除）。
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    /// 活跃（未取消）数量。
    #[must_use]
    pub fn active_len(&self) -> usize {
        self.entries.values().filter(|e| !e.cancelled).count()
    }

    /// 元数据列表。
    #[must_use]
    pub fn list_meta(&self) -> Vec<JobMeta> {
        self.entries.values().map(|e| e.job.meta()).collect()
    }

    /// 推进时钟：运行所有在 `now_ms` 到期的 job。
    ///
    /// 返回成功触发次数；单个 job 错误记入返回的错误列表（其他 job 继续）。
    pub fn tick(&mut self, now_ms: u64) -> TickResult {
        let mut fired = 0usize;
        let mut errors = Vec::new();
        // 收集到期 ID，避免双重借用
        let due: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| !e.cancelled && is_due(e, now_ms))
            .map(|(id, _)| id.clone())
            .collect();

        for id in due {
            let Some(entry) = self.entries.get_mut(&id) else {
                continue;
            };
            match (entry.job.run)() {
                Ok(()) => {
                    fired += 1;
                    mark_fired(entry, now_ms);
                }
                Err(err) => {
                    // 仍推进 last_fire，避免紧密循环打爆
                    mark_fired(entry, now_ms);
                    errors.push((JobId::new(id), err));
                }
            }
        }
        TickResult { fired, errors }
    }
}

impl std::fmt::Debug for JobRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobRunner")
            .field("jobs", &self.entries.len())
            .field("active", &self.active_len())
            .finish()
    }
}

/// `tick` 结果。
#[derive(Debug, Default)]
pub struct TickResult {
    /// 成功执行次数。
    pub fired: usize,
    /// 失败列表。
    pub errors: Vec<(JobId, ScheduleError)>,
}

fn is_due(entry: &Entry, now_ms: u64) -> bool {
    match &entry.schedule {
        Schedule::Once { at_ms } => !entry.fired_once && now_ms >= *at_ms,
        Schedule::FixedDelay { every_ms, first_at_ms } => {
            if now_ms < *first_at_ms {
                return false;
            }
            match entry.last_fire_ms {
                None => true,
                Some(last) => now_ms.saturating_sub(last) >= *every_ms,
            }
        }
        Schedule::Cron { parsed, .. } => match parsed {
            CronParsed::EveryMs { every_ms } => match entry.last_fire_ms {
                None => cron_matches(parsed, now_ms) || now_ms == 0,
                Some(last) => {
                    now_ms.saturating_sub(last) >= *every_ms && cron_matches(parsed, now_ms)
                }
            },
            CronParsed::MinuteMatch { .. } => {
                if !cron_matches(parsed, now_ms) {
                    return false;
                }
                let minute_index = now_ms / 60_000;
                match entry.last_minute_index {
                    None => true,
                    Some(prev) => minute_index > prev,
                }
            }
        },
    }
}

fn mark_fired(entry: &mut Entry, now_ms: u64) {
    entry.last_fire_ms = Some(now_ms);
    entry.last_minute_index = Some(now_ms / 60_000);
    if matches!(entry.schedule, Schedule::Once { .. }) {
        entry.fired_once = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Job;
    use crate::schedule::Schedule;
    use std::sync::{Arc, Mutex};

    #[test]
    fn once_fires_at_tick() {
        let hits = Arc::new(Mutex::new(0u32));
        let h = Arc::clone(&hits);
        let mut runner = JobRunner::new();
        runner
            .add(
                Job::new("once", move || {
                    *h.lock().unwrap() += 1;
                    Ok(())
                }),
                Schedule::once(100),
            )
            .unwrap();
        assert_eq!(runner.tick(50).fired, 0);
        assert_eq!(runner.tick(100).fired, 1);
        assert_eq!(runner.tick(200).fired, 0);
        assert_eq!(*hits.lock().unwrap(), 1);
    }

    #[test]
    fn fixed_delay_refires() {
        let hits = Arc::new(Mutex::new(0u32));
        let h = Arc::clone(&hits);
        let mut runner = JobRunner::new();
        runner
            .add(
                Job::new("fd", move || {
                    *h.lock().unwrap() += 1;
                    Ok(())
                }),
                Schedule::fixed_delay(10).unwrap(),
            )
            .unwrap();
        assert_eq!(runner.tick(0).fired, 1);
        assert_eq!(runner.tick(5).fired, 0);
        assert_eq!(runner.tick(10).fired, 1);
        assert_eq!(runner.tick(20).fired, 1);
        assert_eq!(*hits.lock().unwrap(), 3);
    }

    #[test]
    fn cancel_stops_job() {
        let mut runner = JobRunner::new();
        runner.add(Job::new("c", || Ok(())), Schedule::fixed_delay(1).unwrap()).unwrap();
        assert!(runner.cancel("c"));
        assert_eq!(runner.tick(10).fired, 0);
        assert!(runner.contains("c"));
        assert!(runner.remove("c"));
        assert!(!runner.cancel("missing"));
    }

    #[test]
    fn invalid_cron_rejected() {
        assert!(Schedule::cron("not-a-cron").is_err());
        assert!(Schedule::cron("1 2 3 4 5").is_err());
    }
}
