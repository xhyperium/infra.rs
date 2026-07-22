# Changelog — testkit

本文件记录 testkit 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

## [0.1.3] - 2026-07-23 — IntegrationHarness fail-closed

### Changed

- `IntegrationHarness` 改为消费型 builder；`run(self)` 返回 terminal `HarnessReport`。
- 步骤错误保留 `Error::source`，panic 转为 terminal `HarnessRunError`，后续步骤不再执行。
- 使用 workspace `thiserror` 统一 crate 专用错误派生；补齐公开 getter/报告所有权与确定性并发重叠回归。
- `StepRecord` 字段私有化并提供 step 前后 snapshot getter；缺失快照使用 `Option`，禁止 epoch 0 哨兵。
- step 前后 snapshot 失败或 wall fault 形成 terminal `ObservationFailed`，不得记录为成功。

### Added

- `HarnessReport`、`HarnessRunError` 与四态 `StepOutcome`。

## [0.1.2] - 2026-07-22 — DEFER close: IntegrationHarness

### Added

- `IntegrationHarness`：基于 `ManualClock` 的多步确定性集成 harness
  （`step` / `step_advance_wall` / `step_advance_monotonic` / `run` / 断言辅助）
- `StepRecord` 结果记录；`dyn Clock` 经 `clock()` 访问内部 `ManualClock`
- unit 测试覆盖成功路径、首失败中止、空 run、缓存 re-run

## [0.1.1] - 2026-07-21 — four-crate production tranche（test-support L1）

### Added

- 可运行 `examples/basic.rs`（ManualClock 推进 + fault）
- `tests/public_api_surface.rs` 全量公开方法与 fault/error 变体断言
- 真实 `benches/hot_path`（`cargo bench -p testkit --bench hot_path -- --quick`）
- `docs/API.md` 完整面；README 声明 **L1 ManualClock test-support（非生产 runtime）**
- package 选择器统一为 `testkit`

### Notes

- 证据：`docs/plans/releases/2026-07-21-four-crates-internal-release.md`
- **不是** 生产 runtime；仅 dev-dep

### Historical

- 将 ManualClock V2 移植到 **infra.rs** workspace
- 合同 / 并发 / property 测试与 unit tests 一并落地
- SPEC-TESTKIT-002 core 对齐补齐：`mono_overflow` 单测、`api_compile`（!Default/!Clone/Send+Sync）、property（mono overflow + fault sequence）、`public_surface` 守卫
- 质量门禁：line-cov / miri / mutants 本地命令与 CI workflow（`testkit-*.yml`）

## [0.1.1] - 2026-07-14（上游 xhyper.rs ship；infra 移植保留版本号）

### 新增

- `ManualClock` V2：`Mutex` 状态模型；墙钟/单调钟 checked 控制 API；wall fault；`snapshot`；无 `Default` / `Clone`
- 合同/并发/property 测试
- crate 属性：`forbid(unsafe_code)`、`deny(missing_docs)`、`deny(unreachable_pub)`；`publish = false`

### 移除（相对历史草案）

- `xlib_test!`、`mock!`、`FixtureBuilder`、`provider_capability_contract_tests!`
