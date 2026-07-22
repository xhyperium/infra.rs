# Changelog — schedulex

本文件记录 schedulex 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### Changed

- `JobRunner::add` 在插入前统一校验 JobId 与公开 Schedule 状态。
- 同 tick 执行与 `list_meta` 按 Rust `str::cmp` 的 Job ID 字典序稳定。
- 时间回退 fail-closed；FixedDelay 大跨度不补跑；错误/取消/panic 语义文档化。
- `every:<ms>` 改为不会因 tick 偏离 epoch 而饥饿的 stateful interval；Err 推进且跨度不补跑。
- cron/调度错误详情改为简体中文。
- active SSOT、AGENTS、README 与现存 JobRunner 统一。

### Added

- 外部 public seam 集成测试，覆盖非法输入、失败原子性、顺序、时间、替换、取消、错误、Cron 与 panic。

## [0.1.1] - 2026-07-22

### Added

- std-only 的 `Job`、`Schedule` 与显式 `JobRunner::tick(now_ms)` additive 面。
- registry helpers、API 文档、bench 与公开面回归测试。

## [0.1.0] - 2026-07-21

### Added

- `Scheduler::{new, schedule, cancel, list}` + `Default` 任务 ID registry。
