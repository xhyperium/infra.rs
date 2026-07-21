# Changelog — schedulex

本文件记录 schedulex 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

### Added

- 真实 `benches/hot_path`（`cargo bench -- --quick` 可测）
- 公开 API 集成覆盖扩展（`tests/public_api_surface.rs` 等）
- `docs/API.md`：公开消费面与最小用法


## [0.1.0] - 2026-07-21

### 新增

- 在 **infra.rs** workspace 落地 `xhyper-schedulex` 0.1.0（lib `schedulex`）
- active SSOT 最小登记合同：`Scheduler::{new, schedule, cancel, list}` + `Default`
- std-only；无 timer / Clock / Job / async runtime
- 单元 + 集成测试覆盖五条 SSOT 行为；line coverage 目标 100%
