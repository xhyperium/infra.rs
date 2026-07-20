# Changelog

本文件记录 `xhyper-kernel`（lib 名 `kernel`）的变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

版本号与 workspace `package.version` 对齐。

---

## [Unreleased]

### Added

- 对齐 **SPEC-KERNEL-002** 可移植合同（§3–§8 / §11）：error / clock / lifecycle 生产面
- `tests/lifecycle_concurrency_loom.rs`（`cfg(loom)` Shutdown 模型测试）
- dev-deps：`static_assertions`；`cfg(loom)` 条件依赖 `loom`
- rustdoc `compile_fail` 负向面（字段私有、无 Default、无 Component、ShutdownGuard !Clone）
- proptest 属性测试：Timestamp checked 运算 / mono 反向 / ErrorKind 构造矩阵
- ControlledClock 双通道测试替身；真实 mutex poison 恢复测试
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
