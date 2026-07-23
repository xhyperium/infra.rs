# MATRIX-SCHEDULEX-003

| 条款 | 实现 | public seam 证据 | 状态 |
|---|---|---|---|
| ID fail-closed | `runner::add` + `validate_task_id` | `add_rejects_unchecked_invalid_job_id_without_insertion` | PASS |
| Schedule fail-closed | `validate_schedule` | `add_rejects_publicly_constructible_invalid_schedule_without_insertion` | PASS |
| add 失败原子性 | 校验先于 `entries.insert` | `failed_replacement_preserves_existing_job_and_runtime_state` + cancellation | PASS |
| 执行稳定排序 | due ID sort | `tick_runs_same_due_set_in_lexical_job_id_order` | PASS |
| metadata 稳定排序 | `list_meta` sort | `list_meta_is_stable_in_lexical_job_id_order` | PASS |
| 回退 fail-closed | `last_tick_ms` | `regressed_logical_time_is_ignored_without_advancing_jobs` | PASS |
| Err 继续/推进 | `tick` | `tick_error_order_matches_execution_order_and_other_jobs_continue` | PASS |
| 无补跑 | FixedDelay 状态 | `fixed_delay_skips_missed_intervals_instead_of_replaying_them` | PASS |
| 重复 ID 完整替换 | `entries.insert` | `duplicate_job_id_replaces_callback_schedule_and_runtime_state` | PASS |
| cancel 存在语义 | cancel/remove | `cancel_reports_entry_existence_and_keeps_it_removable` | PASS |
| Cron 逻辑分钟 | minute index | `cron_uses_logical_time_and_fires_at_most_once_per_matching_minute` | PASS |
| Cron every interval | `last_fire_ms` | `cron_every_ms_is_stateful_interval_and_skips_missed_runs` + error/regression | PASS |
| panic 传播 | 不捕获 unwind | `job_panic_propagates_to_the_tick_caller` | PASS |
| 中文错误 | parser details | `schedule_validation_error_details_are_simplified_chinese` | PASS |
| timer/async/persistence/distributed | 无实现、无依赖 | Cargo 与源码审计 | NO-GO |
