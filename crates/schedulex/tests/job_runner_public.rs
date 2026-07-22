//! `JobRunner` 公开 seam 的确定性与失败语义。

use schedulex::{CronParsed, Job, JobRunner, MAX_ID_LEN, Schedule, ScheduleError};
use std::sync::{Arc, Mutex};

#[test]
fn add_rejects_unchecked_invalid_job_id_without_insertion() {
    let mut runner = JobRunner::new();

    assert_eq!(
        runner.add(Job::new("", || Ok(())), Schedule::once(0)).unwrap_err(),
        ScheduleError::EmptyId
    );
    assert_eq!(
        runner.add(Job::new("a\nb", || Ok(())), Schedule::once(0)).unwrap_err(),
        ScheduleError::IdControlChar
    );
    assert_eq!(
        runner.add(Job::new("z".repeat(MAX_ID_LEN + 1), || Ok(())), Schedule::once(0)).unwrap_err(),
        ScheduleError::IdTooLong { max: MAX_ID_LEN }
    );

    assert_eq!(runner.active_len(), 0);
    assert!(runner.list_meta().is_empty());
}

#[test]
fn tick_runs_same_due_set_in_lexical_job_id_order() {
    for round in 0..32 {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut runner = JobRunner::new();
        for prefix in ["z", "a", "m"] {
            let id = format!("{prefix}-{round:02}");
            let recorded_id = id.clone();
            let calls = Arc::clone(&calls);
            runner
                .add(
                    Job::new(id, move || {
                        calls.lock().expect("calls lock").push(recorded_id.clone());
                        Ok(())
                    }),
                    Schedule::once(0),
                )
                .expect("valid job");
        }

        assert_eq!(runner.tick(0).fired, 3);
        assert_eq!(
            *calls.lock().expect("calls lock"),
            vec![format!("a-{round:02}"), format!("m-{round:02}"), format!("z-{round:02}"),]
        );
    }
}

#[test]
fn list_meta_is_stable_in_lexical_job_id_order() {
    for round in 0..32 {
        let mut runner = JobRunner::new();
        for prefix in ["z", "a", "m"] {
            runner
                .add(Job::new(format!("{prefix}-{round:02}"), || Ok(())), Schedule::once(0))
                .expect("valid job");
        }

        let ids: Vec<String> =
            runner.list_meta().into_iter().map(|meta| meta.id.as_str().to_owned()).collect();
        assert_eq!(
            ids,
            vec![format!("a-{round:02}"), format!("m-{round:02}"), format!("z-{round:02}"),]
        );
    }
}

#[test]
fn regressed_logical_time_is_ignored_without_advancing_jobs() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();

    assert_eq!(runner.tick(100).fired, 0);
    runner
        .add(
            Job::new("late-registration", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Ok(())
            }),
            Schedule::once(50),
        )
        .expect("valid job");

    assert_eq!(runner.tick(99).fired, 0);
    assert_eq!(*calls.lock().expect("calls lock"), 0);
    assert_eq!(runner.tick(100).fired, 1);
    assert_eq!(*calls.lock().expect("calls lock"), 1);
}

#[test]
fn tick_error_order_matches_execution_order_and_other_jobs_continue() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut runner = JobRunner::new();
    for (id, fails) in [("z-fail", true), ("a-fail", true), ("m-ok", false)] {
        let calls = Arc::clone(&calls);
        runner
            .add(
                Job::new(id, move || {
                    calls.lock().expect("calls lock").push(id);
                    if fails {
                        Err(ScheduleError::JobFailed(format!("{id} 失败")))
                    } else {
                        Ok(())
                    }
                }),
                Schedule::once(0),
            )
            .expect("valid job");
    }

    let result = runner.tick(0);
    assert_eq!(result.fired, 1);
    let error_ids: Vec<&str> = result.errors.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(error_ids, ["a-fail", "z-fail"]);
    assert_eq!(*calls.lock().expect("calls lock"), ["a-fail", "m-ok", "z-fail"]);

    assert_eq!(runner.tick(0).fired, 0);
    assert_eq!(calls.lock().expect("calls lock").len(), 3);
}

