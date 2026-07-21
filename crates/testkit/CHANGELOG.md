# Changelog — testkit

本文件记录 testkit 的用户可见变更，遵循 [Keep a Changelog](https://keepachangelog.com/)。

## [Unreleased]

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
