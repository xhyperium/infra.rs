# Changelog

本文件记录 package `kernel`（lib 名 `kernel`）的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

## [0.3.1] - 2026-07-23 — deadline 失败语义显式化

### Changed

- `ShutdownSignal::wait_timeout` 返回 `Result<bool, WaitTimeoutError>`。
- `Duration::MAX` 等不可表示 deadline 返回 `DeadlineOverflow`，不再伪装为普通超时。
- 已触发状态优先于 deadline 构造；即使 timeout 不可表示，已完成等待仍立即返回 `Ok(true)`。

### Documentation

- 收敛 SPEC-KERNEL-002 的 `ClockDomain`、进程共享 origin、隐藏构造 seam 与公开面合同。

## [0.3.0] - 2026-07-21 — 内部生产发布（L1+L4）

### Added

- 可运行 `examples/basic.rs`（墙钟 + 关停 + 错误分类）
- `tests/public_api_surface.rs` 全量根 re-export 断言（含 `wait`、ClockError→XError、全 ComponentState 链）
- 真实 `benches/hot_path`（`cargo bench -p kernel --bench hot_path -- --quick`）
- `docs/API.md`：完整公开消费面；README 声明 **L1+L4** 生产层级与硬限制
- package 选择器统一为 Cargo 名 `kernel`（命令路径）
- crate 发布记录：[`releases/0.3.0-internal.md`](./releases/0.3.0-internal.md)

### Notes

- **已执行内部发布**：tag `v0.3.0-four-crates`；证据 `docs/plans/releases/2026-07-21-four-crates-internal-release.md`
- `publish = false`；**不** crates.io

### Historical

- 对齐 **SPEC-KERNEL-002** 可移植合同（§3–§8 / §11）：error / clock / lifecycle 生产面
- `tests/lifecycle_concurrency_loom.rs`（`cfg(loom)` Shutdown 模型测试）
- dev-deps：`static_assertions`；`cfg(loom)` 条件依赖 `loom`
- rustdoc `compile_fail` 负向面（字段私有、无 Default、无 Component、ShutdownGuard !Clone、禁 not_found/other/From&lt;str&gt;、Clock 无默认 monotonic）
- `api_compile`：`XError: !From<&str|String>`、`Timestamp: !Display/!From<SystemTime>`、serde 负向扩至 Signal/Guard
- proptest 属性测试：Timestamp checked 运算 / ComponentState 二元组合 / ErrorKind 构造矩阵
- ControlledClock 双通道测试替身；真实 mutex poison 恢复测试
- 本仓对齐文档：`docs/ssot/kernel-ssot-alignment.md`
- `publish = false`；`[lints] workspace = true`（继承根 `workspace.lints`）
- 按 crates 子模块标准补齐 `README.md`、`AGENTS.md`、`examples/`、`docs/` 骨架

### Fixed

- `Timestamp::checked_duration_since`：改用 `i128` 中间值，使 `i64::MAX - i64::MIN` 返回 `Some(u64::MAX nanos)` 而非溢出为 `None`

### Changed

- `lifecycle` 在 `cfg(loom)` 下切换 `loom::sync` 并发原语，供模型检验

---

## [0.3.0] — 2026-07-21

### Added

- L0 语义：`error`（`ErrorKind` / `XError`）、`clock`（墙钟 / 单调钟）、`lifecycle`（关停信号）
- 集成测试：`public_api`、`api_compile`、`clock_contract`、`lifecycle_concurrency`