#[test]
fn fixed_delay_skips_missed_intervals_instead_of_replaying_them() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("fixed", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Ok(())
            }),
            Schedule::fixed_delay(10).expect("valid fixed delay"),
        )
        .expect("valid job");

    assert_eq!(runner.tick(0).fired, 1);
    assert_eq!(runner.tick(1_000).fired, 1);
    assert_eq!(*calls.lock().expect("calls lock"), 2);
    assert_eq!(runner.tick(1_005).fired, 0);
    assert_eq!(runner.tick(1_010).fired, 1);
    assert_eq!(*calls.lock().expect("calls lock"), 3);
}

#[test]
fn duplicate_job_id_replaces_callback_schedule_and_runtime_state() {
    let old_calls = Arc::new(Mutex::new(0_u32));
    let new_calls = Arc::new(Mutex::new(0_u32));
    let mut runner = JobRunner::new();

    let recorded_old = Arc::clone(&old_calls);
    runner
        .add(
            Job::new("replace", move || {
                *recorded_old.lock().expect("old lock") += 1;
                Ok(())
            }),
            Schedule::once(0),
        )
        .expect("valid old job");
    assert_eq!(runner.tick(0).fired, 1);

    let recorded_new = Arc::clone(&new_calls);
    runner
        .add(
            Job::new("replace", move || {
                *recorded_new.lock().expect("new lock") += 1;
                Ok(())
            }),
            Schedule::once(10),
        )
        .expect("valid replacement");

    assert_eq!(runner.active_len(), 1);
    assert_eq!(runner.tick(9).fired, 0);
    assert_eq!(runner.tick(10).fired, 1);
    assert_eq!(*old_calls.lock().expect("old lock"), 1);
    assert_eq!(*new_calls.lock().expect("new lock"), 1);
}

#[test]
fn cancel_reports_entry_existence_and_keeps_it_removable() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("cancelled", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Ok(())
            }),
            Schedule::once(0),
        )
        .expect("valid job");

    assert!(runner.cancel("cancelled"));
    assert!(runner.cancel("cancelled"));
    assert!(runner.contains("cancelled"));
    assert_eq!(runner.active_len(), 0);
    assert_eq!(runner.tick(0).fired, 0);
    assert_eq!(*calls.lock().expect("calls lock"), 0);
    assert!(runner.remove("cancelled"));
    assert!(!runner.cancel("cancelled"));
}

#[test]
fn cron_every_ms_is_stateful_interval_and_skips_missed_runs() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("interval", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Ok(())
            }),
            Schedule::cron("every:10").expect("valid interval"),
        )
        .expect("valid job");

    assert_eq!(runner.tick(7).fired, 1);
    assert_eq!(runner.tick(16).fired, 0);
    assert_eq!(runner.tick(17).fired, 1);
    assert_eq!(runner.tick(1_007).fired, 1);
    assert_eq!(*calls.lock().expect("calls lock"), 3);
}

#[test]
fn cron_every_ms_error_and_regressed_tick_preserve_interval_baseline() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("failing-interval", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Err(ScheduleError::JobFailed("预期失败".into()))
            }),
            Schedule::cron("every:10").expect("valid interval"),
        )
        .expect("valid job");

    assert_eq!(runner.tick(7).errors.len(), 1);
    assert!(runner.tick(6).errors.is_empty());
    assert!(runner.tick(8).errors.is_empty());
    assert!(runner.tick(16).errors.is_empty());
    assert_eq!(runner.tick(17).errors.len(), 1);
    assert_eq!(*calls.lock().expect("calls lock"), 2);
}

#[test]
fn cron_uses_logical_time_and_fires_at_most_once_per_matching_minute() {
    let calls = Arc::new(Mutex::new(0_u32));
    let recorded_calls = Arc::clone(&calls);
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("cron", move || {
                *recorded_calls.lock().expect("calls lock") += 1;
                Ok(())
            }),
            Schedule::cron("*/5 * * * *").expect("valid cron"),
        )
        .expect("valid job");

    assert_eq!(runner.tick(0).fired, 1);
    assert_eq!(runner.tick(1).fired, 0);
    assert_eq!(runner.tick(5 * 60_000).fired, 1);
    assert_eq!(runner.tick(5 * 60_000 + 59_999).fired, 0);
    assert_eq!(runner.tick(6 * 60_000).fired, 0);
    assert_eq!(*calls.lock().expect("calls lock"), 2);
}

