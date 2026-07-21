//! 任务 ID 登记表：schedule / list / cancel。**不会**触发任何定时执行。
//!
//! ```bash
//! cargo run -p schedulex --example registry_only
//! ```
//!
//! # 生产红线
//! - 本 crate **不是** timer / cron / Job 执行器。
//! - `schedule(id)` 仅把 ID 放入集合；没有任何回调会在未来被调用。
//! - 需要真实调度时请使用外部调度器，勿扩展本 crate 冒充 production scheduler（SSOT §3 禁止）。

use schedulex::Scheduler;

fn main() {
    let mut s = Scheduler::new();
    s.schedule("job-a");
    s.schedule("job-b");
    s.schedule("job-a"); // 幂等覆盖

    let mut ids = s.list();
    ids.sort();
    assert_eq!(ids, vec!["job-a".to_string(), "job-b".to_string()]);

    assert!(s.cancel("job-a"));
    assert!(!s.cancel("job-a")); // 已不存在

    // 注意：此处没有任何 sleep / timer / 回调被触发。
    println!("schedulex_registry_ok remaining={:?} (IDs only; no timers fired)", s.list());
}