#[test]
fn job_panic_propagates_to_the_tick_caller() {
    let mut runner = JobRunner::new();
    runner
        .add(
            Job::new("panic", || -> Result<(), ScheduleError> { panic!("job panic") }),
            Schedule::once(0),
        )
        .expect("valid job");

    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runner.tick(0)));
    assert!(panic.is_err());
}

#[test]
fn schedule_validation_error_details_are_simplified_chinese() {
    let cases = [
        (Schedule::fixed_delay(0), "必须大于 0"),
        (Schedule::cron(""), "不能为空"),
        (Schedule::cron("every:bad"), "毫秒"),
        (Schedule::cron(format!("every:{}s", u64::MAX)), "溢出"),
        (Schedule::cron("* * *"), "5 段"),
        (Schedule::cron("1 2 * * *"), "仅分钟字段"),
        (Schedule::cron("*/x * * * *"), "分钟步长"),
        (Schedule::cron("60 * * * *"), "分钟必须"),
    ];

    for (result, expected) in cases {
        let ScheduleError::InvalidSchedule(detail) = result.expect_err("must reject") else {
            panic!("expected invalid schedule")
        };
        assert!(detail.contains(expected), "detail={detail:?}, expected={expected:?}");
    }
}

#[test]
fn add_rejects_publicly_constructible_invalid_schedule_without_insertion() {
    let invalid = [
        Schedule::FixedDelay { every_ms: 0, first_at_ms: 0 },
        Schedule::Cron { expr: "every:10".into(), parsed: CronParsed::EveryMs { every_ms: 0 } },
        Schedule::Cron {
            expr: "60 * * * *".into(),
            parsed: CronParsed::MinuteMatch { every_n: None, exact: Some(60) },
        },
        Schedule::Cron { expr: "every:10".into(), parsed: CronParsed::EveryMs { every_ms: 20 } },
    ];
    let mut runner = JobRunner::new();

    for (index, schedule) in invalid.into_iter().enumerate() {
        let error = runner.add(Job::new(format!("bad-{index}"), || Ok(())), schedule).unwrap_err();
        assert!(matches!(error, ScheduleError::InvalidSchedule(_)));
    }

    assert_eq!(runner.active_len(), 0);
    assert!(runner.list_meta().is_empty());
}

#[test]
fn failed_replacement_preserves_existing_job_and_runtime_state() {
    let old_calls = Arc::new(Mutex::new(0_u32));
    let replacement_calls = Arc::new(Mutex::new(0_u32));
    let mut runner = JobRunner::new();

    let recorded_old = Arc::clone(&old_calls);
    runner
        .add(
            Job::new("stable", move || {
                *recorded_old.lock().expect("old lock") += 1;
                Ok(())
            })
            .with_name("原任务"),
            Schedule::fixed_delay(10).expect("valid fixed delay"),
        )
        .expect("valid job");
    assert_eq!(runner.tick(0).fired, 1);

    let recorded_replacement = Arc::clone(&replacement_calls);
    let error = runner
        .add(
            Job::new("stable", move || {
                *recorded_replacement.lock().expect("replacement lock") += 1;
                Ok(())
            })
            .with_name("无效替换"),
            Schedule::FixedDelay { every_ms: 0, first_at_ms: 0 },
        )
        .expect_err("invalid replacement must fail");
    assert!(matches!(error, ScheduleError::InvalidSchedule(_)));

    let metadata = runner.list_meta();
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].name.as_deref(), Some("原任务"));
    assert_eq!(runner.tick(9).fired, 0);
    assert_eq!(runner.tick(10).fired, 1);
    assert_eq!(*old_calls.lock().expect("old lock"), 2);
    assert_eq!(*replacement_calls.lock().expect("replacement lock"), 0);
}

#[test]
fn failed_replacement_preserves_existing_cancellation_state() {
    let mut runner = JobRunner::new();
    runner.add(Job::new("cancelled", || Ok(())), Schedule::once(0)).expect("valid job");
    assert!(runner.cancel("cancelled"));

    runner
        .add(
            Job::new("cancelled", || Ok(())),
            Schedule::Cron {
                expr: "every:10".into(),
                parsed: CronParsed::EveryMs { every_ms: 20 },
            },
        )
        .expect_err("invalid replacement must fail");

    assert_eq!(runner.active_len(), 0);
    assert_eq!(runner.tick(0).fired, 0);
    assert!(runner.cancel("cancelled"));
    assert!(runner.remove("cancelled"));
}
